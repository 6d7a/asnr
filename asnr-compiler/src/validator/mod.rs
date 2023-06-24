//! The `validator` module ensures that the
//! parsed data elements of the ASN1 specification
//! are valid declarations that can be further
//! processed. Among other things, the `validator`
//! assures that all dependencies of the individual
//! data elements resolve, and checks for conflicting
//! constraints and value definitions.
pub(crate) mod error;

use std::{collections::HashMap, error::Error};

use asnr_grammar::{
    information_object::{ASN1Information, ClassLink, InformationObjectClass},
    subtyping::*,
    types::*,
    *,
};

use self::error::{ValidatorError, ValidatorErrorType};

pub struct Validator {
    tlds: Vec<ToplevelDeclaration>,
}

impl Validator {
    pub fn new(tlds: Vec<ToplevelDeclaration>) -> Validator {
        Self { tlds }
    }

    fn link(mut self) -> Result<Self, ValidatorError> {
        let class_clones = self
            .tlds
            .iter()
            .filter_map(|tld| match tld {
                ToplevelDeclaration::Information(i) => match &i.value {
                    ASN1Information::ObjectClass(c) => Some((i.name.clone(), c.clone())),
                    _ => None,
                },
                _ => None,
            })
            .collect::<HashMap<String, InformationObjectClass>>();
        'io_link: while let Some(ToplevelDeclaration::Information(info)) =
            self.tlds.iter_mut().next()
        {
            match &info.value {
                ASN1Information::ObjectClass(_) => (),
                _ => {
                    let class_name = match &info.class {
                        Some(ClassLink::ByName(n)) => n.clone(),
                        Some(ClassLink::ByReference(_)) => continue 'io_link,
                        None => unreachable!(),
                    };
                    let class_copy = class_clones.get(class_name.as_str()).ok_or(ValidatorError { data_element: Some(info.name.clone()), details: format!("Linker Error: Cannot find Information Object Class \"{class_name}\" in parsed sources!"), kind: ValidatorErrorType::MissingDependencyError })?.clone();
                    info.class = Some(ClassLink::ByReference(class_copy.clone()))
                }
            }
        }
        Ok(self)
    }

    pub fn validate(
        mut self,
    ) -> Result<(Vec<ToplevelDeclaration>, Vec<Box<dyn Error>>), Box<dyn Error>> {
        self = self.link()?;
        Ok(self.tlds.into_iter().fold(
            (
                Vec::<ToplevelDeclaration>::new(),
                Vec::<Box<dyn Error>>::new(),
            ),
            |(mut tlds, mut errors), tld| {
                match tld.validate() {
                    Ok(_) => tlds.push(tld),
                    Err(e) => errors.push(Box::new(e)),
                }
                (tlds, errors)
            },
        ))
    }
}

pub trait Validate {
    fn validate(&self) -> Result<(), ValidatorError>;
}

impl Validate for ToplevelDeclaration {
    fn validate(&self) -> Result<(), ValidatorError> {
        match self {
            ToplevelDeclaration::Type(t) => {
                if let Err(mut e) = t.r#type.validate() {
                    e.specify_data_element(t.name.clone());
                    return Err(e);
                }
                Ok(())
            }
            ToplevelDeclaration::Value(_v) => Ok(()),
            ToplevelDeclaration::Information(_i) => Ok(()),
        }
    }
}

impl Validate for ASN1Type {
    fn validate(&self) -> Result<(), ValidatorError> {
        match self {
            ASN1Type::Integer(ref i) => i.validate(),
            ASN1Type::BitString(ref b) => b.validate(),
            ASN1Type::CharacterString(ref o) => o.validate(),
            _ => Ok(()),
        }
    }
}

impl Validate for Integer {
    fn validate(&self) -> Result<(), ValidatorError> {
        for c in &self.constraints {
            c.validate()?;
        }
        Ok(())
    }
}

impl Validate for BitString {
    fn validate(&self) -> Result<(), ValidatorError> {
        for c in &self.constraints {
            c.validate()?;
        }
        Ok(())
    }
}

impl Validate for CharacterString {
    fn validate(&self) -> Result<(), ValidatorError> {
        for c in &self.constraints {
            if let Constraint::ValueConstraint(r) = c {
                r.validate()?;
            }
        }
        Ok(())
    }
}

impl Validate for ValueConstraint {
    fn validate(&self) -> Result<(), ValidatorError> {
        if let Some((ASN1Value::Integer(min), ASN1Value::Integer(max))) =
            self.min_value.as_ref().zip(self.max_value.as_ref())
        {
            if min > max {
                return Err(ValidatorError::new(
                    None,
                    "Mininum value exceeds maximum value!",
                    ValidatorErrorType::InvalidConstraintsError,
                ));
            }
        }
        Ok(())
    }
}

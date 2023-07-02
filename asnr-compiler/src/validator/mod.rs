//! The `validator` module ensures that the
//! parsed data elements of the ASN1 specification
//! are valid declarations that can be further
//! processed. Among other things, the `validator`
//! assures that all dependencies of the individual
//! data elements resolve, and checks for conflicting
//! constraints and value definitions.
pub(crate) mod error;

use std::{error::Error};

use asnr_grammar::{
    constraints::*,
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

    fn link(mut self) -> Result<(Self, Vec<Box<dyn Error>>), ValidatorError> {
        let mut i = 0;
        let mut warnings: Vec<Box<dyn Error>> = vec![];
        while i < self.tlds.len() {
            if self
                .tlds
                .get(i)
                .map(|t| match t {
                    ToplevelDeclaration::Type(t) => t.r#type.contains_class_field_reference(),
                    _ => false,
                })
                .unwrap_or(false)
            {
                if let ToplevelDeclaration::Type(mut tld) = self.tlds.remove(i) {
                    tld.r#type = tld.r#type.resolve_class_field_reference(&self.tlds);
                    self.tlds.push(ToplevelDeclaration::Type(tld))
                }
            } else if self
                .tlds
                .get(i)
                .map(|t| t.has_constraint_reference())
                .unwrap_or(false)
            {
                let mut tld = self.tlds.remove(i);
                if !tld.link_constraint_reference(&self.tlds) {
                    warnings.push(
                        Box::new(
                            ValidatorError { 
                                data_element: Some(tld.name().to_string()), 
                                details: format!(
                                    "Failed to link cross-reference to elsewhere defined value in constraint of {}", 
                                    tld.name()), 
                                kind: ValidatorErrorType::MissingDependency
                            }
                        )
                    )
                }
                self.tlds.push(tld);
            } else {
                i += 1;
            }
        }

        Ok((self, warnings))
    }

    pub fn validate(
        mut self,
    ) -> Result<(Vec<ToplevelDeclaration>, Vec<Box<dyn Error>>), Box<dyn Error>> {
        let warnings: Vec<Box<dyn Error>>;
        (self, warnings) = self.link()?;
        Ok(self.tlds.into_iter().fold(
            (Vec::<ToplevelDeclaration>::new(), warnings),
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
            c.validate()?;
        }
        Ok(())
    }
}

impl Validate for Constraint {
    fn validate(&self) -> Result<(), ValidatorError> {
        if let Constraint::SubtypeConstraint(c) = self {
            if let ElementOrSetOperation::Element(SubtypeElement::ValueRange {
                min,
                max,
                extensible: _,
            }) = &c.set
            {
                if let Some((ASN1Value::Integer(min), ASN1Value::Integer(max))) =
                    min.as_ref().zip(max.as_ref())
                {
                    if min > max {
                        return Err(ValidatorError::new(
                            None,
                            "Mininum value exceeds maximum value!",
                            ValidatorErrorType::InvalidConstraintsError,
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

//! The `validator` module ensures that the
//! parsed data elements of the ASN1 specification
//! are valid declarations that can be further
//! processed. Among other things, the `validator`
//! assures that all dependencies of the individual
//! data elements resolve, and checks for conflicting
//! constraints and value definitions.
pub(crate) mod error;

use asnr_grammar::{subtyping::*, types::*, *};

use self::error::{ValidatorError, ValidatorErrorType};

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

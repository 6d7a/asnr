//! The `validator` module ensures that the
//! parsed data elements of the ASN1 specification
//! are valid declarations that can be further
//! processed. Among other things, the `validator`
//! assures that all dependencies of the individual
//! data elements resolve, and checks for conflicting
//! constraints and value definitions.
pub(crate) mod error;

use asnr_grammar::{
    ASN1Type, AsnBitString, AsnInteger, AsnOctetString, Constraint, ToplevelDeclaration,
};

use self::error::{ValidatorError, ValidatorErrorType};

pub trait Validate {
    fn validate(&self) -> Result<(), ValidatorError>;
}

impl Validate for ToplevelDeclaration {
  fn validate(&self) -> Result<(), ValidatorError> {
      if let Err(mut e) = self.r#type.validate() {
        e.specify_data_element(self.name.clone());
        return Err(e)
      }
      Ok(())
  }
}

impl Validate for ASN1Type {
    fn validate(&self) -> Result<(), ValidatorError> {
        match self {
            ASN1Type::Integer(ref i) => i.validate(),
            ASN1Type::BitString(ref b) => b.validate(),
            ASN1Type::OctetString(ref o) => o.validate(),
            _ => Ok(()),
        }
    }
}

impl Validate for AsnInteger {
    fn validate(&self) -> Result<(), ValidatorError> {
        self.constraint.as_ref().map_or(Ok(()), |c| c.validate())
    }
}

impl Validate for AsnBitString {
    fn validate(&self) -> Result<(), ValidatorError> {
        self.constraint.as_ref().map_or(Ok(()), |c| c.validate())
    }
}

impl Validate for AsnOctetString {
    fn validate(&self) -> Result<(), ValidatorError> {
        self.constraint.as_ref().map_or(Ok(()), |c| c.validate())
    }
}

impl Validate for Constraint {
    fn validate(&self) -> Result<(), ValidatorError> {
        if let Some((min, max)) = self.min_value.zip(self.max_value) {
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

use core::fmt::{Display, Formatter, Result};
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DecodingError {
    pub details: String,
    pub kind: DecodingErrorType,
}

impl DecodingError {
    pub fn new(details: &str, kind: DecodingErrorType) -> Self {
      DecodingError { details: details.into(), kind }
    }

}

#[derive(Debug, Clone)]
pub enum DecodingErrorType {
    InvalidEnumeratedIndex,
}

impl Error for DecodingError {}

impl Display for DecodingError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{:?} decoding ASN1 encoding: {}",
            self.kind, self.details
        )
    }
}

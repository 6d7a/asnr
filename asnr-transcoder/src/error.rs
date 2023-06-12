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
    InvalidChoiceIndex,
    InvalidSequenceMemberIndex,
    GenericParsingError,
}

impl Error for DecodingError {}

impl From<nom::Err<nom::error::Error<&[u8]>>> for DecodingError {
    fn from(value: nom::Err<nom::error::Error<&[u8]>>) -> Self {
        DecodingError { details: value.to_string(), kind: DecodingErrorType::GenericParsingError }
    }
}

impl Display for DecodingError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{:?} decoding ASN1 encoding: {}",
            self.kind, self.details
        )
    }
}

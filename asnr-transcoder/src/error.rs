use core::fmt::{Display, Formatter, Result};

use alloc::{
    format,
    string::{String},
};
use asnr_grammar::error::GrammarError;
use nom::{AsBytes};

#[derive(Debug, Clone)]
pub struct DecodingError {
    pub details: String,
    pub kind: DecodingErrorType,
}

impl DecodingError {
    pub fn new(details: &str, kind: DecodingErrorType) -> Self {
        DecodingError {
            details: details.into(),
            kind,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DecodingErrorType {
    InvalidEnumeratedIndex,
    InvalidChoiceIndex,
    InvalidSequenceMemberIndex,
    GenericParsingError,
    ConstraintError,
    Unsupported,
}

impl From<GrammarError> for DecodingError {
    fn from(value: GrammarError) -> Self {
        Self { details: value.details, kind: DecodingErrorType::ConstraintError }
    }
}

impl<I: AsBytes> From<nom::Err<nom::error::Error<I>>> for DecodingError {
    fn from(value: nom::Err<nom::error::Error<I>>) -> Self {
        let error = match value {
            nom::Err::Incomplete(req) => format!("Incomplete input. Needs {:?}", req),
            nom::Err::Error(e) => {
                format!("Encountered error with code {:?} while parsing.", e.code)
            }
            nom::Err::Failure(e) => format!("Encountered unrecoverable error with code {:?} while parsing.", e.code),
        };
        DecodingError {
            details: error,
            kind: DecodingErrorType::GenericParsingError,
        }
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
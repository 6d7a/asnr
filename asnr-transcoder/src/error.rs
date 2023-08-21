use core::fmt::{Debug, Display, Formatter, Result};

use alloc::{format, string::String};
use asnr_grammar::error::GrammarError;
use nom::{
    error::{ErrorKind, ParseError},
    AsBytes,
};

#[derive(Debug, Clone)]
pub struct DecodingError<I: AsBytes> {
    pub details: String,
    pub input: Option<I>,
    pub kind: DecodingErrorType,
}

#[derive(Debug, Clone)]
pub enum DecodingErrorType {
    InvalidEnumeratedIndex,
    InvalidChoiceIndex,
    InvalidSequenceMemberIndex,
    GenericParsingError,
    ConstraintError,
    Unsupported,
    WrappedNomError(ErrorKind),
}

impl<I: AsBytes> DecodingError<I> {
  pub fn new(details: &str, kind: DecodingErrorType) -> Self {
    Self {
      details: details.into(),
      kind,
      input: None
    }
  }
}

impl<I: AsBytes> From<GrammarError> for DecodingError<I> {
    fn from(value: GrammarError) -> Self {
        Self {
            details: value.details,
            kind: DecodingErrorType::ConstraintError,
            input: None,
        }
    }
}

impl<I: AsBytes> From<nom::Err<nom::error::Error<I>>> for DecodingError<I> {
    fn from(value: nom::Err<nom::error::Error<I>>) -> Self {
        let (error, input) = match value {
            nom::Err::Incomplete(req) => (format!("Incomplete input. Needs {:?}", req), None),
            nom::Err::Error(e) => (
                format!("Encountered error with code {:?} while parsing.", e.code),
                Some(e.input),
            ),
            nom::Err::Failure(e) => (
                format!(
                    "Encountered unrecoverable error with code {:?} while parsing.",
                    e.code
                ),
                Some(e.input),
            ),
        };
        DecodingError {
            details: error,
            kind: DecodingErrorType::GenericParsingError,
            input,
        }
    }
}

impl<I: AsBytes + Debug> ParseError<I> for DecodingError<I> {
    // on one line, we show the error code and the input that caused it
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        let details = format!("{:?}:\t{:?}\n", kind, input);
        DecodingError {
            details,
            input: Some(input),
            kind: DecodingErrorType::WrappedNomError(kind),
        }
    }

    // if combining multiple errors, we show them one after the other
    fn append(input: I, kind: ErrorKind, other: Self) -> Self {
        let details = format!("{}{:?}:\t{:?}\n", other.details, kind, input);
        DecodingError {
            details,
            input: Some(input),
            kind: DecodingErrorType::WrappedNomError(kind),
        }
    }

    fn from_char(input: I, c: char) -> Self {
        let details = format!("'{}':\t{:?}\n", c, input);
        DecodingError {
            details,
            input: Some(input),
            kind: DecodingErrorType::GenericParsingError,
        }
    }

    fn or(self, other: Self) -> Self {
        let details = format!("{}\tOR\n{}\n", self.details, other.details);
        DecodingError {
            details,
            input: None,
            kind: DecodingErrorType::GenericParsingError,
        }
    }
}

impl<I: AsBytes + Debug> Display for DecodingError<I> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{:?} decoding ASN1 encoding: {}",
            self.kind, self.details
        )
    }
}

#[derive(Debug, Clone)]
pub struct EncodingError {
    pub details: String,
}

impl<I: AsBytes> From<DecodingError<I>> for EncodingError {
    fn from(value: DecodingError<I>) -> Self {
        EncodingError { details: value.details }
    }
}

impl From<&str> for EncodingError {
    fn from(value: &str) -> Self {
        EncodingError { details: value.into() }
    }
}
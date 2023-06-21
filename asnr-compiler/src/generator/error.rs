use core::fmt::{Display, Formatter, Result};
use std::error::Error;

use asnr_grammar::ToplevelDeclaration;

#[derive(Debug, Clone)]
pub struct GeneratorError {
    pub top_level_declaration: ToplevelDeclaration,
    pub details: String,
    pub kind: GeneratorErrorType,
}

impl GeneratorError {
    pub fn new(tld: ToplevelDeclaration, details: &str, kind: GeneratorErrorType) -> Self {
        GeneratorError {
            top_level_declaration: tld,
            details: details.into(),
            kind,
        }
    }
}

#[derive(Debug, Clone)]
pub enum GeneratorErrorType {
    Asn1TypeMismatch,
    EmptyChoiceType,
}

impl Error for GeneratorError {}

impl Display for GeneratorError {
    fn fmt(&self, f: &mut Formatter) -> Result {
       let name = match &self.top_level_declaration {
        ToplevelDeclaration::Type(t) => &t.name,
        ToplevelDeclaration::Value(v) => &v.name,
    };
        write!(
            f,
            "{:?} generating Rust representation for {name}: {}",
            self.kind, self.details
        )
    }
}

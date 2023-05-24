//! The `generator` module is responsible
//! for generating rust code that handles
//! decoding and encoding of the parsed and
//! validated ASN1 data elements.

use asnr_grammar::ToplevelDeclaration;

use self::{error::GeneratorError, template::{boolean_template, integer_template, enumerated_template}};
pub(crate) mod error;
mod template;
mod util;

pub const GENERATED_RUST_IMPORTS: &str = r#"use asnr_grammar::*;
use asnr_transcoder::{error::{DecodingError, DecodingErrorType}, Decode};

"#;

pub fn generate<'a>(
    tld: ToplevelDeclaration,
    custom_derive: Option<&str>,
) -> Result<std::string::String, GeneratorError> {
    match tld.r#type {
        asnr_grammar::ASN1Type::Boolean => boolean_template(tld, custom_derive),
        asnr_grammar::ASN1Type::Integer(_) => integer_template(tld, custom_derive),
        asnr_grammar::ASN1Type::Enumerated(_) => enumerated_template(tld, custom_derive),
        _ => Ok("".into())
    }
}

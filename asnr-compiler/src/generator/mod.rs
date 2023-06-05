//! The `generator` module is responsible
//! for generating rust code that handles
//! decoding and encoding of the parsed and
//! validated ASN1 data elements.

use asnr_grammar::ToplevelDeclaration;

use self::{error::GeneratorError, template::{boolean_template, integer_template, enumerated_template, bit_string_template, octet_string_template, sequence_template}};
pub(crate) mod error;
mod template;
mod util;

pub const GENERATED_RUST_IMPORTS: &str = r#"use asnr_grammar::*;
use asnr_transcoder::{error::{DecodingError, DecodingErrorType}, Decode, Decoder};

"#;

pub fn generate<'a>(
    tld: ToplevelDeclaration,
    custom_derive: Option<&str>,
) -> Result<std::string::String, GeneratorError> {
    match tld.r#type {
        asnr_grammar::ASN1Type::Boolean => boolean_template(tld, custom_derive),
        asnr_grammar::ASN1Type::Integer(_) => integer_template(tld, custom_derive),
        asnr_grammar::ASN1Type::Enumerated(_) => enumerated_template(tld, custom_derive),
        asnr_grammar::ASN1Type::BitString(_) => bit_string_template(tld, custom_derive),
        asnr_grammar::ASN1Type::OctetString(_) => octet_string_template(tld, custom_derive),
        asnr_grammar::ASN1Type::Sequence(_) => sequence_template(tld, custom_derive),
        _ => Ok("".into())
    }
}

//! The `generator` module is responsible
//! for generating rust code that handles
//! decoding and encoding of the parsed and
//! validated ASN1 data elements.

use asnr_grammar::ToplevelDeclaration;

use self::{error::GeneratorError, builder::{generate_boolean, generate_integer, generate_enumerated, generate_bit_string, character_string_template, generate_sequence, generate_sequence_of, generate_choice}};
pub(crate) mod error;
mod builder;
pub(crate) mod template;
mod util;

pub fn generate<'a>(
    tld: ToplevelDeclaration,
    custom_derive: Option<&str>,
) -> Result<std::string::String, GeneratorError> {
    match tld.r#type {
        asnr_grammar::ASN1Type::Boolean => generate_boolean(tld, custom_derive),
        asnr_grammar::ASN1Type::Integer(_) => generate_integer(tld, custom_derive),
        asnr_grammar::ASN1Type::Enumerated(_) => generate_enumerated(tld, custom_derive),
        asnr_grammar::ASN1Type::BitString(_) => generate_bit_string(tld, custom_derive),
        asnr_grammar::ASN1Type::CharacterString(_) => character_string_template(tld, custom_derive),
        asnr_grammar::ASN1Type::Sequence(_) => generate_sequence(tld, custom_derive),
        asnr_grammar::ASN1Type::SequenceOf(_) => generate_sequence_of(tld, custom_derive),
        asnr_grammar::ASN1Type::Choice(_) => generate_choice(tld, custom_derive),
        _ => Ok("".into())
    }
}

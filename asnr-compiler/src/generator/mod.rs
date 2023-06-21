//! The `generator` module is responsible
//! for generating rust code that handles
//! decoding and encoding of the parsed and
//! validated ASN1 data elements.

use std::ffi::OsStr;

use asnr_grammar::{ToplevelTypeDeclaration, ToplevelDeclaration, ASN1Type};

use self::{error::GeneratorError, builder::{generate_boolean, generate_integer, generate_enumerated, generate_bit_string, character_string_template, generate_sequence, generate_sequence_of, generate_choice, generate_null, generate_typealias, generate_null_value, generate_integer_value, generate_information_object_class}};
pub(crate) mod error;
mod builder;
pub(crate) mod template;
mod util;

pub fn spec_section(name: Option<&OsStr>) -> String {
  format!(r#"

// ================================================
// {}
// ================================================

"#, name.map_or("", |os| os.to_str().unwrap_or("")))
}

pub fn generate<'a>(
    tld: ToplevelDeclaration,
    custom_derive: Option<&str>,
) -> Result<std::string::String, GeneratorError> {
  match tld {
    ToplevelDeclaration::Type(t) => {
      match t.r#type {
        ASN1Type::Null => generate_null(t, custom_derive),
        ASN1Type::Boolean => generate_boolean(t, custom_derive),
        ASN1Type::Integer(_) => generate_integer(t, custom_derive),
        ASN1Type::Enumerated(_) => generate_enumerated(t, custom_derive),
        ASN1Type::BitString(_) => generate_bit_string(t, custom_derive),
        ASN1Type::CharacterString(_) => character_string_template(t, custom_derive),
        ASN1Type::Sequence(_) => generate_sequence(t, custom_derive),
        ASN1Type::SequenceOf(_) => generate_sequence_of(t, custom_derive),
        ASN1Type::Choice(_) => generate_choice(t, custom_derive),
        ASN1Type::ElsewhereDeclaredType(_) => generate_typealias(t, custom_derive),
        ASN1Type::InformationObjectClass(_) => generate_information_object_class(t, custom_derive),
        _ => Ok("".into())
    }
    },
    ToplevelDeclaration::Value(v) => {
      match v.value {
        asnr_grammar::ASN1Value::Null => generate_null_value(v),
        asnr_grammar::ASN1Value::Boolean(_) => todo!(),
        asnr_grammar::ASN1Value::Integer(_) => generate_integer_value(v),
        asnr_grammar::ASN1Value::String(_) => todo!(),
        asnr_grammar::ASN1Value::BitString(_) => todo!(),
        asnr_grammar::ASN1Value::EnumeratedValue(_) => todo!(),
        asnr_grammar::ASN1Value::ElsewhereDeclaredValue(_) => todo!(),
    }
    },
}
}

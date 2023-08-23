//! The `generator` module is responsible for generating rust code that handles
//! decoding and encoding of the parsed and validated ASN1 data elements.
//! The `generator` uses string templates for generating rust code. 

use asnr_grammar::{*, information_object::*};

use self::{
    builder::{
        character_string_template, generate_bit_string, generate_boolean, generate_choice,
        generate_enumerated, generate_information_object_class, generate_integer,
        generate_integer_value, generate_null, generate_null_value, generate_sequence,
        generate_sequence_of, generate_typealias, generate_choice_value, generate_sequence_value, generate_octet_string,
    },
    error::GeneratorError,
};
mod builder;
pub(crate) mod error;
pub(crate) mod template;
pub(crate) mod util;

pub fn generate<'a>(
    tld: ToplevelDeclaration,
    custom_derive: Option<&str>,
) -> Result<std::string::String, GeneratorError> {
    match tld {
        ToplevelDeclaration::Type(t) => match t.r#type {
            ASN1Type::Null => generate_null(t, custom_derive),
            ASN1Type::Boolean => generate_boolean(t, custom_derive),
            ASN1Type::Integer(_) => generate_integer(t, custom_derive),
            ASN1Type::Enumerated(_) => generate_enumerated(t, custom_derive),
            ASN1Type::BitString(_) => generate_bit_string(t, custom_derive),
            ASN1Type::OctetString(_) => generate_octet_string(t, custom_derive),
            ASN1Type::CharacterString(_) => character_string_template(t, custom_derive),
            ASN1Type::Sequence(_) => generate_sequence(t, custom_derive),
            ASN1Type::SequenceOf(_) => generate_sequence_of(t, custom_derive),
            ASN1Type::Choice(_) => generate_choice(t, custom_derive),
            ASN1Type::ElsewhereDeclaredType(_) => generate_typealias(t, custom_derive),
            _ => Ok("".into()),
        },
        ToplevelDeclaration::Value(v) => match v.value {
            ASN1Value::Null => generate_null_value(v),
            ASN1Value::Boolean(_) => todo!(),
            ASN1Value::Integer(_) => generate_integer_value(v),
            ASN1Value::String(_) => todo!(),
            ASN1Value::Real(_) => todo!(),
            ASN1Value::BitString(_) => todo!(),
            ASN1Value::EnumeratedValue(_) => todo!(),
            ASN1Value::ElsewhereDeclaredValue(_) => {
              println!("{:?}", v);
              todo!()
            },
            ASN1Value::All => todo!(),
            ASN1Value::Choice(_,_) => generate_choice_value(v),
            ASN1Value::Sequence(_) => generate_sequence_value(v),
        },
        ToplevelDeclaration::Information(i) => match i.value {
            ASN1Information::ObjectClass(_) => {
                generate_information_object_class(i, custom_derive)
            }
            // ASN1Information::ObjectSet(_) => {
            //   generate_information_object_set(i)
            // }
            _ => {
              Ok("".into())
            },
        },
    }
}

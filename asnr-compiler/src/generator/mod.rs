//! The `generator` module is responsible for generating rust code that handles
//! decoding and encoding of the parsed and validated ASN1 data elements.
//! The `generator` uses string templates for generating rust code.
use std::ffi::OsStr;

use crate::Framework;
use asnr_grammar::{information_object::*, *};

<<<<<<< HEAD
use self::{error::GeneratorError, templates::asnr::builder::AsnrGenerator};
=======
use self::{
    builder::{
        character_string_template, generate_bit_string, generate_boolean, generate_choice,
        generate_enumerated, generate_information_object_class, generate_integer,
        generate_integer_value, generate_null, generate_null_value, generate_sequence,
        generate_sequence_of, generate_typealias, generate_choice_value, generate_sequence_value,
    },
    error::GeneratorError,
};
mod builder;
>>>>>>> 14a7746 (feat(uper): encode choices)
pub(crate) mod error;
pub(crate) mod templates;

pub fn spec_section(name: Option<&OsStr>) -> String {
    format!(
        r#"

// ================================================
// {}
// ================================================

"#,
        name.map_or("", |os| os.to_str().unwrap_or(""))
    )
}

pub trait Generator {
    fn generate_integer_value<'a>(tld: ToplevelValueDeclaration) -> Result<String, GeneratorError>;
    fn generate_integer<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError>;
    fn generate_bit_string<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError>;
    fn character_string_template<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError>;
    fn generate_boolean<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError>;
    fn generate_typealias<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError>;
    fn generate_null_value<'a>(tld: ToplevelValueDeclaration) -> Result<String, GeneratorError>;
    fn generate_null<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError>;
    fn generate_enumerated<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError>;
    fn generate_choice<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError>;
    fn generate_information_object_class<'a>(
        tld: ToplevelInformationDeclaration,
    ) -> Result<String, GeneratorError>;
    fn generate_sequence<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError>;
    fn generate_sequence_of<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError>;
    fn generate_information_object_set<'a>(
        tld: ToplevelInformationDeclaration,
    ) -> Result<String, GeneratorError>;
}

pub fn generate<'a>(
    framework: &Framework,
    tld: ToplevelDeclaration,
    custom_derive: Option<&str>,
) -> Result<std::string::String, GeneratorError> {
<<<<<<< HEAD
    match framework {
        Framework::Asnr => {
            match tld {
                ToplevelDeclaration::Type(t) => match t.r#type {
                    ASN1Type::Null => AsnrGenerator::generate_null(t, custom_derive),
                    ASN1Type::Boolean => AsnrGenerator::generate_boolean(t, custom_derive),
                    ASN1Type::Integer(_) => AsnrGenerator::generate_integer(t, custom_derive),
                    ASN1Type::Enumerated(_) => AsnrGenerator::generate_enumerated(t, custom_derive),
                    ASN1Type::BitString(_) => AsnrGenerator::generate_bit_string(t, custom_derive),
                    ASN1Type::CharacterString(_) => {
                        AsnrGenerator::character_string_template(t, custom_derive)
                    }
                    ASN1Type::Sequence(_) => AsnrGenerator::generate_sequence(t, custom_derive),
                    ASN1Type::SequenceOf(_) => {
                        AsnrGenerator::generate_sequence_of(t, custom_derive)
                    }
                    ASN1Type::Choice(_) => AsnrGenerator::generate_choice(t, custom_derive),
                    ASN1Type::ElsewhereDeclaredType(_) => {
                        AsnrGenerator::generate_typealias(t, custom_derive)
                    }
                    _ => Ok("".into()),
                },
                ToplevelDeclaration::Value(v) => match v.value {
                    ASN1Value::Null => AsnrGenerator::generate_null_value(v),
                    ASN1Value::Boolean(_) => todo!(),
                    ASN1Value::Integer(_) => AsnrGenerator::generate_integer_value(v),
                    ASN1Value::String(_) => todo!(),
                    ASN1Value::BitString(_) => todo!(),
                    ASN1Value::EnumeratedValue(_) => todo!(),
                    ASN1Value::ElsewhereDeclaredValue(_) => todo!(),
                    ASN1Value::All => todo!(),
                },
                ToplevelDeclaration::Information(i) => match i.value {
                    ASN1Information::ObjectClass(_) => {
                        AsnrGenerator::generate_information_object_class(i)
                    }
                    // ASN1Information::ObjectSet(_) => {
                    //   generate_information_object_set(i)
                    // }
                    _ => Ok("".into()),
                },
=======
    match tld {
        ToplevelDeclaration::Type(t) => match t.r#type {
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
>>>>>>> 14a7746 (feat(uper): encode choices)
            }
        },
        Framework::Rasn => todo!()
    }
}

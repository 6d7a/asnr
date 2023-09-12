//! The `generator` module is responsible for generating rust code that handles
//! decoding and encoding of the parsed and validated ASN1 data elements.
//! The `generator` uses string templates for generating rust code. 

use crate::Framework;
use asnr_grammar::{information_object::*, *};

pub(crate) mod error;
pub(crate) mod templates;
use self::{
    error::GeneratorError,
    templates::{
        asnr::{builder::AsnrGenerator, template::asnr_imports_and_generic_types},
        rasn::{builder::RasnGenerator, template::rasn_imports_and_generic_types},
    },
};

pub fn imports_and_generic_types(framework: &Framework, custom_derive: Option<&str>, no_std: bool, include_file_headers: bool) -> String {
    match framework {
        Framework::Asnr => asnr_imports_and_generic_types(custom_derive, no_std, include_file_headers),
        Framework::Rasn => rasn_imports_and_generic_types(include_file_headers),
    }
}

pub trait Generator {
    fn generate_choice_value(tld: ToplevelValueDeclaration) -> Result<String, GeneratorError>;
    fn generate_octet_string<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError>;
    fn generate_integer_value<'a>(tld: ToplevelValueDeclaration) -> Result<String, GeneratorError>;
    fn generate_integer<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError>;
    fn generate_bit_string<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError>;
    fn generate_character_string<'a>(
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
    fn generate_sequence_value(tld: ToplevelValueDeclaration) -> Result<String, GeneratorError>;
}

pub fn generate<'a>(
    framework: &Framework,
    tld: ToplevelDeclaration,
    custom_derive: Option<&str>,
) -> Result<std::string::String, GeneratorError> {
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
                        AsnrGenerator::generate_character_string(t, custom_derive)
                    }
                    ASN1Type::Sequence(_) => AsnrGenerator::generate_sequence(t, custom_derive),
                    ASN1Type::SequenceOf(_) => {
                        AsnrGenerator::generate_sequence_of(t, custom_derive)
                    }
                    ASN1Type::Choice(_) => AsnrGenerator::generate_choice(t, custom_derive),
                    ASN1Type::ElsewhereDeclaredType(_) => {
                        AsnrGenerator::generate_typealias(t, custom_derive)
                    }
                    ASN1Type::OctetString(_) => {
                        AsnrGenerator::generate_octet_string(t, custom_derive)
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
                    ASN1Value::Choice(_, _) => AsnrGenerator::generate_choice_value(v),
                    ASN1Value::Sequence(_) => AsnrGenerator::generate_sequence_value(v),
                    ASN1Value::Real(_) => todo!(),
                    ASN1Value::ObjectIdentifier(_) => todo!()
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
            }
        }
        Framework::Rasn => match tld {
            ToplevelDeclaration::Type(t) => match t.r#type {
                ASN1Type::Null => RasnGenerator::generate_null(t, custom_derive),
                ASN1Type::Boolean => RasnGenerator::generate_boolean(t, custom_derive),
                ASN1Type::Integer(_) => RasnGenerator::generate_integer(t, custom_derive),
                ASN1Type::Enumerated(_) => RasnGenerator::generate_enumerated(t, custom_derive),
                ASN1Type::BitString(_) => RasnGenerator::generate_bit_string(t, custom_derive),
                ASN1Type::CharacterString(_) => {
                    RasnGenerator::generate_character_string(t, custom_derive)
                }
                ASN1Type::Sequence(_) | ASN1Type::Set(_) => {
                    RasnGenerator::generate_sequence_or_set(t, custom_derive)
                }
                ASN1Type::SequenceOf(_) => {
                    RasnGenerator::generate_sequence_of(t, custom_derive)
                }
                ASN1Type::ElsewhereDeclaredType(_) => {
                    RasnGenerator::generate_typealias(t, custom_derive)
                }
                ASN1Type::Choice(_) => RasnGenerator::generate_choice(t, custom_derive),
                ASN1Type::OctetString(_) => {
                    RasnGenerator::generate_octet_string(t, custom_derive)
                }
                _ => Ok("".into()),
            },
            ToplevelDeclaration::Value(v) => match v.value {
                ASN1Value::Null => RasnGenerator::generate_null_value(v),
                ASN1Value::Boolean(_) => todo!(),
                ASN1Value::Integer(_) => RasnGenerator::generate_integer_value(v),
                ASN1Value::String(_) => todo!(),
                ASN1Value::BitString(_) => todo!(),
                ASN1Value::EnumeratedValue(_) => todo!(),
                ASN1Value::ElsewhereDeclaredValue(_) => todo!(),
                ASN1Value::All => todo!(),
                // ASN1Value::Choice(_, _) => RasnGenerator::generate_choice_value(v),
                // ASN1Value::Sequence(_) => RasnGenerator::generate_sequence_value(v),
                ASN1Value::Real(_) => todo!(),
                ASN1Value::ObjectIdentifier(_) => RasnGenerator::generate_object_identifier_value(v),
                _ => Ok("".into()),
            },
            ToplevelDeclaration::Information(i) => match i.value {
                // ASN1Information::ObjectClass(_) => {
                //     RasnGenerator::generate_information_object_class(i)
                // }
                // ASN1Information::ObjectSet(_) => {
                //   generate_information_object_set(i)
                // }
                _ => Ok("".into()),
            },
        },
    }
}

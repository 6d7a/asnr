use crate::{
    generator::Generator,
    generator::{
        error::{GeneratorError, GeneratorErrorType},
        generate,
    },
    utils::{to_rust_camel_case, to_rust_title_case},
    Framework,
};
use asnr_grammar::{information_object::*, utils::int_type_token, *};
use asnr_traits::*;

use super::{template::*, util::*};

pub struct StringifiedNameType {
    pub name: String,
    pub r#type: String,
}

pub struct AsnrGenerator;

impl Generator for AsnrGenerator {
    fn generate_integer_value(tld: ToplevelValueDeclaration) -> Result<String, GeneratorError> {
        if let ASN1Value::Integer(i) = tld.value {
            if tld.type_name == INTEGER {
                Ok(integer_value_template(
                    format_comments(&tld.comments),
                    to_rust_camel_case(&tld.name),
                    int_type_token(i, i),
                    i.to_string(),
                ))
            } else {
                Ok(integer_value_template(
                    format_comments(&tld.comments),
                    to_rust_camel_case(&tld.name),
                    tld.type_name.as_str(),
                    format!("{}({})", tld.type_name, i),
                ))
            }
        } else {
            Err(GeneratorError::new(
                Some(ToplevelDeclaration::Value(tld)),
                "Expected INTEGER value top-level declaration",
                GeneratorErrorType::Asn1TypeMismatch,
            ))
        }
    }

    fn generate_integer<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError> {
        if let ASN1Type::Integer(ref int) = tld.r#type {
            Ok(integer_template(
                format_comments(&tld.comments),
                custom_derive.unwrap_or(DERIVE_DEFAULT),
                to_rust_title_case(&tld.name),
                int.type_token(),
                format_distinguished_values(&tld),
                int.declare(),
            ))
        } else {
            Err(GeneratorError::new(
                Some(ToplevelDeclaration::Type(tld)),
                "Expected INTEGER top-level declaration",
                GeneratorErrorType::Asn1TypeMismatch,
            ))
        }
    }

    fn generate_bit_string<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError> {
        if let ASN1Type::BitString(ref bitstr) = tld.r#type {
            Ok(bit_string_template(
                format_comments(&tld.comments),
                custom_derive.unwrap_or(DERIVE_DEFAULT),
                to_rust_title_case(&tld.name),
                format_distinguished_values(&tld),
                bitstr.declare(),
            ))
        } else {
            Err(GeneratorError::new(
                Some(ToplevelDeclaration::Type(tld)),
                "Expected BIT STRING top-level declaration",
                GeneratorErrorType::Asn1TypeMismatch,
            ))
        }
    }

    fn generate_octet_string<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError> {
        if let ASN1Type::OctetString(ref oct_str) = tld.r#type {
            Ok(octet_string_template(
                format_comments(&tld.comments),
                custom_derive.unwrap_or(DERIVE_DEFAULT),
                to_rust_title_case(&tld.name),
                oct_str.declare(),
            ))
        } else {
            Err(GeneratorError::new(
                Some(ToplevelDeclaration::Type(tld)),
                "Expected OCTET STRING top-level declaration",
                GeneratorErrorType::Asn1TypeMismatch,
            ))
        }
    }

    fn generate_character_string<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError> {
        if let ASN1Type::CharacterString(ref char_str) = tld.r#type {
            Ok(char_string_template(
                format_comments(&tld.comments),
                custom_derive.unwrap_or(DERIVE_DEFAULT),
                to_rust_title_case(&tld.name),
                char_str.declare(),
            ))
        } else {
            Err(GeneratorError::new(
                Some(ToplevelDeclaration::Type(tld)),
                "Expected Character String top-level declaration",
                GeneratorErrorType::Asn1TypeMismatch,
            ))
        }
    }

    fn generate_boolean<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError> {
        if let ASN1Type::Boolean = tld.r#type {
            Ok(boolean_template(
                format_comments(&tld.comments),
                custom_derive.unwrap_or(DERIVE_DEFAULT),
                to_rust_title_case(&tld.name),
            ))
        } else {
            Err(GeneratorError::new(
                Some(ToplevelDeclaration::Type(tld)),
                "Expected BOOLEAN top-level declaration",
                GeneratorErrorType::Asn1TypeMismatch,
            ))
        }
    }

    fn generate_typealias<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError> {
        if let ASN1Type::ElsewhereDeclaredType(dec) = &tld.r#type {
            Ok(typealias_template(
                format_comments(&tld.comments),
                custom_derive.unwrap_or(DERIVE_DEFAULT),
                to_rust_title_case(&tld.name),
                to_rust_title_case(&dec.identifier),
                tld.r#type.declare(),
            ))
        } else {
            Err(GeneratorError::new(
                Some(ToplevelDeclaration::Type(tld)),
                "Expected type alias top-level declaration",
                GeneratorErrorType::Asn1TypeMismatch,
            ))
        }
    }

    fn generate_null_value(tld: ToplevelValueDeclaration) -> Result<String, GeneratorError> {
        if let ASN1Value::Null = tld.value {
            Ok(null_value_template(
                format_comments(&tld.comments),
                to_rust_camel_case(&tld.name),
            ))
        } else {
            Err(GeneratorError::new(
                Some(ToplevelDeclaration::Value(tld)),
                "Expected NULL value top-level declaration",
                GeneratorErrorType::Asn1TypeMismatch,
            ))
        }
    }

    fn generate_null<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError> {
        if let ASN1Type::Null = tld.r#type {
            Ok(null_template(
                format_comments(&tld.comments),
                custom_derive.unwrap_or(DERIVE_DEFAULT),
                to_rust_title_case(&tld.name),
            ))
        } else {
            Err(GeneratorError::new(
                Some(ToplevelDeclaration::Type(tld)),
                "Expected NULL top-level declaration",
                GeneratorErrorType::Asn1TypeMismatch,
            ))
        }
    }

    fn generate_enumerated<'a>(
        mut tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError> {
        if let ASN1Type::Enumerated(ref mut enumerated) = tld.r#type {
            enumerated.members.sort_by(|a, b| a.index.cmp(&b.index));
            handle_duplicate_enumerals(&mut enumerated.members);
            let name = to_rust_title_case(&tld.name);
            let mut enumerals = enumerated
                .members
                .iter()
                .map(format_enumeral)
                .collect::<Vec<String>>()
                .join("\n\t");
            if enumerated.extensible.is_some() {
                enumerals.push_str("\n\tUnknownExtension")
            }
            let unknown_index_case = if enumerated.extensible.is_some() {
                format!("Ok(Self::UnknownExtension)")
            } else {
                format!(
                    r#"Err(
              DecodingError {{
                details: format!("Invalid enumerated index decoding {name}. Received index {{}}",v), 
                kind: DecodingErrorType::InvalidEnumeratedIndex,
                input: None
              }}
            )"#
                )
            };
            let enumerals_from_int = enumerated
                .members
                .iter()
                .map(format_enumeral_from_int)
                .collect::<Vec<String>>()
                .join("\n\t\t  ");
            Ok(enumerated_template(
                format_comments(&tld.comments),
                custom_derive.unwrap_or(DERIVE_DEFAULT),
                name,
                enumerals,
                enumerals_from_int,
                unknown_index_case,
                enumerated.declare(),
            ))
        } else {
            Err(GeneratorError::new(
                Some(ToplevelDeclaration::Type(tld)),
                "Expected ENUMERATED top-level declaration",
                GeneratorErrorType::Asn1TypeMismatch,
            ))
        }
    }

    fn generate_choice_value(tld: ToplevelValueDeclaration) -> Result<String, GeneratorError> {
        if let ASN1Value::Choice(id, val) = tld.value {
            let type_name = to_rust_camel_case(&tld.type_name);
            Ok(choice_value_template(
                format_comments(&tld.comments),
                to_rust_camel_case(&tld.name),
                &type_name,
                id,
                val.value_as_string(Some(&type_name))?,
            ))
        } else {
            Err(GeneratorError::new(
                Some(ToplevelDeclaration::Value(tld)),
                "Expected CHOICE value top-level declaration",
                GeneratorErrorType::Asn1TypeMismatch,
            ))
        }
    }

    fn generate_choice<'a>(
        mut tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError> {
        if let ASN1Type::Choice(ref mut choice) = tld.r#type {
            handle_duplicate_options(&mut choice.options);
            let name = to_rust_title_case(&tld.name);
            let inner_options = flatten_nested_choice_options(&choice.options, &name).join("\n");
            let options = extract_choice_options(&choice.options, &name);
            let mut options_declaration = format_option_declaration(&options);
            if choice.extensible.is_some() {
                options_declaration.push_str("\n\tUnknownChoiceValue(Vec<u8>)");
            }
            let unknown_index_case = if choice.extensible.is_some() {
                r#"_ => Ok(|input| D::decode_unknown_extension(input).map(|(r, v)|(r, Self::UnknownChoiceValue(v))))"#.to_owned()
            } else {
                format!(
                    r#"x => Err(
  DecodingError::new(
    &format!("Invalid choice index decoding {name}. Received {{x}}"),
    DecodingErrorType::InvalidChoiceIndex
  )
)"#
                )
            };
            let default_option = match options.first() {
                Some(o) => default_choice(o),
                None => {
                    return Err(GeneratorError {
                        top_level_declaration: Some(ToplevelDeclaration::Type(tld)),
                        details: "Empty CHOICE types are not yet supported!".into(),
                        kind: GeneratorErrorType::EmptyChoiceType,
                    })
                }
            };
            let options_from_int: String = options
                .iter()
                .enumerate()
                .map(format_option_from_int)
                .collect::<Vec<String>>()
                .join("\n\t\t  ");
            let encoder_options_body: String = options
                .iter()
                .enumerate()
                .map(format_option_encoder_from_int)
                .collect::<Vec<String>>()
                .join("\n\t\t  ");
            Ok(choice_template(
                format_comments(&tld.comments),
                custom_derive.unwrap_or("#[derive(Debug, Clone, PartialEq)]"),
                name,
                inner_options,
                default_option,
                options_declaration,
                encoder_options_body,
                options_from_int,
                unknown_index_case,
                choice.declare(),
            ))
        } else {
            Err(GeneratorError::new(
                Some(ToplevelDeclaration::Type(tld)),
                "Expected CHOICE top-level declaration",
                GeneratorErrorType::Asn1TypeMismatch,
            ))
        }
    }

    fn generate_information_object_class<'a>(
        tld: ToplevelInformationDeclaration,
    ) -> Result<String, GeneratorError> {
        if let ASN1Information::ObjectClass(ref ioc) = tld.value {
            Ok(information_object_class_template(
                format_comments(&tld.comments),
                to_rust_title_case(&tld.name),
                ioc.declare(),
            ))
        } else {
            Err(GeneratorError::new(
                Some(ToplevelDeclaration::Information(tld)),
                "Expected CLASS top-level declaration",
                GeneratorErrorType::Asn1TypeMismatch,
            ))
        }
    }

    fn generate_sequence_value(tld: ToplevelValueDeclaration) -> Result<String, GeneratorError> {
        if let ASN1Value::Sequence(_) = tld.value {
            let type_name = to_rust_camel_case(&tld.type_name);
            Ok(sequence_value_template(
                format_comments(&tld.comments),
                to_rust_camel_case(&tld.name),
                &type_name,
                tld.value.value_as_string(Some(&type_name))?,
            ))
        } else {
            Err(GeneratorError::new(
                Some(ToplevelDeclaration::Value(tld)),
                "Expected CHOICE value top-level declaration",
                GeneratorErrorType::Asn1TypeMismatch,
            ))
        }
    }

    fn generate_sequence<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError> {
        if let ASN1Type::Sequence(ref seq) = tld.r#type {
            let name = to_rust_title_case(&tld.name);
            let members = extract_sequence_members(&seq.members, &name, seq.extensible);
            let extension_decoder = format_extensible_sequence(&name, seq.extensible.is_some());

            Ok(sequence_template(
                format_comments(&tld.comments),
                custom_derive.unwrap_or(DERIVE_DEFAULT),
                flatten_nested_sequence_members(&seq.members, &name)?.join("\n"),
                name,
                format_member_declaration(&members),
                format_decode_member_body(&members),
                format_encoder_member_body(&members),
                format_has_optional_body(&members),
                extension_decoder,
                seq.declare(),
            ))
        } else {
            Err(GeneratorError::new(
                Some(ToplevelDeclaration::Type(tld)),
                "Expected SEQUENCE top-level declaration",
                GeneratorErrorType::Asn1TypeMismatch,
            ))
        }
    }

    fn generate_sequence_of<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError> {
        if let ASN1Type::SequenceOf(ref seq_of) = tld.r#type {
            let name = to_rust_title_case(&tld.name);
            let anonymous_item = match seq_of.r#type.as_ref() {
                ASN1Type::ElsewhereDeclaredType(_) => None,
                n => Some(generate(
                    &Framework::Asnr,
                    ToplevelDeclaration::Type(ToplevelTypeDeclaration {
                        parameterization: None,
                        comments: " Anonymous SEQUENCE OF member ".into(),
                        name: String::from("Anonymous") + &name,
                        r#type: n.clone(),
                        tag: None,
                    }),
                    None,
                )?),
            }
            .ok_or(GeneratorError {
                details: format!("Could not generate SEQUENCE OF member for {}", tld.name),
                top_level_declaration: Some(ToplevelDeclaration::Type(tld.clone())),
                kind: GeneratorErrorType::Asn1TypeMismatch,
            })?;
            let member_type = match seq_of.r#type.as_ref() {
                ASN1Type::ElsewhereDeclaredType(d) => to_rust_title_case(&d.identifier),
                _ => String::from("Anonymous") + &name,
            };
            Ok(sequence_of_template(
                format_comments(&tld.comments),
                custom_derive.unwrap_or(DERIVE_DEFAULT),
                name,
                anonymous_item,
                member_type,
                seq_of.declare(),
            ))
        } else {
            Err(GeneratorError::new(
                Some(ToplevelDeclaration::Type(tld)),
                "Expected SEQUENCE OF top-level declaration",
                GeneratorErrorType::Asn1TypeMismatch,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use asnr_grammar::{constraints::*, types::*, *};

    use crate::generator::templates::asnr::builder::*;

    #[test]
    fn generates_enumerated_from_template() {
        let enum_tld = ToplevelTypeDeclaration {
            parameterization: None,

            name: "TestEnum".into(),
            comments: "".into(),
            r#type: ASN1Type::Enumerated(Enumerated {
                constraints: vec![],
                members: vec![
                    Enumeral {
                        name: "forward".into(),
                        description: Some("This means forward".into()),
                        index: 1,
                    },
                    Enumeral {
                        name: "backward".into(),
                        description: Some("This means backward".into()),
                        index: 2,
                    },
                    Enumeral {
                        name: "unavailable".into(),
                        description: Some("This means nothing".into()),
                        index: 3,
                    },
                ],
                extensible: None,
            }),
            tag: None,
        };
        println!(
            "{}",
            AsnrGenerator::generate_enumerated(enum_tld, None).unwrap()
        )
    }

    #[test]
    fn generates_bitstring_from_template() {
        let bs_tld = ToplevelTypeDeclaration {
            parameterization: None,
            name: "BitString".into(),
            comments: "".into(),
            r#type: ASN1Type::BitString(BitString {
                constraints: vec![Constraint::SubtypeConstraint(ElementSet {
                    set: ElementOrSetOperation::Element(SubtypeElement::SizeConstraint(Box::new(
                        ElementOrSetOperation::Element(SubtypeElement::ValueRange {
                            min: Some(ASN1Value::Integer(8)),
                            max: Some(ASN1Value::Integer(18)),
                            extensible: false,
                        }),
                    ))),
                    extensible: false,
                })],
                distinguished_values: Some(vec![
                    DistinguishedValue {
                        name: "firstBit".into(),
                        value: 0,
                    },
                    DistinguishedValue {
                        name: "secondBit".into(),
                        value: 0,
                    },
                    DistinguishedValue {
                        name: "thirdBit".into(),
                        value: 0,
                    },
                ]),
            }),
            tag: None,
        };
        println!(
            "{}",
            AsnrGenerator::generate_bit_string(bs_tld, None).unwrap()
        )
    }

    #[test]
    fn generates_integer_from_template() {
        let int_tld = ToplevelTypeDeclaration {
            parameterization: None,
            name: "TestInt".into(),
            comments: "".into(),
            r#type: ASN1Type::Integer(Integer {
                constraints: vec![Constraint::SubtypeConstraint(ElementSet {
                    set: ElementOrSetOperation::Element(SubtypeElement::ValueRange {
                        min: Some(ASN1Value::Integer(8)),
                        max: Some(ASN1Value::Integer(18)),
                        extensible: false,
                    }),
                    extensible: false,
                })],
                distinguished_values: Some(vec![
                    DistinguishedValue {
                        name: "negativeOutOfRange".into(),
                        value: -16898,
                    },
                    DistinguishedValue {
                        name: "positiveOutOfRange".into(),
                        value: 16898,
                    },
                    DistinguishedValue {
                        name: "invalid".into(),
                        value: 16899,
                    },
                ]),
            }),
            tag: None,
        };
        println!(
            "{}",
            AsnrGenerator::generate_integer(int_tld, None).unwrap()
        )
    }

    #[test]
    fn generates_sequence_from_template() {
        let seq_tld = ToplevelTypeDeclaration {
            parameterization: None,
            name: "SequenceOrSet".into(),
            comments: "".into(),
            r#type: ASN1Type::Sequence(SequenceOrSet {
                constraints: vec![],
                extensible: Some(1),
                members: vec![SequenceOrSetMember {
                    name: "nested".into(),
                    tag: None,
                    r#type: ASN1Type::Sequence(SequenceOrSet {
                        extensible: Some(3),
                        constraints: vec![],
                        members: vec![
                            SequenceOrSetMember {
                                name: "wow".into(),
                                tag: None,
                                r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere {
                                    identifier: "Wow".into(),
                                    constraints: vec![],
                                }),
                                default_value: None,
                                is_optional: false,
                                constraints: vec![],
                            },
                            SequenceOrSetMember {
                                name: "this-is-annoying".into(),
                                tag: None,
                                r#type: ASN1Type::Boolean,
                                default_value: Some(ASN1Value::Boolean(true)),
                                is_optional: true,
                                constraints: vec![],
                            },
                            SequenceOrSetMember {
                                name: "another".into(),
                                tag: None,
                                r#type: ASN1Type::Sequence(SequenceOrSet {
                                    extensible: None,
                                    constraints: vec![],
                                    members: vec![SequenceOrSetMember {
                                        name: "inner".into(),

                                        tag: None,
                                        r#type: ASN1Type::BitString(BitString {
                                            constraints: vec![Constraint::SubtypeConstraint(
                                                ElementSet {
                                                    set: ElementOrSetOperation::Element(
                                                        SubtypeElement::SizeConstraint(Box::new(
                                                            ElementOrSetOperation::Element(
                                                                SubtypeElement::ValueRange {
                                                                    min: Some(ASN1Value::Integer(
                                                                        8,
                                                                    )),
                                                                    max: Some(ASN1Value::Integer(
                                                                        18,
                                                                    )),
                                                                    extensible: false,
                                                                },
                                                            ),
                                                        )),
                                                    ),
                                                    extensible: false,
                                                },
                                            )],
                                            distinguished_values: None,
                                        }),
                                        default_value: Some(ASN1Value::String("0".into())),
                                        is_optional: true,
                                        constraints: vec![],
                                    }],
                                }),
                                default_value: None,
                                is_optional: true,
                                constraints: vec![],
                            },
                        ],
                    }),
                    default_value: None,
                    is_optional: false,
                    constraints: vec![],
                }],
            }),
            tag: None,
        };
        println!(
            "{}",
            AsnrGenerator::generate_sequence(seq_tld, None).unwrap()
        )
    }
}

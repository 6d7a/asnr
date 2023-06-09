use asnr_grammar::{ASN1Type, Quote, ToplevelDeclaration};

use super::{
    error::{GeneratorError, GeneratorErrorType},
    generate,
    template::{boolean_template, *},
    util::*,
};

pub struct StringifiedNameType {
    pub name: String,
    pub r#type: String,
}

pub fn generate_integer<'a>(
    tld: ToplevelDeclaration,
    custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Type::Integer(ref int) = tld.r#type {
        let integer_type = int
            .constraint
            .as_ref()
            .map_or("i128", |c| c.int_type_token());
        Ok(integer_template(
            format_comments(&tld.comments),
            custom_derive.unwrap_or(DERIVE_DEFAULT),
            rustify_name(&tld.name),
            integer_type,
            format_distinguished_values(&tld),
            int.quote(),
        ))
    } else {
        Err(GeneratorError::new(
            tld,
            "Expected INTEGER top-level declaration",
            GeneratorErrorType::Asn1TypeMismatch,
        ))
    }
}

pub fn generate_bit_string<'a>(
    tld: ToplevelDeclaration,
    custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Type::BitString(ref bitstr) = tld.r#type {
        Ok(bit_string_template(
            format_comments(&tld.comments),
            custom_derive.unwrap_or(DERIVE_DEFAULT),
            rustify_name(&tld.name),
            format_distinguished_values(&tld),
            bitstr.quote(),
        ))
    } else {
        Err(GeneratorError::new(
            tld,
            "Expected BIT STRING top-level declaration",
            GeneratorErrorType::Asn1TypeMismatch,
        ))
    }
}

pub fn character_string_template<'a>(
    tld: ToplevelDeclaration,
    custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Type::CharacterString(ref char_str) = tld.r#type {
        Ok(char_string_template(
            format_comments(&tld.comments),
            custom_derive.unwrap_or(DERIVE_DEFAULT),
            rustify_name(&tld.name),
            char_str.quote(),
        ))
    } else {
        Err(GeneratorError::new(
            tld,
            "Expected Character String top-level declaration",
            GeneratorErrorType::Asn1TypeMismatch,
        ))
    }
}

pub fn generate_boolean<'a>(
    tld: ToplevelDeclaration,
    custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Type::Boolean = tld.r#type {
        Ok(boolean_template(
            format_comments(&tld.comments),
            custom_derive.unwrap_or(DERIVE_DEFAULT),
            rustify_name(&tld.name),
        ))
    } else {
        Err(GeneratorError::new(
            tld,
            "Expected BOOLEAN top-level declaration",
            GeneratorErrorType::Asn1TypeMismatch,
        ))
    }
}

pub fn generate_enumerated<'a>(
    tld: ToplevelDeclaration,
    custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Type::Enumerated(ref enumerated) = tld.r#type {
        let enumerals = enumerated
            .members
            .iter()
            .map(format_enumeral)
            .collect::<Vec<String>>()
            .join("\n\t");
        let enumerals_from_int = enumerated
            .members
            .iter()
            .map(format_enumeral_from_int)
            .collect::<Vec<String>>()
            .join("\n\t\t  ");
        Ok(enumerated_template(
            format_comments(&tld.comments),
            custom_derive.unwrap_or(DERIVE_DEFAULT),
            rustify_name(&tld.name),
            enumerals,
            enumerals_from_int,
            enumerated.quote(),
        ))
    } else {
        Err(GeneratorError::new(
            tld,
            "Expected ENUMERATED top-level declaration",
            GeneratorErrorType::Asn1TypeMismatch,
        ))
    }
}

// pub fn choice_template<'a>(
//   tld: ToplevelDeclaration,
//   custom_derive: Option<&'a str>,
// ) -> Result<String, GeneratorError> {
//   if let ASN1Type::Choice(ref choice) = tld.r#type {
//       let name = rustify_name(&tld.name);
//       let comments = format_comments(&tld.comments);
//       let derive = custom_derive.unwrap_or("#[derive(Debug, Clone, PartialEq, Default)]");
//       let inner_options = flatten_nested_choice_options(&choice.options).join("\n");
//       let members = extract_sequence_members(&seq.members);
//       let member_declaration = format_member_declaration(&members);
//       let member_decoding = format_sequence_instance_construction(&members);
//       let (extension_decl, extension_marker, extension_decoder) =
//           format_extensible_sequence(seq.extensible.is_some());
//       Ok(format!(
//           r#"
// {inner_options}

// {comments}{derive}
// pub enum {name} {{
// {member_declaration}{extension_decl}
// }}

// impl Decode for {name} {{
// fn decode<'a, D>(decoder: &D, input: &'a [u8]) -> IResult<&'a [u8], Self>
// where
//   D: Decoder,
//   Self: Sized,
// {{
//     let mut remaining = input;
//     let mut instance = Self::default();{extension_marker}
//     {member_decoding};{extension_decoder}
//     Ok((remaining, instance))
// }}
// }}
// "#
//       ))
//   } else {
//       Err(GeneratorError::new(
//           tld,
//           "Expected SEQUENCE top-level declaration",
//           GeneratorErrorType::Asn1TypeMismatch,
//       ))
//   }
// }

pub fn generate_sequence<'a>(
    tld: ToplevelDeclaration,
    custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Type::Sequence(ref seq) = tld.r#type {
        let name = rustify_name(&tld.name);
        let members = extract_sequence_members(&seq.members);
        let (extension_decl, extension_decoder) =
            format_extensible_sequence(&name, seq.extensible.is_some());

        Ok(sequence_template(
            format_comments(&tld.comments),
            custom_derive.unwrap_or(DERIVE_DEFAULT),
            name,
            flatten_nested_sequence_members(&seq.members).join("\n"),
            format_member_declaration(&members),
            extension_decl,
            format_decode_member_body(&members),
            extension_decoder,
            seq.quote(),
        ))
    } else {
        Err(GeneratorError::new(
            tld,
            "Expected SEQUENCE top-level declaration",
            GeneratorErrorType::Asn1TypeMismatch,
        ))
    }
}

pub fn generate_sequence_of<'a>(
    tld: ToplevelDeclaration,
    custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Type::SequenceOf(ref seq_of) = tld.r#type {
        let name = rustify_name(&tld.name);
        let anonymous_item = match seq_of.r#type.as_ref() {
            ASN1Type::ElsewhereDeclaredType(_) => None,
            n => Some(generate(
                ToplevelDeclaration {
                    comments: " Anonymous SEQUENCE OF member ".into(),
                    name: String::from("Anonymous_") + &name,
                    r#type: n.clone(),
                },
                None,
            )?),
        }
        .unwrap_or(String::new());
        let member_type = match seq_of.r#type.as_ref() {
            ASN1Type::ElsewhereDeclaredType(d) => rustify_name(&d.0),
            _ => String::from("Anonymous_") + &name,
        };
        Ok(sequence_of_template(
            format_comments(&tld.comments),
            custom_derive.unwrap_or(DERIVE_DEFAULT),
            name,
            anonymous_item,
            member_type,
            seq_of.quote()
        ))
    } else {
        Err(GeneratorError::new(
            tld,
            "Expected SEQUENCE OF top-level declaration",
            GeneratorErrorType::Asn1TypeMismatch,
        ))
    }
}

#[cfg(test)]
mod tests {
    use asnr_grammar::{
        ASN1Type, ASN1Value, AsnBitString, AsnEnumerated, AsnInteger, AsnSequence, Constraint,
        DeclarationElsewhere, DistinguishedValue, Enumeral, SequenceMember, ToplevelDeclaration,
    };

    use crate::generator::builder::{
        generate_bit_string, generate_enumerated, generate_integer, generate_sequence,
    };

    #[test]
    fn generates_enumerated_from_template() {
        let enum_tld = ToplevelDeclaration {
            name: "TestEnum".into(),
            comments: "".into(),
            r#type: ASN1Type::Enumerated(AsnEnumerated {
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
        };
        println!("{}", generate_enumerated(enum_tld, None).unwrap())
    }

    #[test]
    fn generates_bitstring_from_template() {
        let bs_tld = ToplevelDeclaration {
            name: "BitString".into(),
            comments: "".into(),
            r#type: ASN1Type::BitString(AsnBitString {
                constraint: Some(Constraint {
                    max_value: Some(8),
                    min_value: Some(8),
                    extensible: true,
                }),
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
        };
        println!("{}", generate_bit_string(bs_tld, None).unwrap())
    }

    #[test]
    fn generates_integer_from_template() {
        let int_tld = ToplevelDeclaration {
            name: "TestInt".into(),
            comments: "".into(),
            r#type: ASN1Type::Integer(AsnInteger {
                constraint: Some(Constraint {
                    max_value: Some(1),
                    min_value: Some(8),
                    extensible: false,
                }),
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
        };
        println!("{}", generate_integer(int_tld, None).unwrap())
    }

    #[test]
    fn generates_sequence_from_template() {
        let seq_tld = ToplevelDeclaration {
            name: "Sequence".into(),
            comments: "".into(),
            r#type: ASN1Type::Sequence(AsnSequence {
                extensible: Some(1),
                members: vec![SequenceMember {
                    name: "nested".into(),
                    r#type: ASN1Type::Sequence(AsnSequence {
                        extensible: Some(3),
                        members: vec![
                            SequenceMember {
                                name: "wow".into(),
                                r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere(
                                    "Wow".into(),
                                )),
                                default_value: None,
                                is_optional: false,
                            },
                            SequenceMember {
                                name: "this-is-annoying".into(),
                                r#type: ASN1Type::Boolean,
                                default_value: Some(ASN1Value::Boolean(true)),
                                is_optional: true,
                            },
                            SequenceMember {
                                name: "another".into(),
                                r#type: ASN1Type::Sequence(AsnSequence {
                                    extensible: None,
                                    members: vec![SequenceMember {
                                        name: "inner".into(),
                                        r#type: ASN1Type::BitString(AsnBitString {
                                            constraint: Some(Constraint {
                                                min_value: Some(1),
                                                max_value: Some(1),
                                                extensible: true,
                                            }),
                                            distinguished_values: None,
                                        }),
                                        default_value: Some(ASN1Value::String("0".into())),
                                        is_optional: true,
                                    }],
                                }),
                                default_value: None,
                                is_optional: true,
                            },
                        ],
                    }),
                    default_value: None,
                    is_optional: false,
                }],
            }),
        };
        println!("{}", generate_sequence(seq_tld, None).unwrap())
    }
}
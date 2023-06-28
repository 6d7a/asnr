use asnr_grammar::{information_object::*, utils::int_type_token, *};
use asnr_traits::*;

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

pub fn generate_integer_value(tld: ToplevelValueDeclaration) -> Result<String, GeneratorError> {
    if let ASN1Value::Integer(i) = tld.value {
        if tld.type_name == INTEGER {
            Ok(integer_value_template(
                format_comments(&tld.comments),
                rustify_name(&tld.name),
                int_type_token(i, i),
                i.to_string(),
            ))
        } else {
            Ok(integer_value_template(
                format_comments(&tld.comments),
                rustify_name(&tld.name),
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

pub fn generate_integer<'a>(
    tld: ToplevelTypeDeclaration,
    custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Type::Integer(ref int) = tld.r#type {
        let integer_min_max = int.constraints.iter().fold((0i128, 0i128), |acc, c| {
            if let Some((ASN1Value::Integer(min), ASN1Value::Integer(max))) =
                c.min_value.as_ref().zip(c.max_value.as_ref())
            {
                (acc.0.min(*min), acc.1.max(*max))
            } else {
                acc
            }
        });
        Ok(integer_template(
            format_comments(&tld.comments),
            custom_derive.unwrap_or(DERIVE_DEFAULT),
            rustify_name(&tld.name),
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

pub fn generate_bit_string<'a>(
    tld: ToplevelTypeDeclaration,
    custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Type::BitString(ref bitstr) = tld.r#type {
        Ok(bit_string_template(
            format_comments(&tld.comments),
            custom_derive.unwrap_or(DERIVE_DEFAULT),
            rustify_name(&tld.name),
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

pub fn character_string_template<'a>(
    tld: ToplevelTypeDeclaration,
    custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Type::CharacterString(ref char_str) = tld.r#type {
        Ok(char_string_template(
            format_comments(&tld.comments),
            custom_derive.unwrap_or(DERIVE_DEFAULT),
            rustify_name(&tld.name),
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

pub fn generate_boolean<'a>(
    tld: ToplevelTypeDeclaration,
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
            Some(ToplevelDeclaration::Type(tld)),
            "Expected BOOLEAN top-level declaration",
            GeneratorErrorType::Asn1TypeMismatch,
        ))
    }
}

pub fn generate_typealias<'a>(
    tld: ToplevelTypeDeclaration,
    custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Type::ElsewhereDeclaredType(dec) = &tld.r#type {
        Ok(typealias_template(
            format_comments(&tld.comments),
            custom_derive.unwrap_or(DERIVE_DEFAULT),
            rustify_name(&tld.name),
            rustify_name(&dec.identifier),
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

pub fn generate_null_value(tld: ToplevelValueDeclaration) -> Result<String, GeneratorError> {
    if let ASN1Value::Null = tld.value {
        Ok(null_value_template(
            format_comments(&tld.comments),
            rustify_name(&tld.name),
        ))
    } else {
        Err(GeneratorError::new(
            Some(ToplevelDeclaration::Value(tld)),
            "Expected NULL value top-level declaration",
            GeneratorErrorType::Asn1TypeMismatch,
        ))
    }
}

pub fn generate_null<'a>(
    tld: ToplevelTypeDeclaration,
    custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Type::Null = tld.r#type {
        Ok(null_template(
            format_comments(&tld.comments),
            custom_derive.unwrap_or(DERIVE_DEFAULT),
            rustify_name(&tld.name),
        ))
    } else {
        Err(GeneratorError::new(
            Some(ToplevelDeclaration::Type(tld)),
            "Expected NULL top-level declaration",
            GeneratorErrorType::Asn1TypeMismatch,
        ))
    }
}

pub fn generate_enumerated<'a>(
    tld: ToplevelTypeDeclaration,
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

pub fn generate_choice<'a>(
    tld: ToplevelTypeDeclaration,
    custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Type::Choice(ref choice) = tld.r#type {
        let name = rustify_name(&tld.name);
        let inner_options = flatten_nested_choice_options(&choice.options, &name).join("\n");
        let options = extract_choice_options(&choice.options, &name);
        let options_declaration = format_option_declaration(&options);
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
        Ok(choice_template(
            format_comments(&tld.comments),
            custom_derive.unwrap_or("#[derive(Debug, Clone, PartialEq)]"),
            name,
            inner_options,
            default_option,
            options_declaration,
            options_from_int,
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

pub fn generate_information_object_class<'a>(
    tld: ToplevelInformationDeclaration,
    _custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Information::ObjectClass(ref ioc) = tld.value {
        Ok(information_object_class_template(
            format_comments(&tld.comments),
            rustify_name(&tld.name),
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

pub fn generate_sequence<'a>(
    tld: ToplevelTypeDeclaration,
    custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Type::Sequence(ref seq) = tld.r#type {
        let name = rustify_name(&tld.name);
        let members = extract_sequence_members(&seq.members, &name);
        let (extension_decl, extension_decoder) =
            format_extensible_sequence(&name, seq.extensible.is_some());

        Ok(sequence_template(
            format_comments(&tld.comments),
            custom_derive.unwrap_or(DERIVE_DEFAULT),
            flatten_nested_sequence_members(&seq.members, &name)?.join("\n"),
            name,
            format_member_declaration(&members),
            extension_decl,
            format_decode_member_body(&members),
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

pub fn generate_sequence_of<'a>(
    tld: ToplevelTypeDeclaration,
    custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Type::SequenceOf(ref seq_of) = tld.r#type {
        let name = rustify_name(&tld.name);
        let anonymous_item = match seq_of.r#type.as_ref() {
            ASN1Type::ElsewhereDeclaredType(_) => None,
            n => Some(generate(
                ToplevelDeclaration::Type(ToplevelTypeDeclaration {
                    parameterization: None,
                    comments: " Anonymous SEQUENCE OF member ".into(),
                    name: String::from("Anonymous_") + &name,
                    r#type: n.clone(),
                }),
                None,
            )?),
        }
        .unwrap_or(String::new());
        let member_type = match seq_of.r#type.as_ref() {
            ASN1Type::ElsewhereDeclaredType(d) => rustify_name(&d.identifier),
            _ => String::from("Anonymous_") + &name,
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

pub fn generate_information_object_set<'a>(
    tld: ToplevelInformationDeclaration,
) -> Result<String, GeneratorError> {
    if let ASN1Information::ObjectSet(o) = &tld.value {
        let class: &InformationObjectClass = match tld.class {
            Some(ClassLink::ByReference(ref c)) => c,
            _ => {
                return Err(GeneratorError::new(
                    None,
                    "Missing class link in Information Object Set",
                    GeneratorErrorType::MissingClassLink,
                ))
            }
        };
        let keys_to_types = o
            .values
            .iter()
            .map(|v| {
                match v {
                    ObjectSetValue::Reference(_) => todo!(),
                    // basically, an information object specifies a sequence implementing a class. So we sould treat information objects like sequences
                    ObjectSetValue::Inline(InformationObjectFields::CustomSyntax(s)) => {
                        resolve_syntax(class, s)
                    }
                    ObjectSetValue::Inline(InformationObjectFields::DefaultSyntax(s)) => {
                        todo!()
                    }
                }
            })
            .collect::<Result<Vec<(ASN1Value, Vec<ASN1Type>)>, GeneratorError>>()?;
        let mut options = keys_to_types
            .iter()
            .map(|(k, types)| {
                format!(
                    "_{}({})",
                    k.to_string(),
                    types
                        .iter()
                        .map(|t| format!("pub {}", t.to_string()))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            })
            .collect::<Vec<String>>()
            .join(",\n\t");
        if o.extensible.is_some() {
            options.push_str(",\n\tUnknownClassImplementation(pub Vec<u8>)");
        }
        let key_type = match class
            .fields
            .iter()
            .find_map(|f| {
                f.is_unique
                    .then(|| f.r#type.as_ref().map(|t| t.to_string()))
            })
            .flatten()
        {
            Some(key_type) => key_type,
            None => {
                return Err(GeneratorError::new(
                    None,
                    "Could not determine class key type!",
                    GeneratorErrorType::MissingClassKey,
                ))
            }
        };
        let mut branches = keys_to_types
            .iter()
            .map(|(k, _)| format!("{} => todo!()", k.to_string(),))
            .collect::<Vec<String>>()
            .join(",\n\t");
        if o.extensible.is_some() {
            branches.push_str(",\n\t_ => todo!()");
        }
        Ok(information_object_set_template(
            format_comments(&tld.comments),
            rustify_name(&tld.name),
            options,
            key_type,
            branches,
        ))
    } else {
        Err(GeneratorError::new(
            Some(ToplevelDeclaration::Information(tld)),
            "Expected Object Set top-level declaration",
            GeneratorErrorType::Asn1TypeMismatch,
        ))
    }
}

#[cfg(test)]
mod tests {
    use asnr_grammar::{subtyping::*, types::*, *};

    use crate::generator::builder::{
        generate_bit_string, generate_enumerated, generate_integer, generate_sequence,
    };

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
        };
        println!("{}", generate_enumerated(enum_tld, None).unwrap())
    }

    #[test]
    fn generates_bitstring_from_template() {
        let bs_tld = ToplevelTypeDeclaration {
            parameterization: None,
            name: "BitString".into(),
            comments: "".into(),
            r#type: ASN1Type::BitString(BitString {
                constraints: vec![ValueConstraint {
                    max_value: Some(ASN1Value::Integer(8)),
                    min_value: Some(ASN1Value::Integer(8)),
                    extensible: true,
                }],
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
        let int_tld = ToplevelTypeDeclaration {
            parameterization: None,
            name: "TestInt".into(),
            comments: "".into(),
            r#type: ASN1Type::Integer(Integer {
                constraints: vec![ValueConstraint {
                    max_value: Some(ASN1Value::Integer(1)),
                    min_value: Some(ASN1Value::Integer(8)),
                    extensible: false,
                }],
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
        let seq_tld = ToplevelTypeDeclaration {
            parameterization: None,
            name: "Sequence".into(),
            comments: "".into(),
            r#type: ASN1Type::Sequence(Sequence {
                constraints: vec![],
                extensible: Some(1),
                members: vec![SequenceMember {
                    name: "nested".into(),
                    tag: None,
                    r#type: ASN1Type::Sequence(Sequence {
                        extensible: Some(3),
                        constraints: vec![],
                        members: vec![
                            SequenceMember {
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
                            SequenceMember {
                                name: "this-is-annoying".into(),
                                tag: None,
                                r#type: ASN1Type::Boolean,
                                default_value: Some(ASN1Value::Boolean(true)),
                                is_optional: true,
                                constraints: vec![],
                            },
                            SequenceMember {
                                name: "another".into(),
                                tag: None,
                                r#type: ASN1Type::Sequence(Sequence {
                                    extensible: None,
                                    constraints: vec![],
                                    members: vec![SequenceMember {
                                        name: "inner".into(),

                                        tag: None,
                                        r#type: ASN1Type::BitString(BitString {
                                            constraints: vec![ValueConstraint {
                                                min_value: Some(ASN1Value::Integer(1)),
                                                max_value: Some(ASN1Value::Integer(1)),
                                                extensible: true,
                                            }],
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
        };
        println!("{}", generate_sequence(seq_tld, None).unwrap())
    }
}

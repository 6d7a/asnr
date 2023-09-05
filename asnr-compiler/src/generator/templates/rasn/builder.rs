use asnr_grammar::{
    utils::int_type_token, ASN1Type, ASN1Value, ToplevelDeclaration, ToplevelTypeDeclaration,
    ToplevelValueDeclaration, INTEGER,
};

use crate::generator::{
    error::{GeneratorError, GeneratorErrorType},
    templates::asnr::util::{format_comments, rustify_name},
};

use super::{
    template::{
        bit_string_template, boolean_template, char_string_template, enumerated_template,
        integer_template, integer_value_template, null_template, null_value_template,
        sequence_or_set_template,
    },
    utils::{
        format_alphabet_annotations, format_enum_members, format_nested_sequence_members,
        format_range_annotations, format_sequence_or_set_members, format_tag,
    },
};

pub struct RasnGenerator;

impl RasnGenerator {
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
        _custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError> {
        if let ASN1Type::Integer(ref int) = tld.r#type {
            Ok(integer_template(
                format_comments(&tld.comments),
                rustify_name(&tld.name),
                format_range_annotations(true, &int.constraints)?,
                format_tag(tld.tag.as_ref()),
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
        _custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError> {
        if let ASN1Type::BitString(ref bitstr) = tld.r#type {
            Ok(bit_string_template(
                format_comments(&tld.comments),
                rustify_name(&tld.name),
                format_range_annotations(true, &bitstr.constraints)?,
                format_tag(tld.tag.as_ref()),
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
        _custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError> {
        if let ASN1Type::CharacterString(ref char_str) = tld.r#type {
            Ok(char_string_template(
                format_comments(&tld.comments),
                rustify_name(&tld.name),
                format_range_annotations(true, &char_str.constraints)?,
                format_alphabet_annotations(char_str.r#type, &char_str.constraints)?,
                format_tag(tld.tag.as_ref()),
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
        _custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError> {
        if let ASN1Type::Boolean = tld.r#type {
            Ok(boolean_template(
                format_comments(&tld.comments),
                rustify_name(&tld.name),
                format_tag(tld.tag.as_ref()),
            ))
        } else {
            Err(GeneratorError::new(
                Some(ToplevelDeclaration::Type(tld)),
                "Expected BOOLEAN top-level declaration",
                GeneratorErrorType::Asn1TypeMismatch,
            ))
        }
    }

    //     fn generate_typealias<'a>(
    //         tld: ToplevelTypeDeclaration,
    //         custom_derive: Option<&'a str>,
    //     ) -> Result<String, GeneratorError> {
    //         if let ASN1Type::ElsewhereDeclaredType(dec) = &tld.r#type {
    //             Ok(typealias_template(
    //                 format_comments(&tld.comments),
    //                 custom_derive.unwrap_or(DERIVE_DEFAULT),
    //                 rustify_name(&tld.name),
    //                 rustify_name(&dec.identifier),
    //                 tld.r#type.declare(),
    //             ))
    //         } else {
    //             Err(GeneratorError::new(
    //                 Some(ToplevelDeclaration::Type(tld)),
    //                 "Expected type alias top-level declaration",
    //                 GeneratorErrorType::Asn1TypeMismatch,
    //             ))
    //         }
    //     }

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
        _custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError> {
        if let ASN1Type::Null = tld.r#type {
            Ok(null_template(
                format_comments(&tld.comments),
                rustify_name(&tld.name),
                format_tag(tld.tag.as_ref()),
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
        _custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError> {
        if let ASN1Type::Enumerated(ref enumerated) = tld.r#type {
            let extensible = if enumerated.extensible.is_some() {
                r#"
                #[non_exhaustive]"#
            } else {
                ""
            };
            Ok(enumerated_template(
                format_comments(&tld.comments),
                rustify_name(&tld.name),
                extensible,
                format_enum_members(enumerated),
                format_tag(tld.tag.as_ref()),
            ))
        } else {
            Err(GeneratorError::new(
                Some(ToplevelDeclaration::Type(tld)),
                "Expected ENUMERATED top-level declaration",
                GeneratorErrorType::Asn1TypeMismatch,
            ))
        }
    }

    //     fn generate_choice<'a>(
    //         tld: ToplevelTypeDeclaration,
    //         custom_derive: Option<&'a str>,
    //     ) -> Result<String, GeneratorError> {
    //         if let ASN1Type::Choice(ref choice) = tld.r#type {
    //             let name = rustify_name(&tld.name);
    //             let inner_options = flatten_nested_choice_options(&choice.options, &name).join("\n");
    //             let options = extract_choice_options(&choice.options, &name);
    //             let mut options_declaration = format_option_declaration(&options);
    //             if choice.extensible.is_some() {
    //                 options_declaration.push_str("\n\tUnknownChoiceValue(Vec<u8>)");
    //             }
    //             let unknown_index_case = if choice.extensible.is_some() {
    //                 r#"_ => Ok(|input| D::decode_unknown_extension(input).map(|(r, v)|(r, Self::UnknownChoiceValue(v))))"#.to_owned()
    //             } else {
    //                 format!(
    //                     r#"x => Err(
    //   DecodingError::new(
    //     &format!("Invalid choice index decoding {name}. Received {{x}}"),
    //     DecodingErrorType::InvalidChoiceIndex
    //   )
    // )"#
    //                 )
    //             };
    //             let default_option = match options.first() {
    //                 Some(o) => default_choice(o),
    //                 None => {
    //                     return Err(GeneratorError {
    //                         top_level_declaration: Some(ToplevelDeclaration::Type(tld)),
    //                         details: "Empty CHOICE types are not yet supported!".into(),
    //                         kind: GeneratorErrorType::EmptyChoiceType,
    //                     })
    //                 }
    //             };
    //             let options_from_int: String = options
    //                 .iter()
    //                 .enumerate()
    //                 .map(format_option_from_int)
    //                 .collect::<Vec<String>>()
    //                 .join("\n\t\t  ");
    //             Ok(choice_template(
    //                 format_comments(&tld.comments),
    //                 custom_derive.unwrap_or("#[derive(Debug, Clone, PartialEq)]"),
    //                 name,
    //                 inner_options,
    //                 default_option,
    //                 options_declaration,
    //                 options_from_int,
    //                 unknown_index_case,
    //                 choice.declare(),
    //             ))
    //         } else {
    //             Err(GeneratorError::new(
    //                 Some(ToplevelDeclaration::Type(tld)),
    //                 "Expected CHOICE top-level declaration",
    //                 GeneratorErrorType::Asn1TypeMismatch,
    //             ))
    //         }
    //     }

    //     fn generate_information_object_class<'a>(
    //         tld: ToplevelInformationDeclaration,
    //     ) -> Result<String, GeneratorError> {
    //         if let ASN1Information::ObjectClass(ref ioc) = tld.value {
    //             Ok(information_object_class_template(
    //                 format_comments(&tld.comments),
    //                 rustify_name(&tld.name),
    //                 ioc.declare(),
    //             ))
    //         } else {
    //             Err(GeneratorError::new(
    //                 Some(ToplevelDeclaration::Information(tld)),
    //                 "Expected CLASS top-level declaration",
    //                 GeneratorErrorType::Asn1TypeMismatch,
    //             ))
    //         }
    //     }

    pub fn generate_sequence_or_set<'a>(
        tld: ToplevelTypeDeclaration,
        _custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError> {
        match tld.r#type {
            ASN1Type::Sequence(ref seq) | ASN1Type::Set(ref seq) => {
                let name = rustify_name(&tld.name);
                let extensible = if seq.extensible.is_some() {
                    r#"
                #[non_exhaustive]"#
                } else {
                    ""
                };
                let set_annotation = if matches!(tld.r#type, ASN1Type::Set(_)) {
                    "set"
                } else {
                    ""
                };
                Ok(sequence_or_set_template(
                    format_comments(&tld.comments),
                    name.clone(),
                    extensible,
                    format_sequence_or_set_members(seq, &name)?,
                    format_nested_sequence_members(seq, &name)?,
                    format_tag(tld.tag.as_ref()),
                    set_annotation.into(),
                ))
            }
            _ => Err(GeneratorError::new(
                Some(ToplevelDeclaration::Type(tld)),
                "Expected SEQUENCE top-level declaration",
                GeneratorErrorType::Asn1TypeMismatch,
            )),
        }
    }

    //     fn generate_sequence_of<'a>(
    //         tld: ToplevelTypeDeclaration,
    //         custom_derive: Option<&'a str>,
    //     ) -> Result<String, GeneratorError> {
    //         if let ASN1Type::SequenceOf(ref seq_of) = tld.r#type {
    //             let name = rustify_name(&tld.name);
    //             let anonymous_item = match seq_of.r#type.as_ref() {
    //                 ASN1Type::ElsewhereDeclaredType(_) => None,
    //                 n => Some(generate(
    //                     &Framework::Asnr,
    //                     ToplevelDeclaration::Type(ToplevelTypeDeclaration {
    //                         parameterization: None,
    //                         comments: " Anonymous SEQUENCE OF member ".into(),
    //                         name: String::from("Anonymous_") + &name,
    //                         r#type: n.clone(),
    //                     }),
    //                     None,
    //                 )?),
    //             }
    //             .unwrap_or(String::new());
    //             let member_type = match seq_of.r#type.as_ref() {
    //                 ASN1Type::ElsewhereDeclaredType(d) => rustify_name(&d.identifier),
    //                 _ => String::from("Anonymous_") + &name,
    //             };
    //             Ok(sequence_of_template(
    //                 format_comments(&tld.comments),
    //                 custom_derive.unwrap_or(DERIVE_DEFAULT),
    //                 name,
    //                 anonymous_item,
    //                 member_type,
    //                 seq_of.declare(),
    //             ))
    //         } else {
    //             Err(GeneratorError::new(
    //                 Some(ToplevelDeclaration::Type(tld)),
    //                 "Expected SEQUENCE OF top-level declaration",
    //                 GeneratorErrorType::Asn1TypeMismatch,
    //             ))
    //         }
    //     }

    //     fn generate_information_object_set<'a>(
    //         tld: ToplevelInformationDeclaration,
    //     ) -> Result<String, GeneratorError> {
    //         if let ASN1Information::ObjectSet(o) = &tld.value {
    //             let class: &InformationObjectClass = match tld.class {
    //                 Some(ClassLink::ByReference(ref c)) => c,
    //                 _ => {
    //                     return Err(GeneratorError::new(
    //                         None,
    //                         "Missing class link in Information Object Set",
    //                         GeneratorErrorType::MissingClassLink,
    //                     ))
    //                 }
    //             };
    //             let keys_to_types = o
    //                 .values
    //                 .iter()
    //                 .map(|v| {
    //                     match v {
    //                         ObjectSetValue::Reference(_) => todo!(),
    //                         // basically, an information object specifies a sequence implementing a class. So we sould treat information objects like sequences
    //                         ObjectSetValue::Inline(InformationObjectFields::CustomSyntax(s)) => {
    //                             resolve_syntax(class, s)
    //                         }
    //                         ObjectSetValue::Inline(InformationObjectFields::DefaultSyntax(_s)) => {
    //                             todo!()
    //                         }
    //                     }
    //                 })
    //                 .collect::<Result<Vec<(ASN1Value, Vec<ASN1Type>)>, GeneratorError>>()?;
    //             let mut options = keys_to_types
    //                 .iter()
    //                 .map(|(k, types)| {
    //                     format!(
    //                         "_{}({})",
    //                         k.to_string(),
    //                         types
    //                             .iter()
    //                             .map(|t| format!("pub {}", t.to_string()))
    //                             .collect::<Vec<String>>()
    //                             .join(", ")
    //                     )
    //                 })
    //                 .collect::<Vec<String>>()
    //                 .join(",\n\t");
    //             if o.extensible.is_some() {
    //                 options.push_str(",\n\tUnknownClassImplementation(pub Vec<u8>)");
    //             }
    //             let key_type = match class
    //                 .fields
    //                 .iter()
    //                 .find_map(|f| {
    //                     f.is_unique
    //                         .then(|| f.r#type.as_ref().map(|t| t.to_string()))
    //                 })
    //                 .flatten()
    //             {
    //                 Some(key_type) => key_type,
    //                 None => {
    //                     return Err(GeneratorError::new(
    //                         None,
    //                         "Could not determine class key type!",
    //                         GeneratorErrorType::MissingClassKey,
    //                     ))
    //                 }
    //             };
    //             let mut branches = keys_to_types
    //                 .iter()
    //                 .map(|(k, _)| format!("{} => todo!()", k.to_string(),))
    //                 .collect::<Vec<String>>()
    //                 .join(",\n\t");
    //             if o.extensible.is_some() {
    //                 branches.push_str(",\n\t_ => todo!()");
    //             }
    //             Ok(information_object_set_template(
    //                 format_comments(&tld.comments),
    //                 rustify_name(&tld.name),
    //                 options,
    //                 key_type,
    //                 branches,
    //             ))
    //         } else {
    //             Err(GeneratorError::new(
    //                 Some(ToplevelDeclaration::Information(tld)),
    //                 "Expected Object Set top-level declaration",
    //                 GeneratorErrorType::Asn1TypeMismatch,
    //             ))
    //         }
    //     }
}

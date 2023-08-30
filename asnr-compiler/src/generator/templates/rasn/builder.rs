use asnr_grammar::{ToplevelValueDeclaration, ASN1Value, INTEGER, utils::int_type_token, ToplevelDeclaration, ToplevelTypeDeclaration, ASN1Type};

use crate::generator::{error::{GeneratorError, GeneratorErrorType}, util::{format_comments, rustify_name}}};

use super::template::integer_template;



pub struct StringifiedNameType {
    pub name: String,
    pub r#type: String,
}

pub struct RasnGenerator;

impl /*Generator for */RasnGenerator {
    fn generate_integer_value(tld: ToplevelValueDeclaration) -> Result<String, GeneratorError> {
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

    fn generate_integer<'a>(
        tld: ToplevelTypeDeclaration,
        custom_derive: Option<&'a str>,
    ) -> Result<String, GeneratorError> {
        if let ASN1Type::Integer(ref int) = tld.r#type {
            Ok(integer_template(
                format_comments(&tld.comments),
                rustify_name(&tld.name),
                None
            ))
        } else {
            Err(GeneratorError::new(
                Some(ToplevelDeclaration::Type(tld)),
                "Expected INTEGER top-level declaration",
                GeneratorErrorType::Asn1TypeMismatch,
            ))
        }
    }

//     fn generate_bit_string<'a>(
//         tld: ToplevelTypeDeclaration,
//         custom_derive: Option<&'a str>,
//     ) -> Result<String, GeneratorError> {
//         if let ASN1Type::BitString(ref bitstr) = tld.r#type {
//             Ok(bit_string_template(
//                 format_comments(&tld.comments),
//                 custom_derive.unwrap_or(DERIVE_DEFAULT),
//                 rustify_name(&tld.name),
//                 format_distinguished_values(&tld),
//                 bitstr.declare(),
//             ))
//         } else {
//             Err(GeneratorError::new(
//                 Some(ToplevelDeclaration::Type(tld)),
//                 "Expected BIT STRING top-level declaration",
//                 GeneratorErrorType::Asn1TypeMismatch,
//             ))
//         }
//     }

//     fn character_string_template<'a>(
//         tld: ToplevelTypeDeclaration,
//         custom_derive: Option<&'a str>,
//     ) -> Result<String, GeneratorError> {
//         if let ASN1Type::CharacterString(ref char_str) = tld.r#type {
//             Ok(char_string_template(
//                 format_comments(&tld.comments),
//                 custom_derive.unwrap_or(DERIVE_DEFAULT),
//                 rustify_name(&tld.name),
//                 char_str.declare(),
//             ))
//         } else {
//             Err(GeneratorError::new(
//                 Some(ToplevelDeclaration::Type(tld)),
//                 "Expected Character String top-level declaration",
//                 GeneratorErrorType::Asn1TypeMismatch,
//             ))
//         }
//     }

//     fn generate_boolean<'a>(
//         tld: ToplevelTypeDeclaration,
//         custom_derive: Option<&'a str>,
//     ) -> Result<String, GeneratorError> {
//         if let ASN1Type::Boolean = tld.r#type {
//             Ok(boolean_template(
//                 format_comments(&tld.comments),
//                 custom_derive.unwrap_or(DERIVE_DEFAULT),
//                 rustify_name(&tld.name),
//             ))
//         } else {
//             Err(GeneratorError::new(
//                 Some(ToplevelDeclaration::Type(tld)),
//                 "Expected BOOLEAN top-level declaration",
//                 GeneratorErrorType::Asn1TypeMismatch,
//             ))
//         }
//     }

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

//     fn generate_null_value(tld: ToplevelValueDeclaration) -> Result<String, GeneratorError> {
//         if let ASN1Value::Null = tld.value {
//             Ok(null_value_template(
//                 format_comments(&tld.comments),
//                 rustify_name(&tld.name),
//             ))
//         } else {
//             Err(GeneratorError::new(
//                 Some(ToplevelDeclaration::Value(tld)),
//                 "Expected NULL value top-level declaration",
//                 GeneratorErrorType::Asn1TypeMismatch,
//             ))
//         }
//     }

//     fn generate_null<'a>(
//         tld: ToplevelTypeDeclaration,
//         custom_derive: Option<&'a str>,
//     ) -> Result<String, GeneratorError> {
//         if let ASN1Type::Null = tld.r#type {
//             Ok(null_template(
//                 format_comments(&tld.comments),
//                 custom_derive.unwrap_or(DERIVE_DEFAULT),
//                 rustify_name(&tld.name),
//             ))
//         } else {
//             Err(GeneratorError::new(
//                 Some(ToplevelDeclaration::Type(tld)),
//                 "Expected NULL top-level declaration",
//                 GeneratorErrorType::Asn1TypeMismatch,
//             ))
//         }
//     }

//     fn generate_enumerated<'a>(
//         mut tld: ToplevelTypeDeclaration,
//         custom_derive: Option<&'a str>,
//     ) -> Result<String, GeneratorError> {
//         if let ASN1Type::Enumerated(ref mut enumerated) = tld.r#type {
//             enumerated.members.sort_by(|a, b| a.index.cmp(&b.index));
//             let name = rustify_name(&tld.name);
//             let mut enumerals = enumerated
//                 .members
//                 .iter()
//                 .map(format_enumeral)
//                 .collect::<Vec<String>>()
//                 .join("\n\t");
//             if enumerated.extensible.is_some() {
//                 enumerals.push_str("\n\tUnknownExtension")
//             }
//             let unknown_index_case = if enumerated.extensible.is_some() {
//                 format!("Ok(Self::UnknownExtension)")
//             } else {
//                 format!(
//                     r#"Err(
//               DecodingError {{
//                 details: format!("Invalid enumerated index decoding {name}. Received index {{}}",v), 
//                 kind: DecodingErrorType::InvalidEnumeratedIndex,
//                 input: None
//               }}
//             )"#
//                 )
//             };
//             let enumerals_from_int = enumerated
//                 .members
//                 .iter()
//                 .map(format_enumeral_from_int)
//                 .collect::<Vec<String>>()
//                 .join("\n\t\t  ");
//             Ok(enumerated_template(
//                 format_comments(&tld.comments),
//                 custom_derive.unwrap_or(DERIVE_DEFAULT),
//                 name,
//                 enumerals,
//                 enumerals_from_int,
//                 unknown_index_case,
//                 enumerated.declare(),
//             ))
//         } else {
//             Err(GeneratorError::new(
//                 Some(ToplevelDeclaration::Type(tld)),
//                 "Expected ENUMERATED top-level declaration",
//                 GeneratorErrorType::Asn1TypeMismatch,
//             ))
//         }
//     }

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

//     fn generate_sequence<'a>(
//         tld: ToplevelTypeDeclaration,
//         custom_derive: Option<&'a str>,
//     ) -> Result<String, GeneratorError> {
//         if let ASN1Type::Sequence(ref seq) = tld.r#type {
//             let name = rustify_name(&tld.name);
//             let members = extract_sequence_members(&seq.members, &name);
//             let (extension_decl, extension_decoder) =
//                 format_extensible_sequence(&name, seq.extensible.is_some());

//             Ok(sequence_template(
//                 format_comments(&tld.comments),
//                 custom_derive.unwrap_or(DERIVE_DEFAULT),
//                 flatten_nested_sequence_members(&seq.members, &name)?.join("\n"),
//                 name,
//                 format_member_declaration(&members),
//                 extension_decl,
//                 format_decode_member_body(&members),
//                 extension_decoder,
//                 seq.declare(),
//             ))
//         } else {
//             Err(GeneratorError::new(
//                 Some(ToplevelDeclaration::Type(tld)),
//                 "Expected SEQUENCE top-level declaration",
//                 GeneratorErrorType::Asn1TypeMismatch,
//             ))
//         }
//     }

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

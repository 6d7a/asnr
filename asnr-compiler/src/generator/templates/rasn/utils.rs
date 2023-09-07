use asnr_grammar::{
    constraints::Constraint,
    encoding_rules::per_visible::{
        per_visible_range_constraints, CharsetSubset, PerVisibleAlphabetConstraints,
    },
    types::{ ChoiceOption, Enumerated, SequenceOrSet, SequenceOrSetMember, Choice},
    utils::*,
    ASN1Type, AsnTag, CharacterStringType, TagClass, ToplevelDeclaration, ToplevelTypeDeclaration,
};

use crate::generator::{error::GeneratorError, generate, templates::inner_name, Framework};

pub fn format_range_annotations(
    signed: bool,
    constraints: &Vec<Constraint>,
) -> Result<String, GeneratorError> {
    if constraints.is_empty() {
        return Ok(String::new());
    }
    let per_constraints = per_visible_range_constraints(signed, constraints)?;
    let range_prefix = if per_constraints.is_size_constraint() {
        "size"
    } else {
        "value"
    };
    // handle default size constraints
    if per_constraints.is_size_constraint()
        && !per_constraints.is_extensible()
        && per_constraints.min::<i128>() == Some(0)
        && per_constraints.max::<i128>().is_none()
    {
        return Ok(String::new());
    }
    Ok(
        match (
            per_constraints.min::<i128>(),
            per_constraints.max::<i128>(),
            per_constraints.is_extensible(),
        ) {
            (Some(min), Some(max), true) => {
                format!(r#"{range_prefix}("{min}..={max}", extensible)"#)
            }
            (Some(min), Some(max), false) => {
                format!(r#"{range_prefix}("{min}..={max}")"#)
            }
            (Some(min), None, true) => {
                format!(r#"{range_prefix}("{min}..", extensible)"#)
            }
            (Some(min), None, false) => format!(r#"{range_prefix}("{min}..")"#),
            (None, Some(max), true) => {
                format!(r#"{range_prefix}("..={max}", extensible)"#)
            }
            (None, Some(max), false) => format!(r#"{range_prefix}("..={max}")"#),
            (None, None, true) => format!(r#"{range_prefix}("..", extensible)"#),
            (None, None, false) => format!(r#""#),
        },
    )
}

pub fn format_alphabet_annotations(
    string_type: CharacterStringType,
    constraints: &Vec<Constraint>,
) -> Result<String, GeneratorError> {
    if constraints.is_empty() {
        return Ok(String::new());
    }
    let mut permitted_alphabet = PerVisibleAlphabetConstraints::default_for(string_type);
    for c in constraints {
        PerVisibleAlphabetConstraints::try_new(c, string_type)?
            .map(|mut p| permitted_alphabet += &mut p);
    }
    permitted_alphabet.finalize();
    let alphabet_unicode: Vec<String> = permitted_alphabet
        .charset_subsets()
        .iter()
        .map(|subset| match subset {
            CharsetSubset::Single(c) => format!("{}", c.escape_unicode()),
            CharsetSubset::Range { from, to } => format!(
                "{}..{}",
                from.map_or(String::from(""), |c| format!("{}", c.escape_unicode())),
                to.map_or(String::from(""), |c| format!("{}", c.escape_unicode()))
            ),
        })
        .collect();
    Ok(if alphabet_unicode.is_empty() {
        "".into()
    } else {
        String::from(", from(") + &alphabet_unicode.join(", ") + ")"
    })
}

pub fn format_enum_members(enumerated: &Enumerated) -> String {
    let first_extension_index = enumerated.extensible;
    enumerated
        .members
        .iter()
        .map(|e| {
            let name = to_rust_title_case(&e.name);
            let index = e.index;
            let extension = if index >= first_extension_index.map_or(i128::MAX, |x| x as i128) {
                r#"#[rasn(extension_addition)]
            "#
            } else {
                ""
            };
            format!(r#"{extension}{name} = {index}"#)
        })
        .collect::<Vec<String>>()
        .join(
            r#",
    "#,
        )
}

pub fn format_tag(tag: Option<&AsnTag>) -> String {
    if let Some(tag) = tag {
        let class = match tag.tag_class {
            TagClass::Universal => "universal, ",
            TagClass::Application => "application, ",
            TagClass::Private => "private, ",
            TagClass::ContextSpecific => "context, ",
        };
        let id = tag.id;
        format!("tag({class}{id})")
    } else {
        String::from("")
    }
}

pub fn format_sequence_or_set_members(
    sequence_or_set: &SequenceOrSet,
    parent_name: &String,
) -> Result<String, GeneratorError> {
    let first_extension_index = sequence_or_set.extensible;
    Ok(sequence_or_set
        .members
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let extension_annotation = if i >= first_extension_index.unwrap_or(usize::MAX)
                && m.name.starts_with("ext_group_")
            {
                "extension_addition_group"
            } else if i >= first_extension_index.unwrap_or(usize::MAX) {
                "extension_addition"
            } else {
                ""
            };
            format_sequence_member(m, parent_name, extension_annotation)
        })
        .collect::<Result<Vec<String>, _>>()?
        .join(
            r#",
    "#,
        ))
}

fn format_sequence_member(
    member: &SequenceOrSetMember,
    parent_name: &String,
    extension_annotation: &str,
) -> Result<String, GeneratorError> {
    let name = to_rust_camel_case(&member.name);
    let (mut all_constraints, mut formatted_type_name) = match &member.r#type {
        ASN1Type::Null => (vec![], "()".into()),
        ASN1Type::Boolean => (vec![], "bool".into()),
        ASN1Type::Integer(i) => {
            let per_constraints = per_visible_range_constraints(true, &i.constraints)?;
            (
                i.constraints.clone(),
                int_type_token(
                    per_constraints.min().unwrap_or(i128::MIN),
                    per_constraints.max().unwrap_or(i128::MAX),
                )
                .into(),
            )
        }
        ASN1Type::Real(_) => (vec![], "f64".into()),
        ASN1Type::BitString(b) => (b.constraints.clone(), "BitString".into()),
        ASN1Type::OctetString(o) => (o.constraints.clone(), "OctetString".into()),
        ASN1Type::CharacterString(c) => (c.constraints.clone(), string_type(&c.r#type)),
        ASN1Type::Enumerated(_)
        | ASN1Type::Choice(_)
        | ASN1Type::Sequence(_)
        | ASN1Type::SequenceOf(_)
        | ASN1Type::Set(_) => (vec![], inner_name(&member.name, parent_name)),
        ASN1Type::ElsewhereDeclaredType(e) => {
            (e.constraints.clone(), to_rust_title_case(&e.identifier))
        }
        ASN1Type::InformationObjectFieldReference(_) => (vec![], "Any".into()),
    };
    all_constraints.append(&mut member.constraints.clone());
    if member.is_optional && member.default_value.is_none() {
        formatted_type_name = String::from("Option<") + &formatted_type_name + ">";
    }
    let default_annotation = if member.default_value.is_some() {
        format!(
            r#"default = "{}""#,
            default_method_name(parent_name, &member.name)
        )
    } else {
        String::new()
    };
    let range_annotations = format_range_annotations(
        matches!(member.r#type, ASN1Type::Integer(_)),
        &all_constraints,
    )?;
    let alphabet_annotations = if let ASN1Type::CharacterString(c_string) = &member.r#type {
        format_alphabet_annotations(c_string.r#type, &all_constraints)?
    } else {
        "".into()
    };
    let tag = format_tag(member.tag.as_ref());
    let annotations = join_annotations(vec![
        extension_annotation.to_string(),
        range_annotations,
        alphabet_annotations,
        tag,
        default_annotation,
    ]);
    Ok(format!(r#"{annotations}{name}: {formatted_type_name}"#))
}

pub fn format_choice_options(
    choice: &Choice,
    parent_name: &String,
) -> Result<String, GeneratorError> {
    let first_extension_index = choice.extensible;
    Ok(choice
        .options
        .iter()
        .enumerate()
        .map(|(i, o)| {
            let extension_annotation = if i >= first_extension_index.unwrap_or(usize::MAX)
                && o.name.starts_with("ext_group_")
            {
                "extension_addition_group"
            } else if i >= first_extension_index.unwrap_or(usize::MAX) {
                "extension_addition"
            } else {
                ""
            };
            format_choice_option(o, parent_name, extension_annotation)
        })
        .collect::<Result<Vec<String>, _>>()?
        .join(
            r#",
    "#,
        ))
}

fn format_choice_option(
    member: &ChoiceOption,
    parent_name: &String,
    extension_annotation: &str,
) -> Result<String, GeneratorError> {
    let name = to_rust_title_case(&member.name);
    let (mut all_constraints, mut formatted_type_name) = match &member.r#type {
        ASN1Type::Null => (vec![], "()".into()),
        ASN1Type::Boolean => (vec![], "bool".into()),
        ASN1Type::Integer(i) => {
            let per_constraints = per_visible_range_constraints(true, &i.constraints)?;
            (
                i.constraints.clone(),
                int_type_token(
                    per_constraints.min().unwrap_or(i128::MIN),
                    per_constraints.max().unwrap_or(i128::MAX),
                )
                .into(),
            )
        }
        ASN1Type::Real(_) => (vec![], "f64".into()),
        ASN1Type::BitString(b) => (b.constraints.clone(), "BitString".into()),
        ASN1Type::OctetString(o) => (o.constraints.clone(), "OctetString".into()),
        ASN1Type::CharacterString(c) => (c.constraints.clone(), string_type(&c.r#type)),
        ASN1Type::Enumerated(_)
        | ASN1Type::Choice(_)
        | ASN1Type::Sequence(_)
        | ASN1Type::SequenceOf(_)
        | ASN1Type::Set(_) => (vec![], inner_name(&member.name, parent_name)),
        ASN1Type::ElsewhereDeclaredType(e) => {
            (e.constraints.clone(), to_rust_title_case(&e.identifier))
        }
        ASN1Type::InformationObjectFieldReference(_) => (vec![], "Any".into()),
    };
    all_constraints.append(&mut member.constraints.clone());
    let range_annotations = format_range_annotations(
        matches!(member.r#type, ASN1Type::Integer(_)),
        &all_constraints,
    )?;
    let alphabet_annotations = if let ASN1Type::CharacterString(c_string) = &member.r#type {
        format_alphabet_annotations(c_string.r#type, &all_constraints)?
    } else {
        "".into()
    };
    let tag = format_tag(member.tag.as_ref());
    let annotations = join_annotations(vec![
        extension_annotation.to_string(),
        range_annotations,
        alphabet_annotations,
        tag,
    ]);
    Ok(format!(r#"{annotations}{name}({formatted_type_name})"#))
}

pub fn string_type(c_type: &CharacterStringType) -> String {
    match c_type {
        CharacterStringType::NumericString => "NumericString".into(),
        CharacterStringType::VisibleString => "VisibleString".into(),
        CharacterStringType::IA5String => "Ia5String".into(),
        CharacterStringType::TeletexString => "TeletexString".into(),
        CharacterStringType::VideotexString => todo!(),
        CharacterStringType::GraphicString => todo!(),
        CharacterStringType::GeneralString => "GeneralString".into(),
        CharacterStringType::UniversalString => todo!(),
        CharacterStringType::UTF8String => "Utf8String".into(),
        CharacterStringType::BMPString => "BmpString".into(),
        CharacterStringType::PrintableString => "PrintableString".into(),
    }
}

pub fn join_annotations(strings: Vec<String>) -> String {
    match strings
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>()
        .join(",")
    {
        s if s.is_empty() => String::new(),
        s => {
            String::from("#[rasn(")
                + &s
                + r#")]
        "#
        }
    }
}

pub fn default_method_name(parent_name: &String, field_name: &String) -> String {
    format!(
        "{}_{}_default",
        to_rust_camel_case(parent_name),
        to_rust_camel_case(field_name)
    )
}

pub fn format_default_methods(
    members: &Vec<SequenceOrSetMember>,
    parent_name: &String,
) -> Result<String, GeneratorError> {
    let mut output = String::new();
    for member in members {
        if let Some(value) = member.default_value.as_ref() {
            let (type_as_string, value_as_string) = match &member.r#type {
                ASN1Type::BitString(_) => ("BitString".into(), format!("{}.iter().collect()", value.value_as_string(None)?)),
                ASN1Type::ElsewhereDeclaredType(_) => {
                    let stringified_type = member.r#type.to_string();
                    (stringified_type.clone(), format!("{stringified_type}({})", value.value_as_string(None)?))
                }
                ty => (ty.to_string(), value.value_as_string(Some(&to_rust_title_case(&ty.to_string())))?),
            };
            let method_name = default_method_name(parent_name, &member.name);
            output.push_str(&format!(
                r#"fn {method_name}() -> {type_as_string} {{
                {value_as_string}
            }}
            
            "#
            ))
        }
    }
    Ok(output)
}

pub fn format_nested_sequence_members(
    sequence_or_set: &SequenceOrSet,
    parent_name: &String,
) -> Result<String, GeneratorError> {
    Ok(sequence_or_set
        .members
        .iter()
        .filter(|m| {
            matches!(
                m.r#type,
                ASN1Type::Enumerated(_)
                    | ASN1Type::Choice(_)
                    | ASN1Type::Sequence(_)
                    | ASN1Type::SequenceOf(_)
                    | ASN1Type::Set(_)
            )
        })
        .map(|m| {
            generate(
                &Framework::Rasn,
                ToplevelDeclaration::Type(ToplevelTypeDeclaration {
                    parameterization: None,
                    comments: " Inner type ".into(),
                    name: inner_name(&m.name, parent_name),
                    r#type: m.r#type.clone(),
                    tag: None,
                }),
                None,
            )
        })
        .collect::<Result<Vec<String>, _>>()?
        .join(
            r#"
    
    "#,
        ))
}

pub fn format_nested_choice_options(
    choice: &Choice,
    parent_name: &String,
) -> Result<String, GeneratorError> {
    Ok(choice
        .options
        .iter()
        .filter(|m| {
            matches!(
                m.r#type,
                ASN1Type::Enumerated(_)
                    | ASN1Type::Choice(_)
                    | ASN1Type::Sequence(_)
                    | ASN1Type::SequenceOf(_)
                    | ASN1Type::Set(_)
            )
        })
        .map(|m| {
            generate(
                &Framework::Rasn,
                ToplevelDeclaration::Type(ToplevelTypeDeclaration {
                    parameterization: None,
                    comments: " Inner type ".into(),
                    name: inner_name(&m.name, parent_name),
                    r#type: m.r#type.clone(),
                    tag: None,
                }),
                None,
            )
        })
        .collect::<Result<Vec<String>, _>>()?
        .join(
            r#"
    
    "#,
        ))
}

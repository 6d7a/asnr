use asnr_grammar::{
    constraints::Constraint,
    encoding_rules::per_visible::{
        per_visible_range_constraints, CharsetSubset, PerVisibleAlphabetConstraints,
    },
    types::Enumerated,
    CharacterStringType, AsnTag, TagClass,
};

use crate::generator::{error::GeneratorError, templates::asnr::util::rustify_name};

pub fn format_constraint_annotations(
    signed: bool,
    constraints: &Vec<Constraint>,
    is_size_constraint: bool,
) -> Result<String, GeneratorError> {
    let per_constraints = per_visible_range_constraints(signed, constraints)?;
    let range_prefix = if is_size_constraint { "size" } else { "value" };
    Ok(
        match (
            per_constraints.min::<i128>(),
            per_constraints.max::<i128>(),
            per_constraints.is_extensible(),
        ) {
            (Some(min), Some(max), true) => {
                format!(r#", {range_prefix}("{min}..={max}"), extensible"#)
            }
            (Some(min), Some(max), false) => {
                format!(r#", {range_prefix}("{min}..={max}")"#)
            }
            (Some(min), None, true) => {
                format!(r#", {range_prefix}("{min}.."), extensible"#)
            }
            (Some(min), None, false) => format!(r#", {range_prefix}("{min}..")"#),
            (None, Some(max), true) => {
                format!(r#", {range_prefix}("..={max}"), extensible"#)
            }
            (None, Some(max), false) => format!(r#", {range_prefix}("..={max}")"#),
            (None, None, true) => format!(r#", extensible"#),
            (None, None, false) => format!(r#""#),
        },
    )
}

pub fn format_alphabet_annotations(
    string_type: CharacterStringType,
    constraints: &Vec<Constraint>,
) -> Result<String, GeneratorError> {
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
            let name = rustify_name(&e.name);
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
        format!(", tag({class}{id})")
    } else {
        String::from("")
    }
}
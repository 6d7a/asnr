use asnr_grammar::{
    information_object::{
        InformationObjectClass, ObjectFieldIdentifier, SyntaxApplication, SyntaxExpression,
        SyntaxToken,
    },
    types::*,
    *,
    utils::*
};

use crate::{
    generator::{
        error::{GeneratorError, GeneratorErrorType},
        generate, 
    },
    Framework,
};

use super::builder::StringifiedNameType;

/// Resolves the custom syntax declared in an information object class' WITH SYNTAX clause
#[allow(dead_code)]
pub fn resolve_syntax(
    class: &InformationObjectClass,
    application: &Vec<SyntaxApplication>,
) -> Result<(ASN1Value, Vec<ASN1Type>), GeneratorError> {
    let expressions = match &class.syntax {
        Some(s) => &s.expressions,
        None => {
            return Err(GeneratorError {
                top_level_declaration: None,
                details: "No syntax definition for information object class found!".into(),
                kind: GeneratorErrorType::MissingCustomSyntax,
            })
        }
    };

    let tokens = flatten_tokens(&expressions);

    let mut key = None;
    let mut field_index_map = Vec::<(usize, ASN1Type)>::new();

    let mut appl_iter = application.iter();
    'syntax_matching: for (required, token) in tokens {
        if let Some(expr) = appl_iter.next() {
            if compare_tokens(&token, expr) {
                match expr {
                    SyntaxApplication::ObjectSetDeclaration(_) => todo!(),
                    SyntaxApplication::TypeReference(t) => {
                        if let Some(index) = class.fields.iter().enumerate().find_map(|(i, v)| {
                            (v.identifier
                                == ObjectFieldIdentifier::MultipleValue(
                                    token.name_or_empty().to_owned(),
                                ))
                            .then(|| i)
                        }) {
                            field_index_map.push((index, t.clone()))
                        }
                    }
                    SyntaxApplication::ValueReference(v) => {
                        if let Some(_) = class.fields.iter().find(|v| {
                            v.identifier
                                == ObjectFieldIdentifier::SingleValue(
                                    token.name_or_empty().to_owned(),
                                )
                                && v.is_unique
                        }) {
                            key = Some(v.clone())
                        }
                    }
                    _ => continue 'syntax_matching,
                }
            } else if required {
                return Err(GeneratorError {
                    top_level_declaration: None,
                    details: format!("Syntax mismatch while resolving information object."),
                    kind: GeneratorErrorType::SyntaxMismatch,
                });
            } else {
                continue 'syntax_matching;
            }
        } else if required {
            return Err(GeneratorError {
                top_level_declaration: None,
                details: format!("Syntax mismatch while resolving information object."),
                kind: GeneratorErrorType::SyntaxMismatch,
            });
        } else {
            continue 'syntax_matching;
        }
    }
    field_index_map.sort_by(|&(a, _), &(b, _)| a.cmp(&b));
    let types = field_index_map.into_iter().map(|(_, t)| t).collect();
    match key {
        Some(k) => Ok((k, types)),
        None => Err(GeneratorError {
            top_level_declaration: None,
            details: "Could not find class key!".into(),
            kind: GeneratorErrorType::MissingClassKey,
        }),
    }
}

fn flatten_tokens(expressions: &Vec<SyntaxExpression>) -> Vec<(bool, SyntaxToken)> {
    iter_expressions(expressions, false)
        .into_iter()
        .map(|x| match x {
            (is_required, SyntaxExpression::Required(r)) => (is_required, r.clone()),
            _ => unreachable!(),
        })
        .collect()
}

fn iter_expressions(
    expressions: &Vec<SyntaxExpression>,
    optional_recursion: bool,
) -> Vec<(bool, &SyntaxExpression)> {
    expressions
        .iter()
        .flat_map(|x| match x {
            SyntaxExpression::Optional(o) => iter_expressions(o, true),
            r => vec![(!optional_recursion, r)],
        })
        .collect()
}

fn compare_tokens(token: &SyntaxToken, application: &SyntaxApplication) -> bool {
    match token {
        SyntaxToken::Comma => application == &SyntaxApplication::Comma,
        SyntaxToken::Literal(l) => application == &SyntaxApplication::Literal(l.clone()),
        SyntaxToken::Field(ObjectFieldIdentifier::MultipleValue(_m)) => match application {
            SyntaxApplication::ObjectSetDeclaration(_) | SyntaxApplication::TypeReference(_) => {
                true
            }
            _ => false,
        },
        SyntaxToken::Field(ObjectFieldIdentifier::SingleValue(_s)) => {
            if let SyntaxApplication::ValueReference(_) = application {
                true
            } else {
                false
            }
        }
    }
}

pub fn format_comments(comments: &String) -> String {
    if comments.is_empty() {
        String::from("")
    } else {
        String::from("///") + &comments.replace("\n", "\n ///") + "\n"
    }
}

pub fn format_enumeral(enumeral: &Enumeral) -> String {
    (if enumeral.description.is_some() {
        "/// ".to_owned() + enumeral.description.as_ref().unwrap() + "\n\t"
    } else {
        "".to_owned()
    }) + &to_rust_title_case(&enumeral.name)
        + " = "
        + &enumeral.index.to_string()
        + ","
}

/// When formatting identifiers, i.e. type, member, and value names, it can happen
/// that the formatting results in duplicates. Consider for example an ENUMERATED
/// with members `member0-1` and `member01`. These member names would be formatted
/// resulting both in `Member01` and `Member01`. This function resolves the duplicate
/// identifiers by appending an `Bis`, i.e. producing `Member01` and `Member01Bis`
pub fn handle_duplicate_enumerals(input: &mut Vec<Enumeral>) {
    *input = (*input).drain(..).fold(Vec::new(), |mut acc, mut curr| {
        while let Some(_) = acc
            .iter()
            .find(|e| to_rust_title_case(&e.name) == to_rust_title_case(&curr.name))
        {
            curr.name.push_str("Bis");
        }
        acc.push(curr);
        acc
    });
}

/// When formatting identifiers, i.e. type, member, and value names, it can happen
/// that the formatting results in duplicates. Consider for example an CHOICE
/// with members `member0-1` and `member01`. These member names would be formatted
/// resulting both in `Member01` and `Member01`. This function resolves the duplicate
/// identifiers by appending an `Bis`, i.e. producing `Member01` and `Member01Bis`
pub fn handle_duplicate_options(input: &mut Vec<ChoiceOption>) {
    *input = (*input).drain(..).fold(Vec::new(), |mut acc, mut curr| {
        while let Some(_) = acc
            .iter()
            .find(|e| to_rust_title_case(&e.name) == to_rust_title_case(&curr.name))
        {
            curr.name.push_str("Bis");
        }
        acc.push(curr);
        acc
    });
}

pub fn format_option_from_int(args: (usize, &StringifiedNameType)) -> String {
    format!(
        r#"x if x == {index} => Ok(|input| {{
    {t}::decode::<D>(input).map(|(r, v)|(r, Self::{name}(v)))
  }}),"#,
        index = args.0,
        name = args.1.name,
        t = args.1.r#type
    )
}

pub fn format_option_encoder_from_int(args: (usize, &StringifiedNameType)) -> String {
    format!(
        r#"x if x == {index} => Ok(|encodable, output| {{
        if let Self::{name}(inner) = encodable {{
          {t}::encode::<E>(inner.clone(), output)
        }} else {{
          Err(EncodingError {{ details: format!("Index {index} does not correspond to Choice option {name}!") }})
        }}
      }}),"#,
        index = args.0,
        name = args.1.name,
        t = args.1.r#type
    )
}

pub fn format_enumeral_from_int(enumeral: &Enumeral) -> String {
    let name = &to_rust_title_case(&enumeral.name);
    format!("x if x == Self::{name} as i128 => Ok(Self::{name}),")
}

pub fn format_distinguished_values(tld: &ToplevelTypeDeclaration) -> String {
    let name = &to_rust_title_case(&tld.name);
    match &tld.r#type {
        asnr_grammar::ASN1Type::Integer(i) => match &i.distinguished_values {
            Some(d) => {
                let d_vals = d
                    .iter()
                    .map(format_distinguished_int_value)
                    .collect::<Vec<String>>()
                    .join("\n  ");
                format!(
                    r#"

impl {name} {{
  {d_vals}
}}"#
                )
            }
            None => "".into(),
        },
        asnr_grammar::ASN1Type::BitString(b) => match &b.distinguished_values {
            Some(d) => {
                let d_vals = d
                    .iter()
                    .map(format_distinguished_bit_value)
                    .collect::<Vec<String>>()
                    .join("\n  ");
                format!(
                    r#"

impl {name} {{
  {d_vals}
}}"#
                )
            }
            None => "".into(),
        },
        _ => "".into(),
    }
}

pub fn format_distinguished_bit_value(value: &DistinguishedValue) -> String {
    let name = &to_rust_camel_case(&value.name);
    let i = value.value;
    format!("pub fn is_{name}(&self) -> bool {{ *self.0.get({i}).unwrap_or(&false) }}")
}

pub fn format_distinguished_int_value(value: &DistinguishedValue) -> String {
    let name = to_rust_camel_case(&value.name);
    let i = value.value;
    format!("pub fn is_{name}(&self) -> bool {{ self.0 as i128 == {i} }}")
}

pub fn flatten_nested_sequence_members(
    members: &Vec<SequenceOrSetMember>,
    parent_name: &String,
) -> Result<Vec<String>, GeneratorError> {
    members
        .iter()
        .filter_map(|i| match i.r#type {
            ASN1Type::ElsewhereDeclaredType(_) => None,
            ASN1Type::InformationObjectFieldReference(_) => None,
            _ => Some(declare_inner_sequence_member(i, parent_name)),
        })
        .collect::<Result<Vec<String>, GeneratorError>>()
}

pub fn flatten_nested_choice_options(
    options: &Vec<ChoiceOption>,
    parent_name: &String,
) -> Vec<String> {
    options
        .iter()
        .filter(|m| match m.r#type {
            ASN1Type::ElsewhereDeclaredType(_) => false,
            _ => true,
        })
        .map(|i| declare_inner_choice_option(i, parent_name).unwrap())
        .collect::<Vec<String>>()
}

pub fn extract_choice_options(
    options: &Vec<ChoiceOption>,
    parent_name: &String,
) -> Vec<StringifiedNameType> {
    options
        .iter()
        .map(|m| {
            let name = to_rust_title_case(&m.name);
            let rtype = match &m.r#type {
                ASN1Type::ElsewhereDeclaredType(d) => to_rust_title_case(&d.identifier),
                _ => inner_name(&m.name, parent_name),
            };
            StringifiedNameType {
                name,
                r#type: rtype,
            }
        })
        .collect()
}

pub fn format_option_declaration(members: &Vec<StringifiedNameType>) -> String {
    members
        .iter()
        .map(|m| format!("{}({}),", m.name, m.r#type))
        .collect::<Vec<String>>()
        .join("\n  ")
}

pub fn extract_sequence_members(
    members: &Vec<SequenceOrSetMember>,
    parent_name: &String,
    index_of_first_extension: Option<usize>,
) -> Vec<StringifiedNameType> {
    members
        .iter()
        .enumerate()
        .map(|(index, m)| {
            let name = to_rust_camel_case(&m.name);
            let mut rtype = match &m.r#type {
                ASN1Type::ElsewhereDeclaredType(d) => to_rust_title_case(&d.identifier),
                ASN1Type::InformationObjectFieldReference(_) => "Asn1Open".to_string(),
                _ => inner_name(&m.name, parent_name),
            };
            if m.is_optional || index >= index_of_first_extension.unwrap_or(usize::MAX) {
                rtype = String::from("Option<") + &rtype + ">"
            }
            StringifiedNameType {
                name,
                r#type: rtype,
            }
        })
        .collect()
}

pub fn format_member_declaration(members: &Vec<StringifiedNameType>) -> String {
    members
        .iter()
        .map(|m| format!("pub {}: {},", to_rust_camel_case(&m.name), m.r#type))
        .collect::<Vec<String>>()
        .join("\n  ")
}

pub fn format_extensible_sequence<'a>(_name: &String, extensible: bool) -> String {
    if extensible {
        "{ (input, _) = D::decode_unknown_extension(input)? },".into()
    } else {
        format!(
            r#"return Err(
        DecodingError {{
          details: format!("Invalid member index decoding TestSequence. Received index {{}}",index), 
          kind: DecodingErrorType::InvalidEnumeratedIndex, 
          input: None
        }}
      )"#
        )
    }
}

pub fn format_decode_member_body(members: &Vec<StringifiedNameType>) -> String {
    members
        .iter()
        .enumerate()
        .map(|(i, m)| {
            if m.r#type.starts_with("Option<") {
                format!(
                    "{i} => {{ (input, self.{name}) = {t}::decode::<D>(input).map(|(i, v)| (i, Some(v)))? }},",
                    t = &m.r#type[7..m.r#type.len() - 1],
                    name = to_rust_camel_case(&m.name)
                )
            } else {
                format!(
                    "{i} => {{ (input, self.{name}) = {t}::decode::<D>(input)? }},",
                    t = m.r#type,
                    name = to_rust_camel_case(&m.name)
                )
            }
        })
        .collect::<Vec<String>>()
        .join("\n      ")
}

pub fn format_encoder_member_body(members: &Vec<StringifiedNameType>) -> String {
    members
        .iter()
        .enumerate()
        .map(|(i, m)| {
            if m.r#type.starts_with("Option<") {
                format!(
                    r#"{i} => Ok(|parent, output| {{
                        if let Some(value) = parent.{name}.clone() {{
                        {t}::encode::<E>(value, output)
                    }} else {{
                        return Ok(output);
                    }}
                }}),"#,
                    name = to_rust_camel_case(&m.name),
                    t = &m.r#type[7..m.r#type.len() - 1],
                )
            } else {
                format!(
                    "{i} => Ok(|parent, output| {t}::encode::<E>(parent.{name}.clone(), output)),",
                    name = to_rust_camel_case(&m.name),
                    t = m.r#type,
                )
            }
        })
        .collect::<Vec<String>>()
        .join("\n      ")
}

pub fn format_has_optional_body(members: &Vec<StringifiedNameType>) -> String {
    members
        .iter()
        .enumerate()
        .map(|(i, m)| {
            if m.r#type.starts_with("Option<") {
                format!(
                    r#"{i} => self.{name} != None,"#,
                    name = to_rust_camel_case(&m.name),
                )
            } else {
                format!("{i} => true,")
            }
        })
        .collect::<Vec<String>>()
        .join("\n      ")
}

fn declare_inner_sequence_member(
    member: &SequenceOrSetMember,
    parent_name: &String,
) -> Result<String, GeneratorError> {
    generate(
        &Framework::Asnr,
        ToplevelDeclaration::Type(ToplevelTypeDeclaration {
            parameterization: None,
            comments: " Inner type ".into(),
            name: inner_name(&member.name, parent_name),
            r#type: member.r#type.clone(),
            tag: None
        }),
        None,
    )
}

fn declare_inner_choice_option(
    option: &ChoiceOption,
    parent_name: &String,
) -> Result<String, GeneratorError> {
    generate(
        &Framework::Asnr,
        ToplevelDeclaration::Type(ToplevelTypeDeclaration {
            parameterization: None,
            comments: " Inner type ".into(),
            name: inner_name(&option.name, parent_name),
            r#type: option.r#type.clone(),
            tag: None
        }),
        None,
    )
}

fn inner_name(name: &String, parent_name: &String) -> String {
    let mut type_name = name.replace("-", "").replace("_", "");
    let mut name_chars = type_name.chars();
    if let Some(initial) = name_chars.next() {
        type_name = initial.to_uppercase().collect::<String>() + name_chars.as_str();
    }
    format!("Inner{}{}", parent_name, type_name)
}

#[cfg(test)]
mod tests {
    use asnr_grammar::types::*;

    use crate::generator::templates::asnr::util::format_enumeral;

    #[test]
    fn formats_enumeral() {
        assert_eq!(
            "///  This is a descriptive text\n\tTestEnumeral = 2,",
            format_enumeral(&Enumeral {
                name: "TestEnumeral".into(),
                description: Some(" This is a descriptive text".into()),
                index: 2
            })
        )
    }
}

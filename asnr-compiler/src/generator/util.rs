use asnr_grammar::{
    information_object::{
        InformationObjectClass, ObjectFieldIdentifier,
        SyntaxApplication, SyntaxExpression, SyntaxToken,
    },
    types::*,
    *,
};

use super::{
    builder::StringifiedNameType,
    error::{GeneratorError, GeneratorErrorType},
    generate,
};

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
        _o => {
            todo!()
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
    }) + &rustify_name(&enumeral.name)
        + " = "
        + &enumeral.index.to_string()
        + ","
}

pub fn format_option_from_int(args: (usize, &StringifiedNameType)) -> String {
    format!(
        r#"x if x == {index} => Ok(|decoder, input| {{
    {t}::decode::<D>(input).map(|(r, v)|(r, Self::{name}(v)))
  }}),"#,
        index = args.0,
        name = args.1.name,
        t = args.1.r#type
    )
}

pub fn format_enumeral_from_int(enumeral: &Enumeral) -> String {
    let name = &rustify_name(&enumeral.name);
    format!("x if x == Self::{name} as i128 => Ok(Self::{name}),")
}

pub fn format_distinguished_values(tld: &ToplevelTypeDeclaration) -> String {
    let name = &rustify_name(&tld.name);
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
    let name = &rustify_name(&value.name);
    let i = value.value;
    format!("pub fn is_{name}(&self) -> bool {{ *self.0.get({i}).unwrap_or(&false) }}")
}

pub fn format_distinguished_int_value(value: &DistinguishedValue) -> String {
    let name = rustify_name(&value.name);
    let i = value.value;
    format!("pub fn is_{name}(&self) -> bool {{ self.0 as i128 == {i} }}")
}

pub fn flatten_nested_sequence_members(
    members: &Vec<SequenceMember>,
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
            let name = rustify_name(&m.name);
            let rtype = match &m.r#type {
                ASN1Type::ElsewhereDeclaredType(d) => rustify_name(&d.identifier),
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
    members: &Vec<SequenceMember>,
    parent_name: &String,
) -> Vec<StringifiedNameType> {
    members
        .iter()
        .map(|m| {
            let name = rustify_name(&m.name);
            let rtype = match &m.r#type {
                ASN1Type::ElsewhereDeclaredType(d) => rustify_name(&d.identifier),
                ASN1Type::InformationObjectFieldReference(_) => {
                  "ASN1_OPEN".to_string()
                }
                _ => inner_name(&m.name, parent_name),
            };
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
        .map(|m| format!("pub {}: {},", m.name, m.r#type))
        .collect::<Vec<String>>()
        .join("\n  ")
}

pub fn format_extensible_sequence<'a>(name: &String, extensible: bool) -> (String, String) {
    (
        if extensible {
            "\n  pub unknown_extension: Vec<u8>,".into()
        } else {
            "".into()
        },
        if extensible {
            "{ (input, self.unknown_extension) = D::decode_unknown_extension(input)? },".into()
        } else {
            format!(
                r#"return Err(
              nom::Err::Error(nom::error::Error {{ input, code: nom::error::ErrorKind::Fail }})
            )"#
            )
        },
    )
}

pub fn format_decode_member_body(members: &Vec<StringifiedNameType>) -> String {
    members
        .iter()
        .enumerate()
        .map(|(i, m)| {
            format!(
                "{i} => {{ (input, self.{name}) = {t}::decode::<D>(input)? }},",
                t = m.r#type,
                name = m.name
            )
        })
        .collect::<Vec<String>>()
        .join("\n      ")
}

fn declare_inner_sequence_member(
    member: &SequenceMember,
    parent_name: &String,
) -> Result<String, GeneratorError> {
    generate(
        ToplevelDeclaration::Type(ToplevelTypeDeclaration {
            parameterization: None,
            comments: " Inner type ".into(),
            name: inner_name(&member.name, parent_name),
            r#type: member.r#type.clone(),
        }),
        None,
    )
}

fn declare_inner_choice_option(
    option: &ChoiceOption,
    parent_name: &String,
) -> Result<String, GeneratorError> {
    generate(
        ToplevelDeclaration::Type(ToplevelTypeDeclaration {
            parameterization: None,
            comments: " Inner type ".into(),
            name: inner_name(&option.name, parent_name),
            r#type: option.r#type.clone(),
        }),
        None,
    )
}

fn inner_name(name: &String, parent_name: &String) -> String {
    format!("{}_inner_{}", parent_name, rustify_name(&name))
}

pub fn rustify_name(name: &String) -> String {
    name.replace("-", "_")
}

#[cfg(test)]
mod tests {
    use asnr_grammar::types::*;

    use crate::generator::util::format_enumeral;

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

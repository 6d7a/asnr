use asnr_grammar::{types::*, *};

use super::{builder::StringifiedNameType, error::GeneratorError, generate};

pub fn int_type_token<'a>(min: i128, max: i128) -> &'a str {
    match max - min {
        r if r <= u8::MAX.into() && min >= 0 => "u8",
        r if r <= u8::MAX.into() => "i8",
        r if r <= u16::MAX.into() && min >= 0 => "u16",
        r if r <= u16::MAX.into() => "i16",
        r if r <= u32::MAX.into() && min >= 0 => "u32",
        r if r <= u32::MAX.into() => "i32",
        r if r <= u64::MAX.into() && min >= 0 => "u64",
        r if r <= u64::MAX.into() => "i64",
        _ => "i128",
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
    {t}::decode(decoder, input).map(|(r, v)|(r, Self::{name}(v)))
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

pub fn format_distinguished_values(tld: &ToplevelDeclaration) -> String {
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

pub fn flatten_nested_sequence_members(members: &Vec<SequenceMember>, parent_name: &String) -> Vec<String> {
    members
        .iter()
        .filter(|m| match m.r#type {
            ASN1Type::ElsewhereDeclaredType(_) => false,
            _ => true,
        })
        .map(|i| declare_inner_sequence_member(i, parent_name).unwrap())
        .collect::<Vec<String>>()
}

pub fn flatten_nested_choice_options(options: &Vec<ChoiceOption>, parent_name: &String) -> Vec<String> {
    options
        .iter()
        .filter(|m| match m.r#type {
            ASN1Type::ElsewhereDeclaredType(_) => false,
            _ => true,
        })
        .map(|i| declare_inner_choice_option(i, parent_name).unwrap())
        .collect::<Vec<String>>()
}

pub fn extract_choice_options(options: &Vec<ChoiceOption>, parent_name: &String) -> Vec<StringifiedNameType> {
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

pub fn extract_sequence_members(members: &Vec<SequenceMember>, parent_name: &String) -> Vec<StringifiedNameType> {
    members
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
            "decoder.decode_unknown_extension(input).map(|(r, v)| {{ input = r; self.unknown_extension = v.to_vec(); }})?,".into()
        } else {
            format!(
                r#"return Err(
              DecodingError::new(
                &format!("Invalid sequence member index decoding {name}. Received index {{}}",index), DecodingErrorType::InvalidSequenceMemberIndex
              )
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
                "{i} => {t}::decode(decoder, input).map(|(r,v)| {{ self.{name} = v; input = r; }})?,",
                t = m.r#type,
                name = m.name
            )
        })
        .collect::<Vec<String>>()
        .join("\n      ")
}

fn declare_inner_sequence_member(member: &SequenceMember, parent_name: &String) -> Result<String, GeneratorError> {
    generate(
        ToplevelDeclaration {
            comments: " Inner type ".into(),
            name: inner_name(&member.name, parent_name),
            r#type: member.r#type.clone(),
        },
        None,
    )
}

fn declare_inner_choice_option(option: &ChoiceOption, parent_name: &String) -> Result<String, GeneratorError> {
    generate(
        ToplevelDeclaration {
            comments: " Inner type ".into(),
            name: inner_name(&option.name, parent_name),
            r#type: option.r#type.clone(),
        },
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

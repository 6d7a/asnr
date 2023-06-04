use asnr_grammar::{
    ASN1Type, DistinguishedValue, Enumeral, SequenceMember, ToplevelDeclaration,
};

use super::{error::GeneratorError, generate};

pub fn format_comments(comments: &String) -> String {
    if comments.is_empty() {
        String::from("")
    } else {
        String::from("/* ") + comments + "*/\n"
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

pub fn flatten_nested_sequence_members(members: &Vec<SequenceMember>) -> Vec<String> {
    members
        .iter()
        .filter(|m| match m.r#type {
            ASN1Type::ElsewhereDeclaredType(_) => false,
            _ => true,
        })
        .map(|i| declare_inner_sequence_member(i).unwrap())
        .collect::<Vec<String>>()
}

pub fn format_sequence_members(members: &Vec<SequenceMember>) -> String {
    members
        .iter()
        .map(|m| {
            let name = rustify_name(&m.name);
            let rtype = match &m.r#type {
                ASN1Type::ElsewhereDeclaredType(d) => rustify_name(&d.0),
                _ => inner_name(&m.name),
            };
            format!("pub {name}: {rtype},")
        })
        .collect::<Vec<String>>()
        .join("\n  ")
}

fn declare_inner_sequence_member(member: &SequenceMember) -> Result<String, GeneratorError> {
    generate(
        ToplevelDeclaration {
            comments: " Inner type ".into(),
            name: inner_name(&member.name),
            r#type: member.r#type.clone(),
        },
        None,
    )
}

fn inner_name(name: &String) -> String {
    format!("Inner_{}", rustify_name(&name))
}

pub fn rustify_name(name: &String) -> String {
    name.replace("-", "_")
}

#[cfg(test)]
mod tests {
    use asnr_grammar::Enumeral;

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

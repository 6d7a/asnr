use alloc::{collections::BTreeMap, string::String, vec::Vec};

use crate::{
    information_object::{InformationObjectClassField, ObjectFieldIdentifier},
    ASN1Value, ToplevelDeclaration,
};

pub fn int_type_token<'a>(min: i128, max: i128) -> &'a str {
    if min >= 0 {
        match max {
            r if r <= u8::MAX.into() => "u8",
            r if r <= u16::MAX.into() => "u16",
            r if r <= u32::MAX.into() => "u32",
            r if r <= u64::MAX.into() => "u64",
            _ => "u128",
        }
    } else {
        match (min, max) {
            (mi, ma) if mi >= i8::MIN.into() && ma <= i8::MAX.into() => "i8",
            (mi, ma) if mi >= i16::MIN.into() && ma <= i16::MAX.into() => "i16",
            (mi, ma) if mi >= i32::MIN.into() && ma <= i32::MAX.into() => "i32",
            (mi, ma) if mi >= i64::MIN.into() && ma <= i64::MAX.into() => "i64",
            _ => "i128",
        }
    }
}

pub(crate) fn find_tld_or_enum_value_by_name(
    type_name: &String,
    name: &String,
    tlds: &BTreeMap<String, ToplevelDeclaration>,
) -> Option<ASN1Value> {
    if let Some(ToplevelDeclaration::Value(v)) = tlds.get(name) {
        return Some(v.value.clone());
    } else {
        for (_, tld) in tlds.iter() {
            if let Some(value) = tld.get_distinguished_or_enum_value(Some(type_name), name) {
                return Some(value);
            }
        }
        // Make second attempt without requiring a matching type name
        // This is the current best shot at linking inner subtypes
        for (_, tld) in tlds.iter() {
            if let Some(value) = tld.get_distinguished_or_enum_value(None, name) {
                return Some(value);
            }
        }
    }
    None
}

pub(crate) fn walk_object_field_ref_path<'a>(
    fields: &'a Vec<InformationObjectClassField>,
    path: &'a Vec<ObjectFieldIdentifier>,
    mut index: usize,
) -> Option<&'a InformationObjectClassField> {
    fields
        .iter()
        .find_map(|f| {
            path.get(index)
                .map(|id| {
                    (&f.identifier == id).then(|| {
                        if path.len() == (index + 1) {
                            Some(f)
                        } else {
                            index += 1;
                            walk_object_field_ref_path(fields, path, index)
                        }
                    })
                })
                .flatten()
        })
        .flatten()
}

const RUST_KEYWORDS: [&'static str; 38] = [
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while",
];

pub fn to_rust_snake_case(input: &String) -> String {
    let mut input = input.replace("-", "_");
    let input = input.drain(..).fold(String::new(), |mut acc, c| {
        if acc.is_empty() && c.is_uppercase() {
            acc.push(c.to_ascii_lowercase());
        } else if acc.ends_with(|last: char| last.is_lowercase() || last == '_') && c.is_uppercase()
        {
            acc.push('_');
            acc.push(c.to_ascii_lowercase());
        } else {
            acc.push(c);
        }
        acc
    });
    if RUST_KEYWORDS.contains(&input.as_str()) {
        String::from("r_") + &input
    } else {
        input
    }
}

pub fn to_rust_const_case(input: &String) -> String {
    to_rust_snake_case(input).to_uppercase()
}

pub fn to_rust_title_case(input: &String) -> String {
    let mut input = input.replace("-", "_");
    input.drain(..).fold(String::new(), |mut acc, c| {
        if acc.is_empty() && c.is_lowercase() {
            acc.push(c.to_ascii_uppercase());
        } else if acc.ends_with(|last: char| last == '_') && c.is_uppercase() {
            acc.pop();
            acc.push(c);
        } else if acc.ends_with(|last: char| last == '_') {
            acc.pop();
            acc.push(c.to_ascii_uppercase());
        } else {
            acc.push(c);
        }
        acc
    })
}

#[cfg(test)]
mod tests {
    use crate::utils::int_type_token;

    #[test]
    fn determines_int_type() {
        assert_eq!(int_type_token(600, 600), "u16");
        assert_eq!(int_type_token(0, 0), "u8");
        assert_eq!(int_type_token(-1, 1), "i8");
        assert_eq!(int_type_token(0, 124213412341389457931857915125), "u128");
        assert_eq!(int_type_token(-67463, 23123), "i32");
        assert_eq!(int_type_token(255, 257), "u16");
    }
}

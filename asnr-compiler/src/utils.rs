

const RUST_KEYWORDS: [&'static str; 38] = [
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while",
];

pub fn to_rust_camel_case(input: &String) -> String {
    let mut input = input.replace("-", "_");
    let input = input.drain(..).fold(String::new(), |mut acc, c| {
        if acc.is_empty() && c.is_uppercase() {
            acc.push(c.to_ascii_lowercase());
        } else if acc.ends_with(|last: char| last.is_lowercase() || last == '_') && c.is_uppercase() {
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
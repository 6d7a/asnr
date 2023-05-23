use asnr_grammar::Enumeral;

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
    }) + &enumeral.name
        + " = "
        + &enumeral.index.to_string()
        + ","
}

pub fn format_enumeral_from_int(enumeral: &Enumeral) -> String {
  let name = &enumeral.name;
  format!("x if x == Self::{name} as i128 => Ok(Self::{name}),")
}

#[cfg(test)]
mod tests {
    use asnr_grammar::Enumeral;

    use crate::generator::util::format_enumeral;

  #[test]
  fn formats_enumeral() {
    assert_eq!("///  This is a descriptive text\n\tTestEnumeral = 2,", format_enumeral(&Enumeral { name: "TestEnumeral".into(), description: Some(" This is a descriptive text".into()), index: 2 }))
  }
}
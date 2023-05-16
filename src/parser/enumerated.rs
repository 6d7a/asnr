use nom::{
    bytes::complete::tag,
    character::complete::{char, u64},
    combinator::{map, opt},
    multi::fold_many1,
    sequence::{pair, preceded, tuple},
    IResult,
};

use crate::grammar::token::{
    ASN1Type, Enumeral, ExtensionMarker, COMMA, ENUMERATED, 
};

use super::common::*;

fn enumeral<'a>(
    input: &'a str,
) -> IResult<&'a str, (&str, Option<u64>, Option<char>, Option<&str>)> {
    skip_ws_and_comments(tuple((
        skip_ws_and_comments(identifier),
        skip_ws_and_comments(opt(in_parentheses(u64))),
        skip_ws(opt(char(COMMA))),
        opt(comment),
    )))(input)
}

fn enumerals<'a>(input: &'a str) -> IResult<&'a str, Vec<Enumeral>> {
    fold_many1(
        enumeral,
        Vec::<Enumeral>::new,
        |mut acc, (name, index, _, comments)| {
            acc.push(Enumeral {
                name: name.into(),
                description: comments.map(|c| c.into()),
                index: index.unwrap_or(acc.len() as u64),
            });
            acc
        },
    )(input)
}

fn enumerated_body<'a>(
    input: &'a str,
) -> IResult<&'a str, (Vec<Enumeral>, Option<ExtensionMarker>)> {
    in_braces(pair(enumerals, opt(extension_marker)))(input)
}

pub fn enumerated<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    map(
        preceded(skip_ws_and_comments(tag(ENUMERATED)), enumerated_body),
        |m| ASN1Type::Enumerated(m.into()),
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::{
        grammar::token::{ASN1Type, AsnEnumerated, Enumeral},
        parser::enumerated::{enumerals, enumerated},
    };

    #[test]
    fn parses_enumerals_with_line_comments() {
        assert_eq!(
            enumerals(
                r#"forward     (1), -- This means forward
      backward    (2), -- This means backward
      unavailable (3)  -- This means nothing"#
            )
            .unwrap()
            .1,
            [
                Enumeral {
                    name: "forward".into(),
                    description: Some(" This means forward".into(),),
                    index: 1,
                },
                Enumeral {
                    name: "backward".into(),
                    description: Some(" This means backward".into(),),
                    index: 2,
                },
                Enumeral {
                    name: "unavailable".into(),
                    description: Some(" This means nothing".into(),),
                    index: 3,
                },
            ],
        )
    }

    #[test]
    fn parses_enumerated() {
        assert_eq!(
            enumerated(
                r#"ENUMERATED {
      onePerMeter-0-1,
      outOfRange,
      unavailable
  }"#
            )
            .unwrap()
            .1,
            ASN1Type::Enumerated(AsnEnumerated {
                members: vec![
                    Enumeral {
                        name: "onePerMeter-0-1".into(),
                        description: None,
                        index: 0
                    },
                    Enumeral {
                        name: "outOfRange".into(),
                        description: None,
                        index: 1
                    },
                    Enumeral {
                        name: "unavailable".into(),
                        description: None,
                        index: 2
                    }
                ],
                extensible: false
            })
        )
    }

    #[test]
    fn parses_extended_enumerated() {
        assert_eq!(
            enumerated("ENUMERATED {m1, m2, m3 -- another annoying comment we'll ignore --,...}")
                .unwrap()
                .1,
            ASN1Type::Enumerated(AsnEnumerated {
                members: vec![
                    Enumeral {
                        name: "m1".into(),
                        description: None,
                        index: 0
                    },
                    Enumeral {
                        name: "m2".into(),
                        description: None,
                        index: 1
                    },
                    Enumeral {
                        name: "m3".into(),
                        description: None,
                        index: 2
                    }
                ],
                extensible: true
            })
        )
    }

    #[test]
    fn parses_indexed_enumerated() {
        assert_eq!(
            enumerated(
                r#"ENUMERATED {
          forward     (1),--This means forward
          -- Annoyance
          -- another annoyance -- backward    (2), --This means backward
          unavailable (3)--This means nothing
      }"#
            )
            .unwrap()
            .1,
            ASN1Type::Enumerated(AsnEnumerated {
                members: vec![
                    Enumeral {
                        name: "forward".into(),
                        description: Some("This means forward".into()),
                        index: 1
                    },
                    Enumeral {
                        name: "backward".into(),
                        description: Some("This means backward".into()),
                        index: 2
                    },
                    Enumeral {
                        name: "unavailable".into(),
                        description: Some("This means nothing".into()),
                        index: 3
                    }
                ],
                extensible: false
            })
        )
    }

    #[test]
    fn parses_indexed_extended_enumerated() {
        assert_eq!(
            enumerated(
                r#"ENUMERATED {
          forward  -- this, too, ignored --   (1),
          -- let's consider this a comment concerning 'forward' -- ... 
      }"#
            )
            .unwrap()
            .1,
            ASN1Type::Enumerated(AsnEnumerated {
                members: vec![Enumeral {
                    name: "forward".into(),
                    description: Some(
                        " let's consider this a comment concerning 'forward' ".into()
                    ),
                    index: 1
                },],
                extensible: true
            })
        )
    }
}

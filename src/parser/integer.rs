use nom::{
    bytes::complete::tag,
    character::complete::{char, i128},
    combinator::{map, opt},
    multi::many0,
    sequence::{delimited, pair, terminated, tuple},
    IResult,
};

use crate::grammar::token::{ASN1Type, DistinguishedValue, INTEGER};

use super::*;

pub fn integer<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    map(
        tuple((
            skip_ws_and_comments(tag(INTEGER)),
            opt(skip_ws_and_comments(distinguished_values)),
            opt(skip_ws_and_comments(constraint)),
        )),
        |m| ASN1Type::Integer(m.into()),
    )(input)
}

fn distinguished_values<'a>(input: &'a str) -> IResult<&'a str, Vec<DistinguishedValue>> {
    delimited(
        skip_ws_and_comments(char(LEFT_BRACE)),
        many0(terminated(
            skip_ws_and_comments(distinguished_val),
            opt(skip_ws_and_comments(char(COMMA))),
        )),
        skip_ws_and_comments(char(RIGHT_BRACE)),
    )(input)
}

fn distinguished_val<'a>(input: &'a str) -> IResult<&'a str, DistinguishedValue> {
    map_into(pair(skip_ws_and_comments(identifier), in_parentheses(i128)))(input)
}

#[cfg(test)]
mod tests {

    use crate::grammar::token::{AsnInteger, Constraint};

    use super::*;

    #[test]
    fn parses_integer() {
        assert_eq!(
            integer("INTEGER"),
            Ok(("", ASN1Type::Integer(AsnInteger::default())))
        );
        assert_eq!(
            integer("INTEGER  (-9..-4, ...)"),
            Ok((
                "",
                ASN1Type::Integer(Constraint::new(Some(-9), Some(-4), true).into())
            ))
        );
        assert_eq!(
            integer("\r\nINTEGER(-9..-4)"),
            Ok((
                "",
                ASN1Type::Integer(Constraint::new(Some(-9), Some(-4), false).into())
            ))
        );
    }

    #[test]
    fn parses_distinguished_values() {
        let sample = r#"{
    positiveOutOfRange (160),
    unavailable        (161)  
}"#;
        assert_eq!(
            distinguished_values(sample).unwrap().1,
            [
                DistinguishedValue {
                    name: "positiveOutOfRange".into(),
                    value: 160,
                },
                DistinguishedValue {
                    name: "unavailable".into(),
                    value: 161,
                },
            ]
        )
    }

    #[test]
    fn parses_distinguished_values_with_line_comments() {
        let sample = r#"{
    negativeOutOfRange (159), -- ignore this comment
    positiveOutOfRange (160), -- ignore this comment, too
    unavailable        (161)  
}"#;
        assert_eq!(
            distinguished_values(sample).unwrap().1,
            [
                DistinguishedValue {
                    name: "negativeOutOfRange".into(),
                    value: 159,
                },
                DistinguishedValue {
                    name: "positiveOutOfRange".into(),
                    value: 160,
                },
                DistinguishedValue {
                    name: "unavailable".into(),
                    value: 161,
                },
            ]
        )
    }
}

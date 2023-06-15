use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{
        alpha1, alphanumeric1, char, i128, multispace0, multispace1, not_line_ending,
    },
    combinator::{opt, recognize},
    multi::many0,
    sequence::{delimited, pair, preceded, terminated},
    IResult,
};

use asnr_grammar::{*, subtyping::*, types::*};

use super::util::{take_until_or, map_into};

/// This matches both spec-conform ASN1 comments ("--")
/// as well as C-style comments commonly seen ("//", "/* */")
pub fn comment<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    skip_ws(alt((block_comment, line_comment)))(input)
}

pub fn line_comment<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    preceded(
        alt((tag(C_STYLE_LINE_COMMENT), tag(ASN1_COMMENT))),
        not_line_ending,
    )(input)
}

pub fn block_comment<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    alt((
        delimited(
            tag(C_STYLE_BLOCK_COMMENT_BEGIN),
            take_until(C_STYLE_BLOCK_COMMENT_END),
            tag(C_STYLE_BLOCK_COMMENT_END),
        ),
        delimited(
            tag(ASN1_COMMENT),
            take_until_or("\n", ASN1_COMMENT),
            tag(ASN1_COMMENT),
        ),
    ))(input)
}

pub fn identifier<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    recognize(pair(
        alpha1,
        many0(alt((preceded(char('-'), alphanumeric1), alphanumeric1))),
    ))(input)
}

pub fn skip_ws<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    preceded(multispace0, inner)
}

pub fn skip_ws_and_comments<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    preceded(many0(alt((comment, multispace1))), inner)
}

pub fn in_parentheses<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    delimited(
        skip_ws_and_comments(char(LEFT_PARENTHESIS)),
        skip_ws_and_comments(inner),
        skip_ws_and_comments(char(RIGHT_PARENTHESIS)),
    )
}

pub fn in_braces<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    delimited(
        skip_ws_and_comments(char(LEFT_BRACE)),
        skip_ws_and_comments(inner),
        skip_ws_and_comments(char(RIGHT_BRACE)),
    )
}

pub fn range_marker<'a>(input: &'a str) -> IResult<&'a str, RangeMarker> {
    skip_ws_and_comments(tag(RANGE))(input).map(|(remaining, _)| (remaining, RangeMarker()))
}

pub fn extension_marker<'a>(input: &'a str) -> IResult<&'a str, ExtensionMarker> {
    skip_ws_and_comments(tag(ELLIPSIS))(input).map(|(remaining, _)| (remaining, ExtensionMarker()))
}


pub fn assignment<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    skip_ws_and_comments(tag(ASSIGN))(input)
}

pub fn distinguished_values<'a>(input: &'a str) -> IResult<&'a str, Vec<DistinguishedValue>> {
    delimited(
        skip_ws_and_comments(char(LEFT_BRACE)),
        many0(terminated(
            skip_ws_and_comments(distinguished_val),
            opt(skip_ws_and_comments(char(COMMA))),
        )),
        skip_ws_and_comments(char(RIGHT_BRACE)),
    )(input)
}

pub fn distinguished_val<'a>(input: &'a str) -> IResult<&'a str, DistinguishedValue> {
    map_into(pair(skip_ws_and_comments(identifier), in_parentheses(i128)))(input)
}

#[cfg(test)]
mod tests {

    use crate::parser::constraint::value_constraint;

    use super::*;

    #[test]
    fn parses_line_comment() {
        let line = r#"// Test, one, two, three/
"#;
        assert_eq!(" Test, one, two, three/", comment(line).unwrap().1);
    }
    
    #[test]
    fn parses_block_comment() {
        assert_eq!(
            r#" Test, one, two, three
and one "#,
            comment(
                r#"/* Test, one, two, three
and one */"#
            )
            .unwrap()
            .1
        );
        assert_eq!(
            r#"*
       * Hello
       "#,
            comment(
                r#"/**
       * Hello
       */"#
            )
            .unwrap()
            .1
        );
        assert_eq!(
            " Very annoying! ",
            comment("-- Very annoying! --").unwrap().1
        )
    }

    #[test]
    fn parses_ambiguous_asn1_comment() {
        assert_eq!(
            comment(
                r#" -- This means backward
      unavailable"#
            ),
            Ok(("\n      unavailable", " This means backward",),)
        );
        assert_eq!(
            comment(
                r#"-- This means forward
        backward    (2), -- This means backward"#
            ),
            Ok((
                "\n        backward    (2), -- This means backward",
                " This means forward",
            ),)
        )
    }

    #[test]
    fn parses_valid_identifiers() {
        assert_eq!(identifier("EEE-DDD"), Ok(("", "EEE-DDD")));
        assert_eq!(identifier("GenericLane "), Ok((" ", "GenericLane")));
        assert_eq!(identifier("regional "), Ok((" ", "regional")));
        assert_eq!(identifier("NodeXY64"), Ok(("", "NodeXY64")));
        assert_eq!(identifier("Sub-Cause-Code  "), Ok(("  ", "Sub-Cause-Code")));
    }

    #[test]
    fn handles_invalid_identifiers() {
        assert_eq!(identifier("EEE--DDD"), Ok(("--DDD", "EEE")));
        assert!(identifier("-GenericLane").is_err());
        assert!(identifier("64NodeXY").is_err());
        assert!(identifier("&regional").is_err());
        assert_eq!(identifier("Sub-Cause-Code-"), Ok(("-", "Sub-Cause-Code")));
    }

    #[test]
    fn discards_whitespace() {
        assert_eq!(skip_ws(identifier)(" EEE-DDD"), Ok(("", "EEE-DDD")));
        assert_eq!(
            skip_ws(identifier)("\nGenericLane "),
            Ok((" ", "GenericLane"))
        );
        assert_eq!(skip_ws(identifier)("\t regional "), Ok((" ", "regional")));
        assert_eq!(skip_ws(identifier)("   NodeXY64"), Ok(("", "NodeXY64")));
        assert_eq!(
            skip_ws(identifier)("\r\n\nSub-Cause-Code  "),
            Ok(("  ", "Sub-Cause-Code"))
        );
    }

    #[test]
    fn discards_whitespace_and_comments() {
        assert_eq!(
            skip_ws_and_comments(identifier)(" -- comment --EEE-DDD"),
            Ok(("", "EEE-DDD"))
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

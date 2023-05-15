use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until},
    character::complete::{alpha1, alphanumeric1, char, i128, multispace0, multispace1, u64},
    combinator::{map, opt, recognize},
    multi::{fold_many1, many0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

use crate::{
    grammar::token::{
        ASN1Type, Constraint, Enumeral, ExtensionMarker, RangeMarker, ToplevelDeclaration,
        ASN1_COMMENT, ASSIGN, COMMA, C_STYLE_BLOCK_COMMENT_BEGIN, C_STYLE_BLOCK_COMMENT_END,
        C_STYLE_LINE_COMMENT, ENUMERATED, EXTENSION, LEFT_BRACE, LEFT_PARENTHESIS, RANGE,
        RIGHT_BRACE, RIGHT_PARENTHESIS,
    },
    parser::integer::integer,
};

use super::util::map_into;

/// This matches both spec-conform ASN1 comments ("--")
/// as well as C-style comments commonly seen ("//", "/* */")
pub fn comment<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    alt((block_comment, line_comment))(input)
}

pub fn line_comment<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    preceded(
        alt((tag(C_STYLE_LINE_COMMENT), tag(ASN1_COMMENT))),
        is_not("\n"),
    )(input)
}

pub fn block_comment<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    delimited(
        alt((tag(C_STYLE_BLOCK_COMMENT_BEGIN), tag(ASN1_COMMENT))),
        alt((
            take_until(C_STYLE_BLOCK_COMMENT_END),
            take_until(ASN1_COMMENT),
        )),
        alt((tag(C_STYLE_BLOCK_COMMENT_END), tag(ASN1_COMMENT))),
    )(input)
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
    preceded(many0(alt((skip_ws(comment), multispace1))), inner)
}

pub fn int_in_parentheses<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    delimited(
        skip_ws_and_comments(char(LEFT_PARENTHESIS)),
        skip_ws_and_comments(inner),
        skip_ws_and_comments(char(RIGHT_PARENTHESIS)),
    )
}

pub fn constraint<'a>(input: &'a str) -> IResult<&'a str, Constraint> {
    delimited(
        skip_ws_and_comments(char(LEFT_PARENTHESIS)),
        skip_ws_and_comments(alt((
            extensible_range_constraint, // The most elaborate match first
            strict_extensible_constraint,
            range_constraint,
            strict_constraint, // The most simple match last
        ))),
        skip_ws_and_comments(char(RIGHT_PARENTHESIS)),
    )(input)
}

pub fn range_particle<'a>(input: &'a str) -> IResult<&'a str, RangeMarker> {
    skip_ws_and_comments(tag(RANGE))(input).map(|(remaining, _)| (remaining, RangeMarker()))
}

pub fn extension_particle<'a>(input: &'a str) -> IResult<&'a str, ExtensionMarker> {
    skip_ws_and_comments(tag(EXTENSION))(input).map(|(remaining, _)| (remaining, ExtensionMarker()))
}

pub fn strict_constraint<'a>(input: &'a str) -> IResult<&'a str, Constraint> {
    map_into(i128)(input)
}

pub fn strict_extensible_constraint<'a>(input: &'a str) -> IResult<&'a str, Constraint> {
    map_into(pair(i128, preceded(char(','), extension_particle)))(input)
}

pub fn range_constraint<'a>(input: &'a str) -> IResult<&'a str, Constraint> {
    map_into(tuple((i128, range_particle, skip_ws_and_comments(i128))))(input)
}

pub fn extensible_range_constraint<'a>(input: &'a str) -> IResult<&'a str, Constraint> {
    map_into(tuple((
        i128,
        range_particle,
        skip_ws_and_comments(i128),
        preceded(skip_ws_and_comments(char(COMMA)), extension_particle),
    )))(input)
}

pub fn assignment<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    skip_ws_and_comments(tag(ASSIGN))(input)
}

pub fn enumerals<'a>(input: &'a str) -> IResult<&'a str, Vec<Enumeral>> {
    delimited(
        skip_ws_and_comments(char(LEFT_BRACE)),
        fold_many1(
            terminated(
                skip_ws_and_comments(pair(
                    identifier,
                    skip_ws_and_comments(opt(int_in_parentheses(u64))),
                )),
                opt(skip_ws_and_comments(char(COMMA))),
            ),
            Vec::<Enumeral>::new,
            |mut acc, (name, index)| {
                acc.push(Enumeral {
                    name: name.into(),
                    index: index.unwrap_or(acc.len() as u64),
                });
                acc
            },
        ),
        skip_ws_and_comments(char(RIGHT_BRACE)),
    )(input)
}

pub fn enumated<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    map(
        preceded(skip_ws_and_comments(tag(ENUMERATED)), enumerals),
        |m| ASN1Type::Enumerated(m.into()),
    )(input)
}

#[cfg(test)]
mod tests {
    use core::panic;

    use crate::grammar::token::{ASN1Type, Constraint, DistinguishedValue};

    use super::{comment, constraint, identifier, skip_ws, skip_ws_and_comments};

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
    fn parses_constraint() {
        assert_eq!(
            constraint("(5)"),
            Ok(("", Constraint::new(Some(5), Some(5), false)))
        );
        assert_eq!(
            constraint("(5..9)"),
            Ok(("", Constraint::new(Some(5), Some(9), false)))
        );
        assert_eq!(
            constraint("(-5..9)"),
            Ok(("", Constraint::new(Some(-5), Some(9), false)))
        );
        assert_eq!(
            constraint("(-9..-4, ...)"),
            Ok(("", Constraint::new(Some(-9), Some(-4), true)))
        );
    }

    #[test]
    fn parses_constraint_with_inserted_comment() {
        assert_eq!(
            constraint("(-9..-4, -- Very annoying! -- ...)"),
            Ok(("", Constraint::new(Some(-9), Some(-4), true)))
        );
        assert_eq!(
            constraint("(-9-- Very annoying! --..-4,  ...)"),
            Ok(("", Constraint::new(Some(-9), Some(-4), true)))
        );
    }
}

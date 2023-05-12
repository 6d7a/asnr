use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{alpha1, alphanumeric1, char, digit1, i128, multispace0},
    combinator::{map, recognize, value},
    multi::many0,
    sequence::{delimited, pair, preceded, tuple, Tuple},
    IResult,
};

use crate::grammar::token::{INTEGER, RANGE};

use super::{
    token::{AsnInteger, Constraint, ExtensionParticle, RangeParticle, ASSIGN, EXTENSION},
    util::{map_into, try_parse_result_into},
};

/// This matches both spec-conform ASN1 comments ("--")
/// as well as C-style comments commonly seen ("//", "/* */")
pub fn comment<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    alt((line_comment, block_comment))(input)
}

fn line_comment<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    preceded(alt((tag("//"), tag("--"))), is_not("\n"))(input)
}

fn block_comment<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    delimited(
        alt((tag("/*"), tag("--"))),
        alt((is_not("/*"), is_not("--"))),
        alt((tag("*/"), tag("--"))),
    )(input)
}

pub fn identifier<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    recognize(pair(
        alpha1,
        many0(alt((preceded(char('-'), alphanumeric1), alphanumeric1))),
    ))(input)
}

fn skip_ws<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    preceded(multispace0, inner)
}

fn constraint<'a>(input: &'a str) -> IResult<&'a str, Constraint> {
    delimited(
        char('('),
        alt((
            extensible_range_constraint, // The most elaborate match first
            strict_extensible_constraint,
            range_constraint,
            strict_constraint, // The most simple match last
        )),
        char(')'),
    )(input)
}

fn range_particle<'a>(input: &'a str) -> IResult<&'a str, RangeParticle> {
    tag(RANGE)(input).map(|(remaining, dotdot)| (remaining, RangeParticle()))
}

fn extension_particle<'a>(input: &'a str) -> IResult<&'a str, ExtensionParticle> {
    tag(EXTENSION)(input).map(|(remaining, dotdotdot)| (remaining, ExtensionParticle()))
}

fn strict_constraint<'a>(input: &'a str) -> IResult<&'a str, Constraint> {
    map_into(i128)(input)
}

fn strict_extensible_constraint<'a>(input: &'a str) -> IResult<&'a str, Constraint> {
    map_into(pair(i128, preceded(char(','), skip_ws(extension_particle))))(input)
}

fn range_constraint<'a>(input: &'a str) -> IResult<&'a str, Constraint> {
    map_into(tuple((i128, range_particle, i128)))(input)
}

fn extensible_range_constraint<'a>(input: &'a str) -> IResult<&'a str, Constraint> {
    map_into(tuple((
        i128,
        range_particle,
        i128,
        preceded(char(','), skip_ws(extension_particle)),
    )))(input)
}

fn assignment<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    tag(ASSIGN)(input)
}

pub fn integer<'a>(input: &'a str) -> IResult<&'a str, AsnInteger> {
    alt((
        map_into(preceded(skip_ws(tag(INTEGER)), skip_ws(constraint))),
        value(AsnInteger::default(), skip_ws(tag(INTEGER))),
    ))(input)
}

#[cfg(test)]
mod tests {
    use crate::grammar::{
        parser::{constraint, identifier, integer, skip_ws},
        token::{Constraint, AsnInteger},
    };

    use super::comment;

    #[test]
    fn parses_line_comment() {
        let line = r#"// Test, one, two, three/
    "#;
        assert_eq!(" Test, one, two, three/", comment(line).unwrap().1);
    }

    #[test]
    fn parses_block_comment() {
        let line = r#"/* Test, one, two, three
    and one */"#;
        assert_eq!(
            r#" Test, one, two, three
    and one "#,
            comment(line).unwrap().1
        );
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
    fn parses_integer() {
        assert_eq!(
            integer("INTEGER"),
            Ok(("", AsnInteger::default()))
        );
        assert_eq!(
            integer("INTEGER  (-9..-4, ...)"),
            Ok(("", Constraint::new(Some(-9), Some(-4), true).into()))
        );
        assert_eq!(
            integer("\r\nINTEGER(-9..-4)"),
            Ok(("", Constraint::new(Some(-9), Some(-4), false).into()))
        );
    }
}

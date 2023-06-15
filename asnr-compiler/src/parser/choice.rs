use nom::{
  bytes::complete::tag,
  character::complete::char,
  combinator::{into, opt},
  multi::many0,
  sequence::{terminated, tuple},
  IResult,
};

use asnr_grammar::{*, types::*};

use super::{*, constraint::constraint};

/// Tries to parse an ASN1 CHOICE
///
/// *`input` - string slice to be matched against
///
/// `sequence` will try to match an CHOICE declaration in the `input` string.
/// If the match succeeds, the parser will consume the match and return the remaining string
/// and a wrapped `AsnChoice` value representing the ASN1 declaration. If the defined CHOICE
/// contains anonymous members, these nested members will be represented as
/// structs within the same global scope.
/// If the match fails, the parser will not consume the input and will return an error.
pub fn choice<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
  map(
      preceded(
          skip_ws_and_comments(tag(CHOICE)),
          in_braces(tuple((
              many0(terminated(
                  skip_ws_and_comments(choice_option),
                  skip_ws_and_comments(opt(char(COMMA))),
              )),
              opt(terminated(extension_marker, opt(char(COMMA)))),
              opt(many0(terminated(
                  skip_ws_and_comments(choice_option),
                  skip_ws_and_comments(opt(char(COMMA))),
              ))),
          ))),
      ),
      |m| ASN1Type::Choice(m.into()),
  )(input)
}

fn choice_option<'a>(input: &'a str) -> IResult<&'a str, ChoiceOption> {
  into(tuple((
      skip_ws_and_comments(identifier),
      skip_ws_and_comments(asn1_type),
      opt(skip_ws_and_comments(constraint))
  )))(input)
}
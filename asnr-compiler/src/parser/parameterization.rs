use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, char, one_of},
    combinator::{into, map, opt, recognize, value},
    multi::{many0, many1, separated_list1},
    sequence::{pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

use asnr_grammar::{types::*, *};

use super::common::{identifier, in_braces, skip_ws_and_comments};

pub fn parameterization<'a>(input: &'a str) -> IResult<&'a str, Parameterization> {
    into(skip_ws_and_comments(in_braces(separated_list1(
        char(COMMA),
        skip_ws_and_comments(separated_pair(
            identifier,
            skip_ws_and_comments(char(COLON)),
            skip_ws_and_comments(identifier),
        )),
    ))))(input)
}

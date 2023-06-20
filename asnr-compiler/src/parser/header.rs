use asnr_grammar::{
    ExtensibilityEnvironment, Header, TaggingEnvironment, ASSIGN, AUTOMATIC, BEGIN, DEFINITIONS,
    EXPLICIT, EXTENSIBILITY_IMPLIED, IMPLICIT, TAGS, EncodingReferenceDefault, INSTRUCTIONS,
};
use nom::{
    branch::{alt},
    bytes::complete::tag,
    combinator::{into, map, opt},
    sequence::{delimited, pair, terminated, tuple},
    IResult,
};

use super::{
    common::{identifier, skip_ws, skip_ws_and_comments},
    object_identifier::object_identifier,
};

pub fn module_reference<'a>(input: &'a str) -> IResult<&'a str, Header> {
    skip_ws_and_comments(into(tuple((
        identifier,
        skip_ws(object_identifier),
        skip_ws_and_comments(delimited(
            tag(DEFINITIONS),
            environments,
            skip_ws_and_comments(pair(tag(ASSIGN), skip_ws_and_comments(tag(BEGIN)))),
        )),
    ))))(input)
}

fn environments<'a>(
    input: &'a str,
) -> IResult<&'a str, (EncodingReferenceDefault, TaggingEnvironment, ExtensibilityEnvironment)> {
    tuple((
        skip_ws_and_comments(terminated(into(identifier), tag(INSTRUCTIONS))),
        skip_ws_and_comments(terminated(
            map(
                alt((tag(AUTOMATIC), tag(IMPLICIT), tag(EXPLICIT))),
                |m| match m {
                    AUTOMATIC => TaggingEnvironment::AUTOMATIC,
                    IMPLICIT => TaggingEnvironment::IMPLICIT,
                    _ => TaggingEnvironment::EXPLICIT,
                },
            ),
            skip_ws(tag(TAGS)),
        )),
        skip_ws_and_comments(map(opt(tag(EXTENSIBILITY_IMPLIED)), |m| {
            if m.is_some() {
                ExtensibilityEnvironment::IMPLIED
            } else {
                ExtensibilityEnvironment::EXPLICIT
            }
        })),
    ))(input)
}

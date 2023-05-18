use nom::{
    bytes::complete::tag,
    combinator::{into, opt},
    sequence::tuple,
    IResult,
};

use crate::grammar::token::{OptionMarker, SequenceMember, DEFAULT, OPTIONAL, SEQUENCE};

use super::*;

// pub fn sequence<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
//     map(
//         tuple((
//             skip_ws_and_comments(tag(INTEGER)),
//             opt(skip_ws_and_comments(distinguished_values)),
//             opt(skip_ws_and_comments(constraint)),
//         )),
//         |m| ASN1Type::Integer(m.into()),
//     )(input)
// }

fn sequence_member<'a>(input: &'a str) -> IResult<&'a str, SequenceMember> {
    into(tuple((
        skip_ws_and_comments(tag(SEQUENCE)),
        skip_ws_and_comments(asn1_type),
        optional_marker,
        default,
    )))(input)
}

fn optional_marker<'a>(input: &'a str) -> IResult<&'a str, Option<OptionMarker>> {
    opt(into(skip_ws_and_comments(tag(OPTIONAL))))(input)
}

fn default<'a>(input: &'a str) -> IResult<&'a str, Option<ASN1Value>> {
    opt(preceded(
        skip_ws_and_comments(tag(DEFAULT)),
        skip_ws_and_comments(asn1_value),
    ))(input)
}

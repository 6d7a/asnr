use asnr_grammar::{ASN1Type, OF, SEQUENCE, SIZE};
use nom::{
    bytes::complete::tag,
    combinator::{map, opt},
    sequence::{pair, preceded},
    IResult,
};

use super::{
    asn1_type,
    common::{constraint, in_parentheses, skip_ws_and_comments},
};

/// Tries to parse an ASN1 SEQUENCE OF
///
/// *`input` - string slice to be matched against
///
/// `sequence_of` will try to match an SEQUENCE OF declaration in the `input` string.
/// If the match succeeds, the parser will consume the match and return the remaining string
/// and a wrapped `AsnSequenceOf` value representing the ASN1 declaration.
/// If the match fails, the parser will not consume the input and will return an error.
pub fn sequence_of<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    map(
        pair(
            preceded(
                skip_ws_and_comments(tag(SEQUENCE)),
                opt(in_parentheses(preceded(tag(SIZE), constraint))),
            ),
            preceded(skip_ws_and_comments(tag(OF)), asn1_type),
        ),
        |m| ASN1Type::SequenceOf(m.into()),
    )(input)
}

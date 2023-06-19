use nom::{
    branch::alt,
    character::complete::{alphanumeric1, char, one_of},
    combinator::{map, recognize, opt},
    multi::many0,
    sequence::{pair, preceded, tuple, terminated},
    IResult, bytes::complete::tag,
};

use asnr_grammar::{subtyping::*, types::*, *};

use super::common::{skip_ws_and_comments, in_braces};

pub fn information_object_class<'a>(input: &'a str) -> IResult<&'a str, InformationObjectClass<'a>> {
  map(
    preceded(
        skip_ws_and_comments(tag(CLASS)),
            in_braces(tuple((
                many0(terminated(
                    skip_ws_and_comments(information_object_field),
                    skip_ws_and_comments(opt(char(COMMA))),
                )),
            )),
        ),
    ),
    |m| ASN1Type::InformationObjectClass(m.into()),
)(input)
}

fn information_object_field<'a>(input: &'a str) -> IResult<&'a str, InformationObjectField<'a>> {
  todo!()
}

fn object_field_identifier<'a>(input: &'a str) -> IResult<&'a str, ObjectFieldIdentifier<'a>> {
    preceded(
        char(AMPERSAND),
        alt((single_value_field_id, multiple_value_field_id)),
    )(input)
}

fn single_value_field_id<'a>(input: &'a str) -> IResult<&'a str, ObjectFieldIdentifier<'a>> {
    map(
        recognize(pair(
            one_of("abcdefghijklmnopqrstuvwxyz"),
            many0(alt((preceded(char('-'), alphanumeric1), alphanumeric1))),
        )),
        |m| ObjectFieldIdentifier::SingleValue(m),
    )(input)
}

fn multiple_value_field_id<'a>(input: &'a str) -> IResult<&'a str, ObjectFieldIdentifier<'a>> {
    map(
        recognize(pair(
            one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
            many0(alt((preceded(char('-'), alphanumeric1), alphanumeric1))),
        )),
        |m| ObjectFieldIdentifier::MultipleValue(m),
    )(input)
}

use nom::{bytes::complete::tag, combinator::value, IResult, branch::alt};

use asnr_grammar::{ASN1Type, ASN1Value, BOOLEAN, FALSE, TRUE};

use super::common::skip_ws_and_comments;

pub fn boolean_value<'a>(input: &'a str) -> IResult<&'a str, ASN1Value> {
    alt((
        value(ASN1Value::Boolean(true), skip_ws_and_comments(tag(TRUE))),
        value(ASN1Value::Boolean(false), skip_ws_and_comments(tag(FALSE))),
    ))(input)
}

pub fn boolean<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    value(ASN1Type::Boolean, skip_ws_and_comments(tag(BOOLEAN)))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_boolean() {
        assert_eq!(
            boolean(" --who would put a comment here?--BOOLEAN")
                .unwrap()
                .1,
            ASN1Type::Boolean
        )
    }
}

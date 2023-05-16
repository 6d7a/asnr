use nom::{bytes::complete::tag, combinator::value, IResult};

use crate::grammar::token::{ASN1Type, BOOLEAN};

use super::common::skip_ws_and_comments;

pub fn boolean<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    value(ASN1Type::Boolean, skip_ws_and_comments(tag(BOOLEAN)))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

  #[test]
  fn parses_boolean() {
    assert_eq!(boolean(" --who would put a comment here?--BOOLEAN").unwrap().1, ASN1Type::Boolean)
  }
}
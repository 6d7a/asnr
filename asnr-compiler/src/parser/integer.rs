use nom::{
    bytes::complete::tag,
    combinator::{map, opt},
    sequence::tuple,
    IResult, character::complete::i128,
};

use asnr_grammar::{ASN1Type, INTEGER, ASN1Value};

use super::{*, constraint::value_constraint};

pub fn integer_value<'a>(input: &'a str) -> IResult<&'a str, ASN1Value> {
  map(skip_ws_and_comments(i128), |m| ASN1Value::Integer(m))(input)
}

/// Tries to parse an ASN1 INTEGER
/// 
/// *`input` - string slice to be matched against
/// 
/// `integer` will try to match an INTEGER declaration in the `input` string.
/// If the match succeeds, the parser will consume the match and return the remaining string
/// and a wrapped `AsnInteger` value representing the ASN1 declaration.
/// If the match fails, the parser will not consume the input and will return an error.
pub fn integer<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    map(
        tuple((
            skip_ws_and_comments(tag(INTEGER)),
            opt(skip_ws_and_comments(distinguished_values)),
            opt(skip_ws_and_comments(value_constraint)),
        )),
        |m| ASN1Type::Integer(m.into()),
    )(input)
}

#[cfg(test)]
mod tests {

  use asnr_grammar::{*, subtyping::*, types::*};


    use super::*;

    #[test]
    fn parses_integer() {
        assert_eq!(
            integer("INTEGER"),
            Ok(("", ASN1Type::Integer(AsnInteger::default())))
        );
        assert_eq!(
            integer("INTEGER  (-9..-4, ...)"),
            Ok((
                "",
                ASN1Type::Integer(
                    RangeConstraint {
                        min_value: Some(ASN1Value::Integer(-9)),
                        max_value: Some(ASN1Value::Integer(-4)),
                        extensible: true
                    }
                    .into()
                )
            ))
        );
        assert_eq!(
            integer("\r\nINTEGER(-9..-4)"),
            Ok((
                "",
                ASN1Type::Integer(
                    RangeConstraint {
                        min_value: Some(ASN1Value::Integer(-9)),
                        max_value: Some(ASN1Value::Integer(-4)),
                        extensible: false
                    }
                    .into()
                )
            ))
        );
    }
}

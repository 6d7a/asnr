use nom::{
    bytes::complete::tag,
    combinator::{map, opt},
    sequence::tuple,
    IResult, character::complete::i128,
};

use asnr_grammar::{ASN1Type, INTEGER, ASN1Value};

use super::*;

pub fn integer_value<'a>(input: &'a str) -> IResult<&'a str, ASN1Value> {
  map(skip_ws_and_comments(i128), |m| ASN1Value::Integer(m))(input)
}

pub fn integer<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    map(
        tuple((
            skip_ws_and_comments(tag(INTEGER)),
            opt(skip_ws_and_comments(distinguished_values)),
            opt(skip_ws_and_comments(constraint)),
        )),
        |m| ASN1Type::Integer(m.into()),
    )(input)
}

#[cfg(test)]
mod tests {

    use asnr_grammar::{AsnInteger, Constraint};

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
                    Constraint {
                        min_value: Some(-9),
                        max_value: Some(-4),
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
                    Constraint {
                        min_value: Some(-9),
                        max_value: Some(-4),
                        extensible: false
                    }
                    .into()
                )
            ))
        );
    }
}

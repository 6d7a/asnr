use nom::{
    bytes::complete::tag,
    combinator::{map, opt},
    sequence::preceded,
    IResult,
};

use crate::grammar::token::{ASN1Type, BIT_STRING, SIZE};

use super::common::*;

pub fn bit_string<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    map(
        preceded(
            skip_ws_and_comments(tag(BIT_STRING)),
            opt(in_parentheses(preceded(tag(SIZE), constraint))),
        ),
        |m| ASN1Type::BitString(m.into()),
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::grammar::token::{ASN1Type, AsnBitString, Constraint};

    use super::bit_string;

    #[test]
    fn parses_unconfined_bitstring() {
        let sample = "  BIT STRING";
        assert_eq!(
            bit_string(sample).unwrap().1,
            ASN1Type::BitString(AsnBitString { constraint: None })
        )
    }

    #[test]
    fn parses_strictly_constrained_bitstring() {
        let sample = "  BIT STRING(SIZE (8))";
        assert_eq!(
            bit_string(sample).unwrap().1,
            ASN1Type::BitString(AsnBitString {
                constraint: Some(Constraint {
                    max_value: Some(8),
                    min_value: Some(8),
                    extensible: false
                })
            })
        )
    }

    #[test]
    fn parses_range_constrained_bitstring() {
        let sample = "  BIT STRING -- even here?!?!? -- (SIZE (8 ..18))";
        assert_eq!(
            bit_string(sample).unwrap().1,
            ASN1Type::BitString(AsnBitString {
                constraint: Some(Constraint {
                    max_value: Some(18),
                    min_value: Some(8),
                    extensible: false
                })
            })
        )
    }

    #[test]
    fn parses_strictly_constrained_extended_bitstring() {
        let sample = "  BIT STRING (SIZE (2, ...))";
        assert_eq!(
          bit_string(sample).unwrap().1,
          ASN1Type::BitString(AsnBitString {
              constraint: Some(Constraint {
                  max_value: Some(2),
                  min_value: Some(2),
                  extensible: true
              })
          })
      )
    }

    #[test]
    fn parses_range_constrained_extended_bitstring() {
        let sample = "  BIT STRING (SIZE (8 -- junior dev's comment -- .. 18, ...))";
        assert_eq!(
          bit_string(sample).unwrap().1,
          ASN1Type::BitString(AsnBitString {
              constraint: Some(Constraint {
                  max_value: Some(18),
                  min_value: Some(8),
                  extensible: true
              })
          })
      )
    }
}

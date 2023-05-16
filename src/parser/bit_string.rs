use nom::{
    bytes::complete::tag,
    combinator::{map, opt},
    sequence::{pair, preceded},
    IResult,
};

use crate::grammar::token::{ASN1Type, BIT_STRING, SIZE};

use super::common::*;

pub fn bit_string<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    map(
        preceded(
            skip_ws_and_comments(tag(BIT_STRING)),
            pair(
                opt(distinguished_values),
                opt(in_parentheses(preceded(tag(SIZE), constraint))),
            ),
        ),
        |m| ASN1Type::BitString(m.into()),
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::grammar::token::{ASN1Type, AsnBitString, Constraint, DistinguishedValue};

    use super::bit_string;

    #[test]
    fn parses_unconfined_bitstring() {
        let sample = "  BIT STRING";
        assert_eq!(
            bit_string(sample).unwrap().1,
            ASN1Type::BitString(AsnBitString {
                distinguished_values: None,
                constraint: None
            })
        )
    }

    #[test]
    fn parses_strictly_constrained_bitstring() {
        let sample = "  BIT STRING(SIZE (8))";
        assert_eq!(
            bit_string(sample).unwrap().1,
            ASN1Type::BitString(AsnBitString {
                distinguished_values: None,
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
                distinguished_values: None,
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
                distinguished_values: None,
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
        let sample = "  BIT STRING (SIZE (8 -- comment -- .. 18, ...))";
        assert_eq!(
            bit_string(sample).unwrap().1,
            ASN1Type::BitString(AsnBitString {
                distinguished_values: None,
                constraint: Some(Constraint {
                    max_value: Some(18),
                    min_value: Some(8),
                    extensible: true
                })
            })
        )
    }

    #[test]
    fn parses_bitstring_with_distinguished_values() {
        let sample = r#"BIT STRING {
          heavyLoad    (0),
          excessWidth  (1),  -- this is excessive
          excessLength (2),  -- this, too
          excessHeight (3) -- and this
      } (SIZE(4))"#;
        assert_eq!(
            bit_string(sample).unwrap().1,
            ASN1Type::BitString(AsnBitString {
                distinguished_values: Some(vec![
                    DistinguishedValue {
                        name: "heavyLoad".into(),
                        value: 0
                    },
                    DistinguishedValue {
                        name: "excessWidth".into(),
                        value: 1
                    },
                    DistinguishedValue {
                        name: "excessLength".into(),
                        value: 2
                    },
                    DistinguishedValue {
                        name: "excessHeight".into(),
                        value: 3
                    },
                ]),
                constraint: Some(Constraint {
                    max_value: Some(4),
                    min_value: Some(4),
                    extensible: false
                })
            })
        )
    }
}

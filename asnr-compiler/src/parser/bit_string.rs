use nom::{
    bytes::complete::tag,
    character::complete::{one_of, char},
    combinator::{map, opt},
    multi::fold_many0,
    sequence::{delimited, pair, preceded, terminated},
    IResult,
};

use asnr_grammar::{ASN1Type, ASN1Value, BIT_STRING, SINGLE_QUOTE, SIZE};

use super::common::*;

pub fn bit_string_value<'a>(input: &'a str) -> IResult<&'a str, ASN1Value> {
    map(
        skip_ws_and_comments(terminated(
            delimited(
                char(SINGLE_QUOTE),
                fold_many0(one_of("0123456789ABCDEF"), String::new, |mut acc, curr| {
                    acc.push(curr);
                    acc
                }),
                char(SINGLE_QUOTE),
            ),
            one_of("HB"),
        )),
        |m| ASN1Value::String(m),
    )(input)
}

/// Tries to parse an ASN1 BIT STRING
/// 
/// *`input` - string slice to be matched against
/// 
/// `bit_string` will try to match an BIT STRING declaration in the `input` string.
/// If the match succeeds, the parser will consume the match and return the remaining string
/// and a wrapped `AsnBitString` value representing the ASN1 declaration.
/// If the match fails, the parser will not consume the input and will return an error.
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
    use asnr_grammar::{ASN1Type, AsnBitString, Constraint, DistinguishedValue};

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

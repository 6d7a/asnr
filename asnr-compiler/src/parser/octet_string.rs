use nom::{
    bytes::complete::tag,
    combinator::{map, opt},
    sequence::preceded,
    IResult,
};

use asnr_grammar::{ASN1Type, OCTET_STRING, SIZE};

use super::common::*;

/// Tries to parse an ASN1 OCTET STRING
/// 
/// *`input` - string slice to be matched against
/// 
/// `octet_string` will try to match an OCTET STRING declaration in the `input` string.
/// If the match succeeds, the parser will consume the match and return the remaining string
/// and a wrapped `AsnOctetString` value representing the ASN1 declaration.
/// If the match fails, the parser will not consume the input and will return an error.
pub fn octet_string<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    map(
        preceded(
            skip_ws_and_comments(tag(OCTET_STRING)),
            opt(in_parentheses(preceded(tag(SIZE), constraint))),
        ),
        |m| ASN1Type::OctetString(m.into()),
    )(input)
}

#[cfg(test)]
mod tests {
    use asnr_grammar::{ASN1Type, Constraint, AsnOctetString};

    use super::octet_string;

    #[test]
    fn parses_unconfined_octetstring() {
        let sample = "  OCTET STRING";
        assert_eq!(
            octet_string(sample).unwrap().1,
            ASN1Type::OctetString(AsnOctetString { constraint: None })
        )
    }

    #[test]
    fn parses_strictly_constrained_octetstring() {
        let sample = "  OCTET STRING(SIZE (8))";
        assert_eq!(
            octet_string(sample).unwrap().1,
            ASN1Type::OctetString(AsnOctetString {
                constraint: Some(Constraint {
                    max_value: Some(8),
                    min_value: Some(8),
                    extensible: false
                })
            })
        )
    }

    #[test]
    fn parses_range_constrained_octetstring() {
        let sample = "  OCTET STRING -- even here?!?!? -- (SIZE (8 ..18))";
        assert_eq!(
            octet_string(sample).unwrap().1,
            ASN1Type::OctetString(AsnOctetString {
                constraint: Some(Constraint {
                    max_value: Some(18),
                    min_value: Some(8),
                    extensible: false
                })
            })
        )
    }

    #[test]
    fn parses_strictly_constrained_extended_octetstring() {
        let sample = r#"  OCTET STRING 
        (SIZE (2, ...))"#;
        assert_eq!(
          octet_string(sample).unwrap().1,
          ASN1Type::OctetString(AsnOctetString {
              constraint: Some(Constraint {
                  max_value: Some(2),
                  min_value: Some(2),
                  extensible: true
              })
          })
      )
    }

    #[test]
    fn parses_range_constrained_extended_octetstring() {
        let sample = "  OCTET STRING (SIZE (8 -- junior dev's comment -- .. 18, ...))";
        assert_eq!(
          octet_string(sample).unwrap().1,
          ASN1Type::OctetString(AsnOctetString {
              constraint: Some(Constraint {
                  max_value: Some(18),
                  min_value: Some(8),
                  extensible: true
              })
          })
      )
    }
}

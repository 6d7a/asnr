use nom::{
    bytes::complete::tag,
    combinator::{map, opt},
    sequence::{preceded, pair},
    IResult, branch::alt,
};

use asnr_grammar::*;

use super::common::*;

/// Tries to parse an ASN1 Character String type
/// 
/// *`input` - string slice to be matched against
/// 
/// `character_string` will try to match an Character String type declaration in the `input`
/// string, i.e. ASN1 types such as IA5String, UTF8String, VideotexString, but also 
/// OCTET STRING, which is treated like a String and not a buffer.
/// If the match succeeds, the parser will consume the match and return the remaining string
/// and a wrapped `AsnCharacterString` value representing the ASN1 declaration.
/// If the match fails, the parser will not consume the input and will return an error.
pub fn character_string<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    map(
        pair(
            skip_ws_and_comments(alt((
              tag(OCTET_STRING),
              tag(IA5_STRING),
              tag(UTF8_STRING),
              tag(NUMERIC_STRING),
              tag(VISIBLE_STRING),
              tag(TELETEX_STRING),
              tag(VIDEOTEX_STRING),
              tag(GRAPHIC_STRING),
              tag(GENERAL_STRING),
              tag(UNIVERSAL_STRING),
              tag(BMP_STRING),
              tag(PRINTABLE_STRING)
            ))),
            opt(in_parentheses(preceded(tag(SIZE), constraint))),
        ),
        |m| ASN1Type::CharacterString(m.into()),
    )(input)
}

#[cfg(test)]
mod tests {
    use asnr_grammar::{ASN1Type, Constraint, AsnCharacterString, CharacterStringType};

    use super::character_string;

    #[test]
    fn parses_unconfined_characterstring() {
        let sample = "  OCTET STRING";
        assert_eq!(
            character_string(sample).unwrap().1,
            ASN1Type::CharacterString(AsnCharacterString { constraint: None,
            r#type: CharacterStringType::OctetString})
        )
    }

    #[test]
    fn parses_strictly_constrained_characterstring() {
        let sample = "  OCTET STRING(SIZE (8))";
        assert_eq!(
            character_string(sample).unwrap().1,
            ASN1Type::CharacterString(AsnCharacterString {
                constraint: Some(Constraint {
                    max_value: Some(8),
                    min_value: Some(8),
                    extensible: false
                }),
                r#type: CharacterStringType::OctetString
            })
        )
    }

    #[test]
    fn parses_range_constrained_characterstring() {
        let sample = "  OCTET STRING -- even here?!?!? -- (SIZE (8 ..18))";
        assert_eq!(
            character_string(sample).unwrap().1,
            ASN1Type::CharacterString(AsnCharacterString {
                constraint: Some(Constraint {
                    max_value: Some(18),
                    min_value: Some(8),
                    extensible: false
                }),
                r#type: CharacterStringType::OctetString
            })
        )
    }

    #[test]
    fn parses_strictly_constrained_extended_characterstring() {
        let sample = r#"  OCTET STRING 
        (SIZE (2, ...))"#;
        assert_eq!(
          character_string(sample).unwrap().1,
          ASN1Type::CharacterString(AsnCharacterString {
              constraint: Some(Constraint {
                  max_value: Some(2),
                  min_value: Some(2),
                  extensible: true
              }),
              r#type: CharacterStringType::OctetString
          })
      )
    }

    #[test]
    fn parses_range_constrained_extended_characterstring() {
        let sample = "  OCTET STRING (SIZE (8 -- junior dev's comment -- .. 18, ...))";
        assert_eq!(
          character_string(sample).unwrap().1,
          ASN1Type::CharacterString(AsnCharacterString {
              constraint: Some(Constraint {
                  max_value: Some(18),
                  min_value: Some(8),
                  extensible: true
              }),
              r#type: CharacterStringType::OctetString
          })
      )
    }
}

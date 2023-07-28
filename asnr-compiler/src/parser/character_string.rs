use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::streaming::char,
    combinator::{map, opt},
    multi::many1,
    sequence::{delimited, pair},
    IResult,
};

use asnr_grammar::*;

use super::{common::*, constraint::constraint};

pub fn character_string_value<'a>(input: &'a str) -> IResult<&'a str, ASN1Value> {
    map(
        delimited(tag("\""), take_until("\""), tag("\"")),
        |m: &str| ASN1Value::String(m.to_owned()),
    )(input)
}

/// Tries to parse an ASN1 Character String type
///
/// *`input` - string slice to be matched against
///
/// `character_string` will try to match an Character String type declaration in the `input`
/// string, i.e. ASN1 types such as IA5String, UTF8String, VideotexString, but also
/// OCTET STRING, which is treated like a String and not a buffer.
/// If the match succeeds, the parser will consume the match and return the remaining string
/// and a wrapped `CharacterString` value representing the ASN1 declaration.
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
                tag(PRINTABLE_STRING),
            ))),
            opt(constraint),
        ),
        |m| ASN1Type::CharacterString(m.into()),
    )(input)
}

#[cfg(test)]
mod tests {
    use asnr_grammar::{constraints::*, types::*, *};

    use crate::parser::{character_string::character_string_value, asn1_value};

    use super::character_string;

    #[test]
    fn parses_unconfined_characterstring() {
        let sample = "  OCTET STRING";
        assert_eq!(
            character_string(sample).unwrap().1,
            ASN1Type::CharacterString(CharacterString {
                constraints: vec![],
                r#type: CharacterStringType::OctetString
            })
        )
    }

    #[test]
    fn parses_strictly_constrained_characterstring() {
        let sample = "  OCTET STRING(SIZE (8))";
        assert_eq!(
            character_string(sample).unwrap().1,
            ASN1Type::CharacterString(CharacterString {
                constraints: vec![Constraint::SubtypeConstraint(ElementSet {
                    set: ElementOrSetOperation::Element(SubtypeElement::SizeConstraint(Box::new(
                        ElementOrSetOperation::Element(SubtypeElement::SingleValue {
                            value: ASN1Value::Integer(8),
                            extensible: false
                        })
                    ))),
                    extensible: false
                })],
                r#type: CharacterStringType::OctetString
            })
        )
    }

    #[test]
    fn parses_range_constrained_characterstring() {
        let sample = "  OCTET STRING -- even here?!?!? -- (SIZE (8 ..18))";
        assert_eq!(
            character_string(sample).unwrap().1,
            ASN1Type::CharacterString(CharacterString {
                constraints: vec![Constraint::SubtypeConstraint(ElementSet {
                    set: ElementOrSetOperation::Element(SubtypeElement::SizeConstraint(Box::new(
                        ElementOrSetOperation::Element(SubtypeElement::ValueRange {
                            min: Some(ASN1Value::Integer(8)),
                            max: Some(ASN1Value::Integer(18)),
                            extensible: false
                        })
                    ))),
                    extensible: false
                })],
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
            ASN1Type::CharacterString(CharacterString {
                constraints: vec![Constraint::SubtypeConstraint(ElementSet {
                    set: ElementOrSetOperation::Element(SubtypeElement::SizeConstraint(Box::new(
                        ElementOrSetOperation::Element(SubtypeElement::SingleValue {
                            value: ASN1Value::Integer(2),
                            extensible: true
                        })
                    ))),
                    extensible: false
                })],
                r#type: CharacterStringType::OctetString
            })
        )
    }

    #[test]
    fn parses_range_constrained_extended_characterstring() {
        let sample = "  OCTET STRING (SIZE (8 --  comment -- .. 18, ...))";
        assert_eq!(
            character_string(sample).unwrap().1,
            ASN1Type::CharacterString(CharacterString {
                constraints: vec![Constraint::SubtypeConstraint(ElementSet {
                    set: ElementOrSetOperation::Element(SubtypeElement::SizeConstraint(Box::new(
                        ElementOrSetOperation::Element(SubtypeElement::ValueRange {
                            min: Some(ASN1Value::Integer(8)),
                            max: Some(ASN1Value::Integer(18)),
                            extensible: true
                        })
                    ))),
                    extensible: false
                })],
                r#type: CharacterStringType::OctetString
            })
        )
    }

    #[test]
    fn parses_character_string_value() {
      assert_eq!(character_string_value("\"a\"").unwrap().1, ASN1Value::String("a".to_owned()))
    }

    #[test]
    fn parses_character_string_asn1_value() {
      assert_eq!(asn1_value("\"a\"").unwrap().1, ASN1Value::String("a".to_owned()))
    }
}

use asnr_grammar::{ASN1Type, OF, SEQUENCE, SIZE};
use nom::{
    bytes::complete::tag,
    combinator::{map, opt},
    sequence::{pair, preceded},
    IResult,
};

use super::{
    asn1_type,
    common::{constraint, in_parentheses, skip_ws_and_comments},
};

/// Tries to parse an ASN1 SEQUENCE OF
///
/// *`input` - string slice to be matched against
///
/// `sequence_of` will try to match an SEQUENCE OF declaration in the `input` string.
/// If the match succeeds, the parser will consume the match and return the remaining string
/// and a wrapped `AsnSequenceOf` value representing the ASN1 declaration.
/// If the match fails, the parser will not consume the input and will return an error.
pub fn sequence_of<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    map(
        pair(
            preceded(
                skip_ws_and_comments(tag(SEQUENCE)),
                opt(preceded(
                    skip_ws_and_comments(tag(SIZE)),
                    skip_ws_and_comments(constraint),
                )),
            ),
            preceded(skip_ws_and_comments(tag(OF)), asn1_type),
        ),
        |m| ASN1Type::SequenceOf(m.into()),
    )(input)
}

#[cfg(test)]
mod tests {
    use asnr_grammar::{
        ASN1Type, AsnInteger, AsnSequenceOf, Constraint, DeclarationElsewhere, DistinguishedValue,
    };

    use crate::parser::sequence_of;

    #[test]
    fn parses_simple_sequence_of() {
        assert_eq!(
            sequence_of("SEQUENCE OF BOOLEAN").unwrap().1,
            ASN1Type::SequenceOf(AsnSequenceOf {
                constraint: None,
                r#type: Box::new(ASN1Type::Boolean)
            })
        );
    }

    #[test]
    fn parses_simple_sequence_of_elsewhere_declared_type() {
        assert_eq!(
            sequence_of("SEQUENCE OF Things").unwrap().1,
            ASN1Type::SequenceOf(AsnSequenceOf {
                constraint: None,
                r#type: Box::new(ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere(
                    "Things".into()
                )))
            })
        );
    }

    #[test]
    fn parses_constraint_sequence_of_elsewhere_declared_type() {
        assert_eq!(
            sequence_of("SEQUENCE SIZE (1..13,...) OF CorrelationCellValue  ")
                .unwrap()
                .1,
            ASN1Type::SequenceOf(AsnSequenceOf {
                constraint: Some(Constraint {
                    min_value: Some(1),
                    max_value: Some(13),
                    extensible: true
                }),
                r#type: Box::new(ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere(
                    "CorrelationCellValue".into()
                )))
            })
        );
    }

    #[test]
    fn parses_constraint_sequence_of_constraint_integer() {
        assert_eq!(
            sequence_of(
                r#"SEQUENCE SIZE (1..13,...) OF INTEGER {
              one-distinguished-value (12)
            } (1..13,...) "#
            )
            .unwrap()
            .1,
            ASN1Type::SequenceOf(AsnSequenceOf {
                constraint: Some(Constraint {
                    min_value: Some(1),
                    max_value: Some(13),
                    extensible: true
                }),
                r#type: Box::new(ASN1Type::Integer(AsnInteger {
                    constraint: Some(Constraint {
                        min_value: Some(1),
                        max_value: Some(13),
                        extensible: true
                    }),
                    distinguished_values: Some(vec![DistinguishedValue {
                        name: "one-distinguished-value".into(),
                        value: 12
                    }])
                }))
            })
        );
    }
}

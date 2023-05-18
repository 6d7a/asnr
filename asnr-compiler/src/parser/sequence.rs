use nom::{
    bytes::complete::tag,
    character::complete::char,
    combinator::{into, opt},
    multi::many0,
    sequence::{pair, terminated, tuple},
    IResult,
};

use crate::grammar::token::{OptionalMarker, SequenceMember, COMMA, DEFAULT, OPTIONAL, SEQUENCE};

use super::*;

pub fn sequence<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    map(
        preceded(
            skip_ws_and_comments(tag(SEQUENCE)),
            in_braces(pair(
                many0(terminated(
                    skip_ws_and_comments(sequence_member),
                    skip_ws_and_comments(opt(char(COMMA))),
                )),
                opt(extension_marker),
            )),
        ),
        |m| ASN1Type::Sequence(m.into()),
    )(input)
}

fn sequence_member<'a>(input: &'a str) -> IResult<&'a str, SequenceMember> {
    into(tuple((
        skip_ws_and_comments(identifier),
        skip_ws_and_comments(asn1_type),
        optional_marker,
        default,
    )))(input)
}

fn optional_marker<'a>(input: &'a str) -> IResult<&'a str, Option<OptionalMarker>> {
    opt(into(skip_ws_and_comments(tag(OPTIONAL))))(input)
}

fn default<'a>(input: &'a str) -> IResult<&'a str, Option<ASN1Value>> {
    opt(preceded(
        skip_ws_and_comments(tag(DEFAULT)),
        skip_ws_and_comments(asn1_value),
    ))(input)
}

#[cfg(test)]
mod tests {
    use crate::grammar::token::{
        AsnBitString, AsnInteger, AsnOctetString, AsnSequence, Constraint, DeclarationElsewhere,
    };

    use super::*;

    #[test]
    fn parses_optional_marker() {
        assert_eq!(
            optional_marker("\n\tOPTIONAL").unwrap().1,
            Some(OptionalMarker())
        );
        assert_eq!(optional_marker("DEFAULT").unwrap().1, None);
    }

    #[test]
    fn parses_default_int() {
        assert_eq!(
            default("\n\tDEFAULT\t-1").unwrap().1,
            Some(ASN1Value::Integer(-1))
        );
    }

    #[test]
    fn parses_default_boolean() {
        assert_eq!(
            default("  DEFAULT   TRUE").unwrap().1,
            Some(ASN1Value::Boolean(true))
        );
    }

    #[test]
    fn parses_default_bitstring() {
        assert_eq!(
            default("  DEFAULT '001010011'B").unwrap().1,
            Some(ASN1Value::String("001010011".into()))
        );
        assert_eq!(
            default("DEFAULT 'F60E'H").unwrap().1,
            Some(ASN1Value::String("F60E".into()))
        );
    }

    #[test]
    fn parses_default_enumeral() {
        assert_eq!(
            default("  DEFAULT enumeral1").unwrap().1,
            Some(ASN1Value::String("enumeral1".into()))
        );
        assert_eq!(
            default("DEFAULT enumeral1").unwrap().1,
            Some(ASN1Value::String("enumeral1".into()))
        );
    }

    #[test]
    fn parses_simple_sequence() {
        assert_eq!(
            sequence(
                r#"SEQUENCE {
        value         AccelerationValue,
        confidence    AccelerationConfidence
    }"#
            )
            .unwrap()
            .1,
            ASN1Type::Sequence(AsnSequence {
                extensible: false,
                members: vec![
                    SequenceMember {
                        name: "value".into(),
                        r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere(
                            "AccelerationValue".into()
                        )),
                        default_value: None,
                        is_optional: false
                    },
                    SequenceMember {
                        name: "confidence".into(),
                        r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere(
                            "AccelerationConfidence".into()
                        )),
                        default_value: None,
                        is_optional: false
                    }
                ]
            })
        )
    }

    #[test]
    fn parses_sequence_with_optionals() {
        assert_eq!(
            sequence(
                r#"SEQUENCE{    
                  xCoordinate    CartesianCoordinateWithConfidence, 
                  --x
                  yCoordinate    CartesianCoordinateWithConfidence, -- y -- 
                  zCoordinate    CartesianCoordinateWithConfidence OPTIONAL -- this is optional
              }"#
            )
            .unwrap()
            .1,
            ASN1Type::Sequence(AsnSequence {
                extensible: false,
                members: vec![
                    SequenceMember {
                        name: "xCoordinate".into(),
                        r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere(
                            "CartesianCoordinateWithConfidence".into()
                        )),
                        default_value: None,
                        is_optional: false
                    },
                    SequenceMember {
                        name: "yCoordinate".into(),
                        r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere(
                            "CartesianCoordinateWithConfidence".into()
                        )),
                        default_value: None,
                        is_optional: false
                    },
                    SequenceMember {
                        name: "zCoordinate".into(),
                        r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere(
                            "CartesianCoordinateWithConfidence".into()
                        )),
                        default_value: None,
                        is_optional: true
                    }
                ]
            })
        )
    }

    #[test]
    fn parses_extended_sequence_with_default() {
        assert_eq!(
            sequence(
                r#"SEQUENCE {
                  horizontalPositionConfidence  PosConfidenceEllipse OPTIONAL,   
                  deltaAltitude -- COMMENT --   DeltaAltitude DEFAULT unavailable, 
                  altitudeConfidence            AltitudeConfidence DEFAULT unavailable,
                  -- Attention: Extension!
                  ... 
                }"#
            )
            .unwrap()
            .1,
            ASN1Type::Sequence(AsnSequence {
                extensible: true,
                members: vec![
                    SequenceMember {
                        name: "horizontalPositionConfidence".into(),
                        r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere(
                            "PosConfidenceEllipse".into()
                        )),
                        default_value: None,
                        is_optional: true
                    },
                    SequenceMember {
                        name: "deltaAltitude".into(),
                        r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere(
                            "DeltaAltitude".into()
                        )),
                        default_value: Some(ASN1Value::String("unavailable".into())),
                        is_optional: true
                    },
                    SequenceMember {
                        name: "altitudeConfidence".into(),
                        r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere(
                            "AltitudeConfidence".into()
                        )),
                        default_value: Some(ASN1Value::String("unavailable".into())),
                        is_optional: true
                    }
                ]
            })
        )
    }

    #[test]
    fn parses_sequence_with_primitives() {
        assert_eq!(
            sequence(
                r#"SEQUENCE {
                  unNumber                INTEGER (0..9999),
                  limitedQuantity         BOOLEAN DEFAULT FALSE,
                  emergencyActionCode     OCTET STRING (SIZE (1..24)) OPTIONAL,
                  ...
              }"#
            )
            .unwrap()
            .1,
            ASN1Type::Sequence(AsnSequence {
                extensible: true,
                members: vec![
                    SequenceMember {
                        name: "unNumber".into(),
                        r#type: ASN1Type::Integer(AsnInteger {
                            constraint: Some(Constraint {
                                min_value: Some(0),
                                max_value: Some(9999),
                                extensible: false
                            }),
                            distinguished_values: None
                        }),
                        default_value: None,
                        is_optional: false
                    },
                    SequenceMember {
                        name: "limitedQuantity".into(),
                        r#type: ASN1Type::Boolean,
                        default_value: Some(ASN1Value::Boolean(false)),
                        is_optional: true
                    },
                    SequenceMember {
                        name: "emergencyActionCode".into(),
                        r#type: ASN1Type::OctetString(AsnOctetString {
                            constraint: Some(Constraint {
                                min_value: Some(1),
                                max_value: Some(24),
                                extensible: false
                            })
                        }),
                        default_value: None,
                        is_optional: true
                    }
                ]
            })
        )
    }

    #[test]
    fn parses_nested_sequence() {
        assert_eq!(
            sequence(
                r#"SEQUENCE {
                  nested                SEQUENCE {
                    wow         Wow -- WOW!
                    this-is-annoying BOOLEAN DEFAULT TRUE,
                    another 
                    SEQUENCE
                    {
                      inner BIT STRING (SIZE(1,...)) DEFAULT '0'B
                    } OPTIONAL,
                    ...
                  },
                  ...
              }"#
            )
            .unwrap()
            .1,
            ASN1Type::Sequence(AsnSequence {
                extensible: true,
                members: vec![SequenceMember {
                    name: "nested".into(),
                    r#type: ASN1Type::Sequence(AsnSequence {
                        extensible: true,
                        members: vec![
                            SequenceMember {
                                name: "wow".into(),
                                r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere(
                                    "Wow".into()
                                )),
                                default_value: None,
                                is_optional: false
                            },
                            SequenceMember {
                                name: "this-is-annoying".into(),
                                r#type: ASN1Type::Boolean,
                                default_value: Some(ASN1Value::Boolean(true)),
                                is_optional: true
                            },
                            SequenceMember {
                                name: "another".into(),
                                r#type: ASN1Type::Sequence(AsnSequence {
                                    extensible: false,
                                    members: vec![SequenceMember {
                                        name: "inner".into(),
                                        r#type: ASN1Type::BitString(AsnBitString {
                                            constraint: Some(Constraint {
                                                min_value: Some(1),
                                                max_value: Some(1),
                                                extensible: true
                                            }),
                                            distinguished_values: None
                                        }),
                                        default_value: Some(ASN1Value::String("0".into())),
                                        is_optional: true
                                    }]
                                }),
                                default_value: None,
                                is_optional: true
                            }
                        ]
                    }),
                    default_value: None,
                    is_optional: false
                }]
            })
        )
    }
}

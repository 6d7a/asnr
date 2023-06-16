use nom::{
    bytes::complete::tag,
    character::complete::char,
    combinator::{into, opt, value},
    multi::many0,
    sequence::{terminated, tuple},
    IResult,
};

use asnr_grammar::{subtyping::*, types::*, *};

use super::{
    constraint::{component_constraint, constraint},
    *,
};

/// Tries to parse an ASN1 SEQUENCE
///
/// *`input` - string slice to be matched against
///
/// `sequence` will try to match an SEQUENCE declaration in the `input` string.
/// If the match succeeds, the parser will consume the match and return the remaining string
/// and a wrapped `AsnSequence` value representing the ASN1 declaration. If the defined SEQUENCE
/// contains anonymous SEQUENCEs as members, these nested SEQUENCEs will be represented as
/// structs within the same global scope.
/// If the match fails, the parser will not consume the input and will return an error.
pub fn sequence<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    map(
        preceded(
            skip_ws_and_comments(tag(SEQUENCE)),
            pair(
                in_braces(tuple((
                    many0(terminated(
                        skip_ws_and_comments(sequence_member),
                        skip_ws_and_comments(opt(char(COMMA))),
                    )),
                    opt(terminated(extension_marker, opt(char(COMMA)))),
                    opt(many0(terminated(
                        skip_ws_and_comments(sequence_member),
                        skip_ws_and_comments(opt(char(COMMA))),
                    ))),
                ))),
                opt(constraint),
            ),
        ),
        |m| ASN1Type::Sequence(m.into()),
    )(input)
}

fn sequence_member<'a>(input: &'a str) -> IResult<&'a str, SequenceMember> {
    into(tuple((
        skip_ws_and_comments(identifier),
        skip_ws_and_comments(asn1_type),
        opt(component_constraint),
        optional_marker,
        default,
    )))(input)
}

fn optional_marker<'a>(input: &'a str) -> IResult<&'a str, Option<OptionalMarker>> {
    opt(into(skip_ws_and_comments(tag(OPTIONAL))))(input)
}

fn subtype_marker<'a>(input: &'a str) -> IResult<&'a str, ()> {
    value((), opt(skip_ws_and_comments(tag(OPTIONAL))))(input)
}

fn default<'a>(input: &'a str) -> IResult<&'a str, Option<ASN1Value>> {
    opt(preceded(
        skip_ws_and_comments(tag(DEFAULT)),
        skip_ws_and_comments(asn1_value),
    ))(input)
}

#[cfg(test)]
mod tests {
    use std::vec;

    use asnr_grammar::{subtyping::*, types::*, *};

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
    fn parses_subtyped_sequence() {
        assert_eq!(
        sequence(
            r#"SEQUENCE { 
              clusterBoundingBoxShape    Shape (WITH COMPONENTS{..., elliptical ABSENT, radial ABSENT, radialShapes ABSENT}) OPTIONAL,
              ...
           }"#
        )
        .unwrap()
        .1,
        ASN1Type::Sequence(AsnSequence {
            extensible: Some(1),
            constraints: vec![],
            members: vec![
                SequenceMember {
                    name: "clusterBoundingBoxShape".into(),
                    r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere {
                      identifier:
                        "Shape".into(),
                        constraints: vec![]
                }),
                    default_value: None,
                    is_optional: true,
                    constraints: vec![],
                }
            ]
        })
    )
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
                extensible: None,
                constraints: vec![],
                members: vec![
                    SequenceMember {
                        name: "value".into(),
                        r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere {
                            identifier: "AccelerationValue".into(),
                            constraints: vec![]
                        }),
                        default_value: None,
                        is_optional: false,
                        constraints: vec![]
                    },
                    SequenceMember {
                        name: "confidence".into(),
                        r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere {
                            identifier: "AccelerationConfidence".into(),
                            constraints: vec![]
                        }),
                        default_value: None,
                        is_optional: false,
                        constraints: vec![],
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
                extensible: None,
                constraints: vec![],
                members: vec![
                    SequenceMember {
                        name: "xCoordinate".into(),
                        r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere {
                            identifier: "CartesianCoordinateWithConfidence".into(),
                            constraints: vec![]
                        }),
                        default_value: None,
                        is_optional: false,
                        constraints: vec![],
                    },
                    SequenceMember {
                        name: "yCoordinate".into(),
                        r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere {
                            identifier: "CartesianCoordinateWithConfidence".into(),
                            constraints: vec![]
                        }),
                        default_value: None,
                        is_optional: false,
                        constraints: vec![],
                    },
                    SequenceMember {
                        name: "zCoordinate".into(),
                        r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere {
                            identifier: "CartesianCoordinateWithConfidence".into(),
                            constraints: vec![]
                        }),
                        default_value: None,
                        is_optional: true,
                        constraints: vec![],
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
                extensible: Some(3),
                constraints: vec![],
                members: vec![
                    SequenceMember {
                        name: "horizontalPositionConfidence".into(),
                        r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere {
                            identifier: "PosConfidenceEllipse".into(),
                            constraints: vec![]
                        }),
                        default_value: None,
                        is_optional: true,
                        constraints: vec![],
                    },
                    SequenceMember {
                        name: "deltaAltitude".into(),
                        r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere {
                            identifier: "DeltaAltitude".into(),
                            constraints: vec![]
                        }),
                        default_value: Some(ASN1Value::String("unavailable".into())),
                        is_optional: true,
                        constraints: vec![],
                    },
                    SequenceMember {
                        name: "altitudeConfidence".into(),
                        r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere {
                            identifier: "AltitudeConfidence".into(),
                            constraints: vec![]
                        }),
                        default_value: Some(ASN1Value::String("unavailable".into())),
                        is_optional: true,
                        constraints: vec![],
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
                extensible: Some(3),
                constraints: vec![],
                members: vec![
                    SequenceMember {
                        name: "unNumber".into(),
                        r#type: ASN1Type::Integer(AsnInteger {
                            constraints: vec![RangeConstraint {
                                min_value: Some(ASN1Value::Integer(0)),
                                max_value: Some(ASN1Value::Integer(9999)),
                                extensible: false
                            }],
                            distinguished_values: None
                        }),
                        default_value: None,
                        is_optional: false,
                        constraints: vec![],
                    },
                    SequenceMember {
                        name: "limitedQuantity".into(),
                        r#type: ASN1Type::Boolean,
                        default_value: Some(ASN1Value::Boolean(false)),
                        is_optional: true,
                        constraints: vec![],
                    },
                    SequenceMember {
                        name: "emergencyActionCode".into(),
                        r#type: ASN1Type::CharacterString(AsnCharacterString {
                            constraints: vec![Constraint::RangeConstraint(RangeConstraint {
                                min_value: Some(ASN1Value::Integer(1)),
                                max_value: Some(ASN1Value::Integer(24)),
                                extensible: false
                            })],
                            r#type: asnr_grammar::CharacterStringType::OctetString
                        }),
                        default_value: None,
                        is_optional: true,
                        constraints: vec![],
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
                extensible: Some(1),
                constraints: vec![],
                members: vec![SequenceMember {
                    name: "nested".into(),
                    r#type: ASN1Type::Sequence(AsnSequence {
                        extensible: Some(3),
                        constraints: vec![],
                        members: vec![
                            SequenceMember {
                                name: "wow".into(),
                                r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere {
                                    identifier: "Wow".into(),
                                    constraints: vec![]
                                }),
                                default_value: None,
                                is_optional: false,
                                constraints: vec![],
                            },
                            SequenceMember {
                                name: "this-is-annoying".into(),
                                r#type: ASN1Type::Boolean,
                                default_value: Some(ASN1Value::Boolean(true)),
                                is_optional: true,
                                constraints: vec![],
                            },
                            SequenceMember {
                                name: "another".into(),
                                r#type: ASN1Type::Sequence(AsnSequence {
                                    extensible: None,
                                    constraints: vec![],
                                    members: vec![SequenceMember {
                                        name: "inner".into(),
                                        r#type: ASN1Type::BitString(AsnBitString {
                                            constraints: vec![RangeConstraint {
                                                min_value: Some(ASN1Value::Integer(1)),
                                                max_value: Some(ASN1Value::Integer(1)),
                                                extensible: true
                                            }],
                                            distinguished_values: None
                                        }),
                                        default_value: Some(ASN1Value::String("0".into())),
                                        is_optional: true,
                                        constraints: vec![],
                                    }]
                                }),
                                default_value: None,
                                is_optional: true,
                                constraints: vec![],
                            }
                        ]
                    }),
                    default_value: None,
                    is_optional: false,
                    constraints: vec![],
                }]
            })
        )
    }
}

//! The `parser` module contains the parser combinator
//! responsible for interpreting the input as ASN1 notation.
//! The parser is made up of a number of sub-parsers that
//! interpret single elements of ASN1 syntax.SS
//!
//! The `parser` submodules provide parsers for their
//! respective eponymous ASN1 type, with the exception
//! of `common`, which contains parsers for the more
//! generic elements of ASN1 syntax, and `util`, which
//! contains helper parsers not specific to ASN1's notation.
use nom::{
    branch::alt,
    combinator::{into, map, opt},
    multi::{many0, many1},
    sequence::{pair, preceded, tuple, terminated},
    IResult, bytes::complete::tag,
};

use asnr_grammar::{information_object::*, *};

use self::{
    bit_string::{bit_string, bit_string_value},
    boolean::{boolean, boolean_value},
    character_string::character_string,
    choice::*,
    common::*,
    constraint::constraint,
    enumerated::*,
    error::ParserError,
    information_object_class::*,
    integer::*,
    module_reference::module_reference,
    null::*,
    parameterization::parameterization,
    sequence::sequence,
    sequence_of::*,
};

mod bit_string;
mod boolean;
mod character_string;
mod choice;
mod common;
mod constraint;
mod enumerated;
mod error;
mod information_object_class;
mod integer;
mod module_reference;
mod null;
mod object_identifier;
mod parameterization;
mod sequence;
mod sequence_of;
mod util;

pub fn asn_spec<'a>(
    input: &'a str,
) -> Result<Vec<(ModuleReference, Vec<ToplevelDeclaration>)>, ParserError> {
    many1(pair(
        module_reference,
        terminated(
            many0(skip_ws(alt((
                map(top_level_information_declaration, |m| {
                    ToplevelDeclaration::Information(m)
                }),
                map(top_level_type_declaration, |m| ToplevelDeclaration::Type(m)),
                map(top_level_value_declaration, |m| {
                    ToplevelDeclaration::Value(m)
                }),
            )))),
            skip_ws_and_comments(tag(END)),
        ),
    ))(input)
    .map(|(_, res)| res)
    .map_err(|e| e.into())
}

pub fn top_level_type_declaration<'a>(input: &'a str) -> IResult<&'a str, ToplevelTypeDeclaration> {
    into(tuple((
        skip_ws(many0(comment)),
        skip_ws(type_identifier),
        opt(parameterization),
        preceded(assignment, asn1_type),
    )))(input)
}

pub fn top_level_information_declaration<'a>(
    input: &'a str,
) -> IResult<&'a str, ToplevelInformationDeclaration> {
    skip_ws(alt((
        top_level_information_object_declaration,
        top_level_object_set_declaration,
        top_level_object_class_declaration,
    )))(input)
}

pub fn asn1_type<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    alt((
        null,
        sequence_of,
        sequence,
        choice,
        integer,
        enumerated,
        boolean,
        bit_string,
        character_string,
        map(information_object_field_reference, |i| {
            ASN1Type::InformationObjectFieldReference(i)
        }),
        elsewhere_declared_type,
    ))(input)
}

pub fn asn1_value<'a>(input: &'a str) -> IResult<&'a str, ASN1Value> {
    alt((
        null_value,
        bit_string_value,
        boolean_value,
        integer_value,
        elsewhere_declared_value,
    ))(input)
}

pub fn elsewhere_declared_value<'a>(input: &'a str) -> IResult<&'a str, ASN1Value> {
    map(skip_ws_and_comments(value_identifier), |m| {
        ASN1Value::ElsewhereDeclaredValue(m.into())
    })(input)
}

pub fn elsewhere_declared_type<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    map(
        pair(
            skip_ws_and_comments(type_identifier),
            opt(skip_ws_and_comments(constraint)),
        ),
        |m| ASN1Type::ElsewhereDeclaredType(m.into()),
    )(input)
}

fn top_level_value_declaration<'a>(input: &'a str) -> IResult<&'a str, ToplevelValueDeclaration> {
    into(tuple((
        skip_ws(many0(comment)),
        skip_ws(value_identifier),
        skip_ws(identifier),
        preceded(assignment, asn1_value),
    )))(input)
}

fn top_level_information_object_declaration<'a>(
    input: &'a str,
) -> IResult<&'a str, ToplevelInformationDeclaration> {
    into(tuple((
        skip_ws(many0(comment)),
        skip_ws(identifier),
        skip_ws(uppercase_identifier),
        preceded(assignment, information_object),
    )))(input)
}

fn top_level_object_set_declaration<'a>(
    input: &'a str,
) -> IResult<&'a str, ToplevelInformationDeclaration> {
    into(tuple((
        skip_ws(many0(comment)),
        skip_ws(identifier),
        skip_ws(uppercase_identifier),
        preceded(assignment, object_set),
    )))(input)
}

fn top_level_object_class_declaration<'a>(
    input: &'a str,
) -> IResult<&'a str, ToplevelInformationDeclaration> {
    into(tuple((
        skip_ws(many0(comment)),
        skip_ws(uppercase_identifier),
        preceded(assignment, information_object_class),
    )))(input)
}

#[cfg(test)]
mod tests {
    use core::panic;
    use std::vec;

    use asnr_grammar::{information_object::*, subtyping::*, types::*, *};

    use crate::parser::top_level_information_declaration;

    use super::top_level_type_declaration;

    #[test]
    fn parses_toplevel_simple_integer_declaration() {
        let tld = top_level_type_declaration(
            "/**
          * The DE represents a cardinal number that counts the size of a set. 
          * 
          * @category: Basic information
          * @revision: Created in V2.1.1
         */
         CardinalNumber3b ::= INTEGER(1..8)",
        )
        .unwrap()
        .1;
        assert_eq!(tld.name, String::from("CardinalNumber3b"));
        assert!(tld.comments.contains("@revision: Created in V2.1.1"));
        if let ASN1Type::Integer(int) = tld.r#type {
            assert!(!int.constraints.is_empty());
            assert_eq!(
                int.constraints.first().unwrap().min_value,
                Some(ASN1Value::Integer(1))
            );
            assert_eq!(
                int.constraints.first().unwrap().max_value,
                Some(ASN1Value::Integer(8))
            );
            assert_eq!(int.constraints.first().unwrap().extensible, false);
        } else {
            panic!("Top-level declaration contains other type than integer.")
        }
    }

    #[test]
    fn parses_toplevel_macro_integer_declaration() {
        let tld = top_level_type_declaration(r#"/** 
        * This DE represents the magnitude of the acceleration vector in a defined coordinate system.
        *
        * The value shall be set to:
        * - `0` to indicate no acceleration,
        * - `n` (`n > 0` and `n < 160`) to indicate acceleration equal to or less than n x 0,1 m/s^2, and greater than (n-1) x 0,1 m/s^2,
        * - `160` for acceleration values greater than 15,9 m/s^2,
        * - `161` when the data is unavailable.
        *
        * @unit 0,1 m/s^2
        * @category: Kinematic information
        * @revision: Created in V2.1.1
      */
      AccelerationMagnitudeValue ::= INTEGER {
          positiveOutOfRange (160),
          unavailable        (161)  
      } (0.. 161, ...)"#).unwrap().1;
        assert_eq!(tld.name, String::from("AccelerationMagnitudeValue"));
        assert!(tld.comments.contains("@unit 0,1 m/s^2"));
        if let ASN1Type::Integer(int) = tld.r#type {
            assert_eq!(
                int.constraints.first().unwrap().min_value,
                Some(ASN1Value::Integer(0))
            );
            assert_eq!(
                int.constraints.first().unwrap().max_value,
                Some(ASN1Value::Integer(161))
            );
            assert_eq!(int.constraints.first().unwrap().extensible, true);
            assert_eq!(int.distinguished_values.as_ref().unwrap().len(), 2);
            assert_eq!(
                int.distinguished_values.as_ref().unwrap()[0],
                DistinguishedValue {
                    name: String::from("positiveOutOfRange"),
                    value: 160
                }
            );
        } else {
            panic!("Top-level declaration contains other type than integer.")
        }
    }

    #[test]
    fn parses_toplevel_enumerated_declaration() {
        let tld = top_level_type_declaration(
            r#"-- Coverage Enhancement level encoded according to TS 36.331 [16] --
        CE-mode-B-SupportIndicator ::= ENUMERATED {
           supported,
           ...
        }"#,
        )
        .unwrap()
        .1;
        assert_eq!(tld.name, String::from("CE-mode-B-SupportIndicator"));
        assert_eq!(
            tld.comments,
            String::from(" Coverage Enhancement level encoded according to TS 36.331 [16] ")
        );
        if let ASN1Type::Enumerated(e) = tld.r#type {
            assert_eq!(e.members.len(), 1);
            assert_eq!(
                e.members[0],
                Enumeral {
                    name: "supported".into(),
                    index: 0,
                    description: None
                }
            );
            assert_eq!(e.extensible, Some(1));
        } else {
            panic!("Top-level declaration contains other type than integer.")
        }
    }

    #[test]
    fn parses_toplevel_boolean_declaration() {
        let tld = top_level_type_declaration(
            r#"/**
            * This DE indicates whether a vehicle (e.g. public transport vehicle, truck) is under the embarkation process.
            * If that is the case, the value is *TRUE*, otherwise *FALSE*.
            *
            * @category: Vehicle information
            * @revision: editorial update in V2.1.1
            */
           EmbarkationStatus ::= BOOLEAN"#,
        )
        .unwrap()
        .1;
        assert_eq!(tld.name, String::from("EmbarkationStatus"));
        assert!(tld
            .comments
            .contains("@revision: editorial update in V2.1.1"));
        assert_eq!(tld.r#type, ASN1Type::Boolean);
    }

    #[test]
    fn parses_toplevel_crossrefering_declaration() {
        let tld = top_level_type_declaration(
            r#"-- Comments go here
        EventZone::= EventHistory
        ((WITH COMPONENT (WITH COMPONENTS {..., eventDeltaTime PRESENT})) |
         (WITH COMPONENT (WITH COMPONENTS {..., eventDeltaTime ABSENT})))
         }"#,
        )
        .unwrap()
        .1;
        assert_eq!(
            tld,
            ToplevelTypeDeclaration {
                parameterization: None,
                comments: " Comments go here".into(),
                name: "EventZone".into(),
                r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere {
                    identifier: "EventHistory".into(),
                    constraints: vec![
                        Constraint::ArrayComponentConstraint(ComponentConstraint {
                            is_partial: true,
                            constraints: vec![ConstrainedComponent {
                                identifier: "eventDeltaTime".into(),
                                constraints: vec![],
                                presence: ComponentPresence::Present
                            }]
                        }),
                        Constraint::Arithmetic(ArithmeticOperator::Union),
                        Constraint::ArrayComponentConstraint(ComponentConstraint {
                            is_partial: true,
                            constraints: vec![ConstrainedComponent {
                                identifier: "eventDeltaTime".into(),
                                constraints: vec![],
                                presence: ComponentPresence::Absent
                            }]
                        })
                    ]
                })
            }
        );
    }

    #[test]
    fn parses_anonymous_sequence_of_declaration() {
        let tld = top_level_type_declaration(
            r#"--Comments
        InterferenceManagementZones ::= SEQUENCE (SIZE(1..16, ...)) OF InterferenceManagementZone"#,
        )
        .unwrap()
        .1;
        assert_eq!(
            tld,
            ToplevelTypeDeclaration {
                parameterization: None,
                comments: "Comments".into(),
                name: "InterferenceManagementZones".into(),
                r#type: ASN1Type::SequenceOf(SequenceOf {
                    constraints: vec![Constraint::SizeConstraint(ValueConstraint {
                        min_value: Some(ASN1Value::Integer(1)),
                        max_value: Some(ASN1Value::Integer(16)),
                        extensible: true
                    })],
                    r#type: Box::new(ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere {
                        identifier: "InterferenceManagementZone".into(),
                        constraints: vec![]
                    }))
                })
            }
        );
    }

    #[test]
    fn parses_object_set_value() {
        assert_eq!(
            top_level_information_declaration(
                r#"--comments
        CpmContainers CPM-CONTAINER-ID-AND-TYPE ::= {
        {OriginatingVehicleContainer IDENTIFIED BY originatingVehicleContainer} |
        {PerceivedObjectContainer IDENTIFIED BY perceivedObjectContainer},
        ...
    }"#
            )
            .unwrap()
            .1,
            ToplevelInformationDeclaration {
                comments: "comments".into(),
                name: "CpmContainers".into(),
                class: Some(ClassLink::ByName("CPM-CONTAINER-ID-AND-TYPE".into())),
                value: ASN1Information::ObjectSet(ObjectSet {
                    values: vec![
                        ObjectSetValue::Inline(InformationObjectFields::CustomSyntax(vec![
                            SyntaxApplication::TypeReference(ASN1Type::ElsewhereDeclaredType(
                                DeclarationElsewhere {
                                    identifier: "OriginatingVehicleContainer".into(),
                                    constraints: vec![]
                                }
                            )),
                            SyntaxApplication::Literal("IDENTIFIED".into()),
                            SyntaxApplication::Literal("BY".into()),
                            SyntaxApplication::ValueReference(ASN1Value::ElsewhereDeclaredValue(
                                "originatingVehicleContainer".into()
                            ))
                        ])),
                        ObjectSetValue::Inline(InformationObjectFields::CustomSyntax(vec![
                            SyntaxApplication::TypeReference(ASN1Type::ElsewhereDeclaredType(
                                DeclarationElsewhere {
                                    identifier: "PerceivedObjectContainer".into(),
                                    constraints: vec![]
                                }
                            )),
                            SyntaxApplication::Literal("IDENTIFIED".into()),
                            SyntaxApplication::Literal("BY".into()),
                            SyntaxApplication::ValueReference(ASN1Value::ElsewhereDeclaredValue(
                                "perceivedObjectContainer".into()
                            ))
                        ]))
                    ],
                    extensible: Some(2)
                })
            }
        )
    }
}

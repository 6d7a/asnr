use asnr_grammar::{subtyping::*, *};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{into, map, opt, value},
    multi::{many0_count, many1, separated_list1},
    sequence::{pair, preceded, terminated, tuple},
    IResult,
};

use super::{
    asn1_value,
    common::{
        extension_marker, identifier, in_braces, in_parentheses, opt_parentheses, range_seperator,
        skip_ws_and_comments,
    },
    information_object_class::object_set,
};

pub fn constraint<'a>(input: &'a str) -> IResult<&'a str, Vec<Constraint>> {
    alt((
        many1(single_constraint),
        map(
            in_parentheses(pair(
                single_constraint,
                many1(pair(arithmetic_operator, single_constraint)),
            )),
            |(f, ac)| {
                let mut first = vec![f];
                for (a, c) in ac {
                    first.push(a);
                    first.push(c);
                }
                first
            },
        ),
        composed_value_constraint,
    ))(input)
}

pub fn single_constraint<'a>(input: &'a str) -> IResult<&'a str, Constraint> {
    skip_ws_and_comments(alt((
        map(size_constraint, |c| Constraint::SizeConstraint(c)),
        map(table_constraint, |t| Constraint::TableConstraint(t)),
        map(simple_value_constraint, |v| Constraint::ValueConstraint(v)),
        map(array_component_constraint, |c| {
            Constraint::ArrayComponentConstraint(c)
        }),
        map(component_constraint, |c| Constraint::ComponentConstraint(c)),
    )))(input)
}

pub fn arithmetic_operator<'a>(input: &'a str) -> IResult<&'a str, Constraint> {
    skip_ws_and_comments(alt((
        value(
            Constraint::Arithmetic(ArithmeticOperator::Intersection),
            tag(INTERSECTION),
        ),
        value(
            Constraint::Arithmetic(ArithmeticOperator::Intersection),
            tag(CARET),
        ),
        value(
            Constraint::Arithmetic(ArithmeticOperator::Union),
            tag(UNION),
        ),
        value(Constraint::Arithmetic(ArithmeticOperator::Union), tag(PIPE)),
        value(
            Constraint::Arithmetic(ArithmeticOperator::Except),
            tag(EXCEPT),
        ),
    )))(input)
}

pub fn size_constraint<'a>(input: &'a str) -> IResult<&'a str, ValueConstraint> {
    opt_parentheses(preceded(
        skip_ws_and_comments(tag(SIZE)),
        skip_ws_and_comments(simple_value_constraint),
    ))(input)
}

pub fn simple_value_constraint<'a>(input: &'a str) -> IResult<&'a str, ValueConstraint> {
    in_parentheses(value_constraint)(input)
}

fn value_constraint<'a>(input: &'a str) -> IResult<&'a str, ValueConstraint> {
    alt((
        extensible_range_constraint, // The most elaborate match first
        strict_extensible_constraint,
        range_constraint,
        strict_constraint, // The most simple match last
    ))(input)
}

pub fn composed_value_constraint<'a>(input: &'a str) -> IResult<&'a str, Vec<Constraint>> {
    map(
        in_parentheses(pair(
            value_constraint,
            many1(pair(arithmetic_operator, value_constraint)),
        )),
        |(f, ac)| {
            let mut first = vec![Constraint::ValueConstraint(f)];
            for (a, c) in ac {
                first.push(a);
                first.push(Constraint::ValueConstraint(c));
            }
            first
        },
    )(input)
}

pub fn strict_constraint<'a>(input: &'a str) -> IResult<&'a str, ValueConstraint> {
    into(asn1_value)(input)
}

pub fn strict_extensible_constraint<'a>(input: &'a str) -> IResult<&'a str, ValueConstraint> {
    into(pair(asn1_value, preceded(char(','), extension_marker)))(input)
}

pub fn range_constraint<'a>(input: &'a str) -> IResult<&'a str, ValueConstraint> {
    into(tuple((
        asn1_value,
        range_seperator,
        skip_ws_and_comments(asn1_value),
    )))(input)
}

pub fn extensible_range_constraint<'a>(input: &'a str) -> IResult<&'a str, ValueConstraint> {
    into(tuple((
        asn1_value,
        range_seperator,
        skip_ws_and_comments(asn1_value),
        preceded(skip_ws_and_comments(char(COMMA)), extension_marker),
    )))(input)
}

pub fn component_constraint<'a>(input: &'a str) -> IResult<&'a str, ComponentConstraint> {
    into(in_parentheses(preceded(
        tag(WITH_COMPONENTS),
        in_braces(pair(
            opt(skip_ws_and_comments(terminated(
                value(ExtensionMarker(), tag(ELLIPSIS)),
                skip_ws_and_comments(char(COMMA)),
            ))),
            many1(terminated(
                subset_member,
                opt(skip_ws_and_comments(char(COMMA))),
            )),
        )),
    )))(input)
}

fn array_component_constraint<'a>(input: &'a str) -> IResult<&'a str, ComponentConstraint> {
    in_parentheses(preceded(
        tag(WITH_COMPONENT),
        skip_ws_and_comments(alt((component_constraint, array_component_constraint))),
    ))(input)
}

fn subset_member<'a>(
    input: &'a str,
) -> IResult<&'a str, (&str, Option<Vec<Constraint>>, Option<ComponentPresence>)> {
    skip_ws_and_comments(tuple((
        identifier,
        opt(skip_ws_and_comments(constraint)),
        opt(skip_ws_and_comments(alt((
            value(ComponentPresence::Present, tag(PRESENT)),
            value(ComponentPresence::Absent, tag(ABSENT)),
        )))),
    )))(input)
}

fn table_constraint<'a>(input: &'a str) -> IResult<&'a str, TableConstraint> {
    in_parentheses(into(pair(
        object_set,
        opt(in_braces(separated_list1(
            skip_ws_and_comments(char(COMMA)),
            relational_constraint,
        ))),
    )))(input)
}

fn relational_constraint<'a>(input: &'a str) -> IResult<&'a str, RelationalConstraint> {
    into(skip_ws_and_comments(preceded(
        char(AT),
        pair(many0_count(char(DOT)), identifier),
    )))(input)
}

#[cfg(test)]
mod tests {
    use asnr_grammar::{subtyping::*, types::*, *};

    use crate::parser::constraint::{component_constraint, constraint, simple_value_constraint};

    #[test]
    fn parses_value_constraint() {
        assert_eq!(
            simple_value_constraint("(5)"),
            Ok((
                "",
                ValueConstraint {
                    min_value: Some(ASN1Value::Integer(5)),
                    max_value: Some(ASN1Value::Integer(5)),
                    extensible: false
                }
            ))
        );
        assert_eq!(
            simple_value_constraint("(5..9)"),
            Ok((
                "",
                ValueConstraint {
                    min_value: Some(ASN1Value::Integer(5)),
                    max_value: Some(ASN1Value::Integer(9)),
                    extensible: false
                }
            ))
        );
        assert_eq!(
            simple_value_constraint("(-5..9)"),
            Ok((
                "",
                ValueConstraint {
                    min_value: Some(ASN1Value::Integer(-5)),
                    max_value: Some(ASN1Value::Integer(9)),
                    extensible: false
                }
            ))
        );
        assert_eq!(
            simple_value_constraint("(-9..-4, ...)"),
            Ok((
                "",
                ValueConstraint {
                    min_value: Some(ASN1Value::Integer(-9)),
                    max_value: Some(ASN1Value::Integer(-4)),
                    extensible: true
                }
            ))
        );
    }

    #[test]
    fn parses_value_constraint_with_inserted_comment() {
        assert_eq!(
            simple_value_constraint("(-9..-4, -- Very annoying! -- ...)"),
            Ok((
                "",
                ValueConstraint {
                    min_value: Some(ASN1Value::Integer(-9)),
                    max_value: Some(ASN1Value::Integer(-4)),
                    extensible: true
                }
            ))
        );
        assert_eq!(
            simple_value_constraint("(-9-- Very annoying! --..-4,  ...)"),
            Ok((
                "",
                ValueConstraint {
                    min_value: Some(ASN1Value::Integer(-9)),
                    max_value: Some(ASN1Value::Integer(-4)),
                    extensible: true
                }
            ))
        );
    }

    #[test]
    fn parses_size_constraint() {
        assert_eq!(
            constraint("(SIZE(3..16, ...))").unwrap().1,
            vec![Constraint::SizeConstraint(ValueConstraint {
                min_value: Some(ASN1Value::Integer(3)),
                max_value: Some(ASN1Value::Integer(16)),
                extensible: true
            })]
        )
    }

    #[test]
    fn parses_full_component_constraint() {
        assert_eq!(
            component_constraint(
                "(WITH COMPONENTS
              {ordering ABSENT ,
              sales (0..5) PRESENT,
              e-cash-return ABSENT } )"
            )
            .unwrap()
            .1,
            ComponentConstraint {
                is_partial: false,
                constraints: vec![
                    ConstrainedComponent {
                        identifier: "ordering".into(),
                        constraints: vec![],
                        presence: ComponentPresence::Absent
                    },
                    ConstrainedComponent {
                        identifier: "sales".into(),
                        constraints: vec![Constraint::ValueConstraint(ValueConstraint {
                            min_value: Some(ASN1Value::Integer(0)),
                            max_value: Some(ASN1Value::Integer(5)),
                            extensible: false
                        })],
                        presence: ComponentPresence::Present
                    },
                    ConstrainedComponent {
                        identifier: "e-cash-return".into(),
                        constraints: vec![],
                        presence: ComponentPresence::Absent
                    }
                ]
            }
        );
    }

    #[test]
    fn parses_partial_component_constraint() {
        assert_eq!(
            component_constraint(
                "( WITH COMPONENTS
                  {... ,
                  ordering ABSENT,
                  sales (0..5) } )"
            )
            .unwrap()
            .1,
            ComponentConstraint {
                is_partial: true,
                constraints: vec![
                    ConstrainedComponent {
                        identifier: "ordering".into(),
                        constraints: vec![],
                        presence: ComponentPresence::Absent
                    },
                    ConstrainedComponent {
                        identifier: "sales".into(),
                        constraints: vec![Constraint::ValueConstraint(ValueConstraint {
                            min_value: Some(ASN1Value::Integer(0)),
                            max_value: Some(ASN1Value::Integer(5)),
                            extensible: false
                        })],
                        presence: ComponentPresence::Unspecified
                    },
                ]
            }
        );
    }

    #[test]
    fn parses_composite_array_constraint() {
        assert_eq!(
            constraint(
                "((WITH COMPONENT (WITH COMPONENTS {..., eventDeltaTime PRESENT})) |
                (WITH COMPONENT (WITH COMPONENTS {..., eventDeltaTime ABSENT})))
            "
            )
            .unwrap()
            .1,
            vec![
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
        );
    }

    #[test]
    fn parses_composite_component_constraint() {
        assert_eq!(
            constraint(
                "((WITH COMPONENTS {..., laneId PRESENT, connectionId ABSENT }) |
                (WITH COMPONENTS {..., laneId ABSENT, connectionId PRESENT }))
            "
            )
            .unwrap()
            .1,
            vec![
                Constraint::ComponentConstraint(ComponentConstraint {
                    is_partial: true,
                    constraints: vec![
                        ConstrainedComponent {
                            identifier: "laneId".into(),
                            constraints: vec![],
                            presence: ComponentPresence::Present
                        },
                        ConstrainedComponent {
                            identifier: "connectionId".into(),
                            constraints: vec![],
                            presence: ComponentPresence::Absent
                        }
                    ]
                }),
                Constraint::Arithmetic(ArithmeticOperator::Union),
                Constraint::ComponentConstraint(ComponentConstraint {
                    is_partial: true,
                    constraints: vec![
                        ConstrainedComponent {
                            identifier: "laneId".into(),
                            constraints: vec![],
                            presence: ComponentPresence::Absent
                        },
                        ConstrainedComponent {
                            identifier: "connectionId".into(),
                            constraints: vec![],
                            presence: ComponentPresence::Present
                        }
                    ]
                })
            ]
        );
    }

    #[test]
    fn parses_composite_range_constraint() {
        assert_eq!(
            constraint(
                "(0..3|5..8|10)
            "
            )
            .unwrap()
            .1,
            vec![
                Constraint::ValueConstraint(ValueConstraint {
                    min_value: Some(ASN1Value::Integer(0)),
                    max_value: Some(ASN1Value::Integer(3)),
                    extensible: false
                }),
                Constraint::Arithmetic(ArithmeticOperator::Union),
                Constraint::ValueConstraint(ValueConstraint {
                    min_value: Some(ASN1Value::Integer(5)),
                    max_value: Some(ASN1Value::Integer(8)),
                    extensible: false
                }),
                Constraint::Arithmetic(ArithmeticOperator::Union),
                Constraint::ValueConstraint(ValueConstraint {
                    min_value: Some(ASN1Value::Integer(10)),
                    max_value: Some(ASN1Value::Integer(10)),
                    extensible: false
                })
            ]
        );
    }

    #[test]
    fn parses_composite_range_constraint_with_elsewhere_declared_values() {
        assert_eq!(
            constraint(
                "(unknown   | passengerCar..tram
              | agricultural)"
            )
            .unwrap()
            .1,
            vec![
                Constraint::ValueConstraint(ValueConstraint {
                    min_value: Some(ASN1Value::ElsewhereDeclaredValue("unknown".into())),
                    max_value: Some(ASN1Value::ElsewhereDeclaredValue("unknown".into())),
                    extensible: false
                }),
                Constraint::Arithmetic(ArithmeticOperator::Union),
                Constraint::ValueConstraint(ValueConstraint {
                    min_value: Some(ASN1Value::ElsewhereDeclaredValue("passengerCar".into())),
                    max_value: Some(ASN1Value::ElsewhereDeclaredValue("tram".into())),
                    extensible: false
                }),
                Constraint::Arithmetic(ArithmeticOperator::Union),
                Constraint::ValueConstraint(ValueConstraint {
                    min_value: Some(ASN1Value::ElsewhereDeclaredValue("agricultural".into())),
                    max_value: Some(ASN1Value::ElsewhereDeclaredValue("agricultural".into())),
                    extensible: false
                })
            ]
        );
    }

    #[test]
    fn parses_table_constraint() {
        assert_eq!(
            constraint(
                "({
              My-ops | 
              {
                &id 5,
                &Type INTEGER (1..6)
              } |
              {ConnectionManeuverAssist-addGrpC  IDENTIFIED BY addGrpC}, 
              ...
            })"
            )
            .unwrap()
            .1,
            vec![Constraint::TableConstraint(TableConstraint {
                object_set: ObjectSet {
                    values: vec![
                        ObjectSetValue::Reference("My-ops".into()),
                        ObjectSetValue::Inline(InformationObjectFields::DefaultSyntax(vec![
                            InformationObjectField::FixedValueField(FixedValueField {
                                identifier: "&id".into(),
                                value: ASN1Value::Integer(5)
                            }),
                            InformationObjectField::TypeField(TypeField {
                                identifier: "&Type".into(),
                                r#type: ASN1Type::Integer(Integer {
                                    constraints: vec![ValueConstraint {
                                        min_value: Some(ASN1Value::Integer(1)),
                                        max_value: Some(ASN1Value::Integer(6)),
                                        extensible: false
                                    }],
                                    distinguished_values: None
                                })
                            })
                        ])),
                        ObjectSetValue::Inline(InformationObjectFields::CustomSyntax(vec![
                            SyntaxApplication::TypeReference(ASN1Type::ElsewhereDeclaredType(
                                DeclarationElsewhere {
                                    identifier: "ConnectionManeuverAssist-addGrpC".into(),
                                    constraints: vec![]
                                }
                            )),
                            SyntaxApplication::Literal("IDENTIFIED".into()),
                            SyntaxApplication::Literal("BY".into()),
                            SyntaxApplication::ValueReference(ASN1Value::ElsewhereDeclaredValue(
                                "addGrpC".into()
                            ))
                        ]))
                    ],
                    extensible: Some(3)
                },
                linked_fields: vec![]
            })]
        );
    }
}

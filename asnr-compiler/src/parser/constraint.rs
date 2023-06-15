use asnr_grammar::{
    subtyping::{
        ArithmeticOperator, ComponentConstraint, ComponentPresence, Constraint, ExtensionMarker,
        RangeConstraint,
    },
    ABSENT, CARET, COMMA, ELLIPSIS, EXCEPT, INTERSECTION, PIPE, PRESENT, SIZE, UNION,
    WITH_COMPONENTS, WITH_COMPONENT,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, i128},
    combinator::{into, map, opt, value},
    multi::many1,
    sequence::{pair, preceded, terminated, tuple},
    IResult,
};

use super::{
    common::{
        extension_marker, identifier, in_braces, in_parentheses, range_marker, skip_ws_and_comments,
    },
    util::map_into,
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
    ))(input)
}

pub fn single_constraint<'a>(input: &'a str) -> IResult<&'a str, Constraint> {
    skip_ws_and_comments(alt((
        map(value_constraint, |v| Constraint::RangeConstraint(v)),
        map(array_component_constraint, |c| {
            Constraint::ComponentConstraint(c)
        }),
        map(component_constraint, |c| Constraint::ComponentConstraint(c)),
        map(size_constraint, |c| Constraint::RangeConstraint(c)),
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

pub fn size_constraint<'a>(input: &'a str) -> IResult<&'a str, RangeConstraint> {
    preceded(
        skip_ws_and_comments(tag(SIZE)),
        skip_ws_and_comments(value_constraint),
    )(input)
}

pub fn value_constraint<'a>(input: &'a str) -> IResult<&'a str, RangeConstraint> {
    in_parentheses(alt((
        extensible_range_constraint, // The most elaborate match first
        strict_extensible_constraint,
        range_constraint,
        strict_constraint, // The most simple match last
    )))(input)
}

pub fn strict_constraint<'a>(input: &'a str) -> IResult<&'a str, RangeConstraint> {
    map_into(i128)(input)
}

pub fn strict_extensible_constraint<'a>(input: &'a str) -> IResult<&'a str, RangeConstraint> {
    into(pair(i128, preceded(char(','), extension_marker)))(input)
}

pub fn range_constraint<'a>(input: &'a str) -> IResult<&'a str, RangeConstraint> {
    into(tuple((i128, range_marker, skip_ws_and_comments(i128))))(input)
}

pub fn extensible_range_constraint<'a>(input: &'a str) -> IResult<&'a str, RangeConstraint> {
    into(tuple((
        i128,
        range_marker,
        skip_ws_and_comments(i128),
        preceded(skip_ws_and_comments(char(COMMA)), extension_marker),
    )))(input)
}

pub fn component_constraint<'a>(input: &'a str) -> IResult<&'a str, ComponentConstraint> {
    into(in_parentheses(preceded(
        tag(WITH_COMPONENTS),
        skip_ws_and_comments(in_braces(pair(
            opt(skip_ws_and_comments(terminated(
                value(ExtensionMarker(), tag(ELLIPSIS)),
                skip_ws_and_comments(char(COMMA)),
            ))),
            many1(terminated(
                subset_member,
                opt(skip_ws_and_comments(char(COMMA))),
            )),
        ))),
    )))(input)
}

pub fn array_component_constraint<'a>(input: &'a str) -> IResult<&'a str, ComponentConstraint> {
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

#[cfg(test)]
mod tests {
    use asnr_grammar::{subtyping::{
        ArithmeticOperator, ComponentConstraint, ComponentPresence, ConstrainedComponent,
        Constraint, RangeConstraint,
    }, ASN1Value};

    use crate::parser::constraint::{component_constraint, constraint, value_constraint};

    #[test]
    fn parses_value_constraint() {
        assert_eq!(
            value_constraint("(5)"),
            Ok((
                "",
                RangeConstraint {
                    min_value: Some(ASN1Value::Integer(5)),
                    max_value: Some(ASN1Value::Integer(5)),
                    extensible: false
                }
            ))
        );
        assert_eq!(
            value_constraint("(5..9)"),
            Ok((
                "",
                RangeConstraint {
                    min_value: Some(ASN1Value::Integer(5)),
                    max_value: Some(ASN1Value::Integer(9)),
                    extensible: false
                }
            ))
        );
        assert_eq!(
            value_constraint("(-5..9)"),
            Ok((
                "",
                RangeConstraint {
                    min_value: Some(ASN1Value::Integer(-5)),
                    max_value: Some(ASN1Value::Integer(9)),
                    extensible: false
                }
            ))
        );
        assert_eq!(
            value_constraint("(-9..-4, ...)"),
            Ok((
                "",
                RangeConstraint {
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
            value_constraint("(-9..-4, -- Very annoying! -- ...)"),
            Ok((
                "",
                RangeConstraint {
                    min_value: Some(ASN1Value::Integer(-9)),
                    max_value: Some(ASN1Value::Integer(-4)),
                    extensible: true
                }
            ))
        );
        assert_eq!(
            value_constraint("(-9-- Very annoying! --..-4,  ...)"),
            Ok((
                "",
                RangeConstraint {
                    min_value: Some(ASN1Value::Integer(-9)),
                    max_value: Some(ASN1Value::Integer(-4)),
                    extensible: true
                }
            ))
        );
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
                        constraints: vec![Constraint::RangeConstraint(RangeConstraint {
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
                        constraints: vec![Constraint::RangeConstraint(RangeConstraint {
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
                Constraint::ComponentConstraint(ComponentConstraint {
                    is_partial: true,
                    constraints: vec![ConstrainedComponent {
                        identifier: "eventDeltaTime".into(),
                        constraints: vec![],
                        presence: ComponentPresence::Present
                    }]
                }),
                Constraint::Arithmetic(ArithmeticOperator::Union),
                Constraint::ComponentConstraint(ComponentConstraint {
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
}

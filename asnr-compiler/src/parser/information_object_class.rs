use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, char, one_of},
    combinator::{into, map, opt, recognize, value},
    multi::{many0, many1},
    sequence::{pair, preceded, terminated, tuple},
    IResult,
};

use asnr_grammar::{types::*, *};

use super::{
    asn1_type, asn1_value,
    common::{
        default, in_braces, in_brackets, optional_comma,
        optional_marker, skip_ws_and_comments, value_set,
    },
};

pub fn information_object_class<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    map(
        preceded(
            skip_ws_and_comments(tag(CLASS)),
            pair(
                in_braces(many0(terminated(
                    skip_ws_and_comments(information_object_field),
                    optional_comma,
                ))),
                opt(preceded(skip_ws_and_comments(tag(WITH_SYNTAX)), syntax)),
            ),
        ),
        |m| ASN1Type::InformationObjectClass(m.into()),
    )(input)
}

pub fn information_object<'a>(input: &'a str) -> IResult<&'a str, InformationObjectFields> {
    skip_ws_and_comments(in_braces(alt((
        default_syntax_information_object,
        custom_syntax_information_object,
    ))))(input)
}

fn custom_syntax_information_object<'a>(
    input: &'a str,
) -> IResult<&'a str, InformationObjectFields> {
    map(
        skip_ws_and_comments(many1(skip_ws_and_comments(alt((
            value(SyntaxApplication::Comma, char(COMMA)),
            map(syntax_literal, |m| SyntaxApplication::Literal(m.into())),
            map(value_set, |m| SyntaxApplication::ObjectSetDeclaration(m)),
            map(asn1_type, |m| SyntaxApplication::TypeReference(m)),
            map(asn1_value, |m| SyntaxApplication::ValueReference(m))
        ))))),
        |m| InformationObjectFields::CustomSyntax(m),
    )(input)
}

fn default_syntax_information_object<'a>(
    input: &'a str,
) -> IResult<&'a str, InformationObjectFields> {
    map(
        many1(terminated(
            skip_ws_and_comments(alt((
                into(pair(
                    single_value_field_id,
                    skip_ws_and_comments(asn1_value),
                )),
                into(pair(multiple_value_field_id, value_set)),
                into(pair(
                    multiple_value_field_id,
                    skip_ws_and_comments(asn1_type),
                )),
            ))),
            optional_comma,
        )),
        |m| InformationObjectFields::DefaultSyntax(m),
    )(input)
}

fn information_object_field<'a>(input: &'a str) -> IResult<&'a str, InformationObjectClassField> {
    into(tuple((
        skip_ws_and_comments(object_field_identifier),
        opt(skip_ws_and_comments(asn1_type)),
        opt(skip_ws_and_comments(tag(UNIQUE))),
        optional_marker,
        default,
    )))(input)
}

fn object_field_identifier<'a>(input: &'a str) -> IResult<&'a str, ObjectFieldIdentifier> {
    alt((single_value_field_id, multiple_value_field_id))(input)
}

fn single_value_field_id<'a>(input: &'a str) -> IResult<&'a str, ObjectFieldIdentifier> {
    map(
        recognize(tuple((
            char(AMPERSAND),
            one_of("abcdefghijklmnopqrstuvwxyz"),
            many0(alt((preceded(char('-'), alphanumeric1), alphanumeric1))),
        ))),
        |s| ObjectFieldIdentifier::SingleValue(String::from(s)),
    )(input)
}

fn multiple_value_field_id<'a>(input: &'a str) -> IResult<&'a str, ObjectFieldIdentifier> {
    map(
        recognize(tuple((
            char(AMPERSAND),
            one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
            many0(alt((preceded(char('-'), alphanumeric1), alphanumeric1))),
        ))),
        |m| ObjectFieldIdentifier::MultipleValue(String::from(m)),
    )(input)
}

fn syntax<'a>(input: &'a str) -> IResult<&'a str, Vec<SyntaxExpression>> {
    skip_ws_and_comments(in_braces(many1(syntax_token_or_group_spec)))(input)
}

fn syntax_token_or_group_spec<'a>(input: &'a str) -> IResult<&'a str, SyntaxExpression> {
    alt((
        map(syntax_token, |m| SyntaxExpression::Required(m)),
        map(syntax_optional_group, |m| SyntaxExpression::Optional(m)),
    ))(input)
}

fn syntax_optional_group<'a>(input: &'a str) -> IResult<&'a str, Vec<SyntaxExpression>> {
    skip_ws_and_comments(in_brackets(skip_ws_and_comments(many1(
        syntax_token_or_group_spec,
    ))))(input)
}

fn syntax_token<'a>(input: &'a str) -> IResult<&'a str, SyntaxToken> {
    skip_ws_and_comments(alt((
        map(syntax_literal, |m| SyntaxToken::from(m)),
        map(object_field_identifier, |m| SyntaxToken::from(m)),
        map(tag(COMMA.to_string().as_str()), |m| SyntaxToken::from(m)),
    )))(input)
}

fn syntax_literal<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    recognize(pair(
        one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
        many0(alt((
            preceded(char('-'), one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ")),
            one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
        ))),
    ))(input)
}

mod tests {
    use asnr_grammar::{types::*, *};

    use crate::parser::information_object_class::information_object_class;

    #[test]
    fn parses_information_object_class() {
        assert_eq!(
            information_object_class(
                r#"CLASS
      {&operationCode CHOICE {local INTEGER,
      global OCTET STRING}
      UNIQUE,
      &ArgumentType,
      &ResultType,
      &Errors ERROR OPTIONAL }"#
            )
            .unwrap()
            .1,
            ASN1Type::InformationObjectClass(InformationObjectClass {
                syntax: None,
                fields: vec![
                    InformationObjectClassField {
                        identifier: ObjectFieldIdentifier::SingleValue("&operationCode".into()),
                        r#type: Some(ASN1Type::Choice(Choice {
                            extensible: None,
                            options: vec![
                                ChoiceOption {
                                    name: "local".into(),
                                    tag: None,
                                    r#type: ASN1Type::Integer(Integer {
                                        constraints: vec![],
                                        distinguished_values: None
                                    }),
                                    constraints: vec![]
                                },
                                ChoiceOption {
                                    name: "global".into(),
                                    tag: None,
                                    r#type: ASN1Type::CharacterString(CharacterString {
                                        constraints: vec![],
                                        r#type: asnr_grammar::CharacterStringType::OctetString
                                    }),
                                    constraints: vec![]
                                }
                            ],
                            constraints: vec![]
                        })),
                        is_optional: false,
                        is_unique: true,
                        default: None
                    },
                    InformationObjectClassField {
                        identifier: ObjectFieldIdentifier::MultipleValue("&ArgumentType".into()),
                        r#type: None,
                        is_optional: false,
                        is_unique: false,
                        default: None
                    },
                    InformationObjectClassField {
                        identifier: ObjectFieldIdentifier::MultipleValue("&ResultType".into()),
                        r#type: None,
                        is_optional: false,
                        is_unique: false,
                        default: None
                    },
                    InformationObjectClassField {
                        identifier: ObjectFieldIdentifier::MultipleValue("&Errors".into()),
                        r#type: Some(ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere {
                            constraints: vec![],
                            identifier: "ERROR".into()
                        })),
                        is_optional: true,
                        is_unique: false,
                        default: None
                    }
                ]
            })
        )
    }
}

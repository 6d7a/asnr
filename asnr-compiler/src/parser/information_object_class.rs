use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, char, one_of},
    combinator::{into, map, opt, recognize},
    multi::{many0, many1, separated_list1},
    sequence::{pair, preceded, terminated, tuple},
    IResult,
};

use asnr_grammar::{types::*, *};

use super::{
    asn1_type, asn1_value,
    common::{in_braces, optional_comma, optional_marker, skip_ws_and_comments, value_set},
};

pub fn information_object_class<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    map(
        preceded(
            skip_ws_and_comments(tag(CLASS)),
            in_braces(many0(terminated(
                skip_ws_and_comments(information_object_field),
                optional_comma,
            ))),
        ),
        |m| ASN1Type::InformationObjectClass(InformationObjectClass { fields: m }),
    )(input)
}

pub fn information_object<'a>(input: &'a str) -> IResult<&'a str, Vec<InformationObjectField>> {
    skip_ws_and_comments(in_braces(many1(terminated(
        skip_ws_and_comments(alt((
            into(pair(
                single_value_field_id,
                skip_ws_and_comments(asn1_value),
            )),
            into(pair(
                multiple_value_field_id,
                value_set,
            )),
            into(pair(
              multiple_value_field_id,
              skip_ws_and_comments(asn1_type),
          ))
        ))),
        optional_comma,
    ))))(input)
}

fn information_object_field<'a>(input: &'a str) -> IResult<&'a str, InformationObjectClassField> {
    into(tuple((
        skip_ws_and_comments(object_field_identifier),
        opt(skip_ws_and_comments(asn1_type)),
        optional_marker,
        opt(skip_ws_and_comments(tag(UNIQUE))),
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
                        is_unique: true
                    },
                    InformationObjectClassField {
                        identifier: ObjectFieldIdentifier::MultipleValue("&ArgumentType".into()),
                        r#type: None,
                        is_optional: false,
                        is_unique: false
                    },
                    InformationObjectClassField {
                        identifier: ObjectFieldIdentifier::MultipleValue("&ResultType".into()),
                        r#type: None,
                        is_optional: false,
                        is_unique: false
                    },
                    InformationObjectClassField {
                        identifier: ObjectFieldIdentifier::MultipleValue("&Errors".into()),
                        r#type: Some(ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere {
                            constraints: vec![],
                            identifier: "ERROR".into()
                        })),
                        is_optional: true,
                        is_unique: false
                    }
                ]
            })
        )
    }
}

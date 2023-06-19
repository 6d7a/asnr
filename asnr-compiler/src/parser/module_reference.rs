use asnr_grammar::{
    EncodingReferenceDefault, ExtensibilityEnvironment, ModuleReference, TaggingEnvironment,
    ASSIGN, AUTOMATIC, BEGIN, DEFINITIONS, EXPLICIT, EXTENSIBILITY_IMPLIED, IMPLICIT, INSTRUCTIONS,
    TAGS,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{into, map, opt},
    sequence::{delimited, pair, terminated, tuple},
    IResult,
};

use super::{
    common::{identifier, skip_ws, skip_ws_and_comments},
    object_identifier::object_identifier,
};

pub fn module_reference<'a>(input: &'a str) -> IResult<&'a str, ModuleReference> {
    skip_ws_and_comments(into(tuple((
        identifier,
        skip_ws(object_identifier),
        skip_ws_and_comments(delimited(
            tag(DEFINITIONS),
            environments,
            skip_ws_and_comments(pair(tag(ASSIGN), skip_ws_and_comments(tag(BEGIN)))),
        )),
    ))))(input)
}

fn environments<'a>(
    input: &'a str,
) -> IResult<
    &'a str,
    (
        Option<EncodingReferenceDefault>,
        TaggingEnvironment,
        ExtensibilityEnvironment,
    ),
> {
    tuple((
        opt(skip_ws_and_comments(into(terminated(
            identifier,
            tag(INSTRUCTIONS),
        )))),
        skip_ws_and_comments(terminated(
            map(
                alt((tag(AUTOMATIC), tag(IMPLICIT), tag(EXPLICIT))),
                |m| match m {
                    AUTOMATIC => TaggingEnvironment::AUTOMATIC,
                    IMPLICIT => TaggingEnvironment::IMPLICIT,
                    _ => TaggingEnvironment::EXPLICIT,
                },
            ),
            skip_ws(tag(TAGS)),
        )),
        skip_ws_and_comments(map(opt(tag(EXTENSIBILITY_IMPLIED)), |m| {
            if m.is_some() {
                ExtensibilityEnvironment::IMPLIED
            } else {
                ExtensibilityEnvironment::EXPLICIT
            }
        })),
    ))(input)
}

mod tests {
    use asnr_grammar::*;

    use crate::parser::module_reference::module_reference;

    #[test]
    fn parses_a_module_reference() {
        assert_eq!(module_reference(r#"--! @options: no-fields-header

    ETSI-ITS-CDD {itu-t (0) identified-organization (4) etsi (0) itsDomain (5) wg1 (1) 102894 cdd (2) major-version-3 (3) minor-version-1 (1)}
    
    DEFINITIONS AUTOMATIC TAGS ::=
    
    BEGIN
    "#).unwrap().1,
    ModuleReference { name: "ETSI-ITS-CDD".into(), module_identifier: ObjectIdentifier(vec![ObjectIdentifierArc { name: Some("itu-t".into()), number: Some(0) }, ObjectIdentifierArc { name: Some("identified-organization".into()), number: Some(4) }, ObjectIdentifierArc { name: Some("etsi".into()), number: Some(0) }, ObjectIdentifierArc { name: Some("itsDomain".into()), number: Some(5) }, ObjectIdentifierArc { name: Some("wg1".into()), number: Some(1) }, ObjectIdentifierArc { name: None, number: Some(102894) }, ObjectIdentifierArc { name: Some("cdd".into()), number: Some(2) }, ObjectIdentifierArc { name: Some("major-version-3".into()), number: Some(3) }, ObjectIdentifierArc { name: Some("minor-version-1".into()), number: Some(1) }]), encoding_reference_default: None, tagging_environment: asnr_grammar::TaggingEnvironment::AUTOMATIC, extensibility_environment: asnr_grammar::ExtensibilityEnvironment::EXPLICIT }
  )
    }
}

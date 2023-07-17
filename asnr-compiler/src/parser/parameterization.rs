use nom::{
    character::complete::char, combinator::into, multi::separated_list1, sequence::separated_pair,
    IResult,
};

use asnr_grammar::{parameterization::*, *};

use super::common::{identifier, in_braces, skip_ws_and_comments};

pub fn parameterization<'a>(input: &'a str) -> IResult<&'a str, Parameterization> {
    into(in_braces(separated_list1(
        char(COMMA),
        skip_ws_and_comments(separated_pair(
            identifier,
            skip_ws_and_comments(char(COLON)),
            skip_ws_and_comments(identifier),
        )),
    )))(input)
}

#[cfg(test)]
mod tests {
    use asnr_grammar::{
        parameterization::{Parameterization, ParameterizationArgument},
        ASN1Type, DeclarationElsewhere,
    };

    use crate::parser::parameterization::parameterization;

    #[test]
    fn parses_class_parameterization_param() {
        assert_eq!(
            parameterization(r#"{REG-EXT-ID-AND-TYPE : Set}"#)
                .unwrap()
                .1,
            Parameterization {
                parameters: vec![ParameterizationArgument {
                    r#type: "REG-EXT-ID-AND-TYPE".into(),
                    name: "Set".into()
                }]
            }
        )
    }
}

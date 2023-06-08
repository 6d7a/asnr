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
    combinator::{into, map},
    multi::many0,
    sequence::{pair, preceded, tuple},
    IResult,
};

use asnr_grammar::{ASN1Type, ASN1Value, Header, ToplevelDeclaration};

use self::{
    bit_string::{bit_string, bit_string_value},
    boolean::{boolean, boolean_value},
    common::*,
    enumerated::*,
    sequence_of::*,
    error::ParserError,
    header::header,
    integer::*,
    character_string::character_string,
    sequence::sequence,
};

mod bit_string;
mod boolean;
mod common;
mod enumerated;
mod error;
mod header;
mod integer;
mod object_identifier;
mod character_string;
mod sequence;
mod sequence_of;
mod util;

pub fn asn_spec<'a>(input: &'a str) -> Result<(Header, Vec<ToplevelDeclaration>), ParserError> {
    pair(header, many0(skip_ws(top_level_declaration)))(input)
        .map(|(_, res)| res)
        .map_err(|e| e.into())
}

pub fn top_level_declaration<'a>(input: &'a str) -> IResult<&'a str, ToplevelDeclaration> {
    into(tuple((
        skip_ws(many0(comment)),
        skip_ws(identifier),
        preceded(assignment, asn1_type),
    )))(input)
}

pub fn asn1_type<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    alt((
        sequence_of,
        sequence,
        integer,
        enumerated,
        boolean,
        bit_string,
        character_string,
        elsewhere_declared_type,
    ))(input)
}

pub fn asn1_value<'a>(input: &'a str) -> IResult<&'a str, ASN1Value> {
    alt((
        bit_string_value,
        boolean_value,
        integer_value,
        enumerated_value,
    ))(input)
}

pub fn elsewhere_declared_type<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    map(skip_ws_and_comments(identifier), |m| {
        ASN1Type::ElsewhereDeclaredType(m.into())
    })(input)
}

#[cfg(test)]
mod tests {
    use core::panic;

    use asnr_grammar::{ASN1Type, DistinguishedValue, Enumeral};

    use super::top_level_declaration;

    #[test]
    fn parses_toplevel_simple_integer_declaration() {
        let tld = top_level_declaration(
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
            assert!(int.constraint.is_some());
            assert_eq!(int.constraint.as_ref().unwrap().min_value, Some(1));
            assert_eq!(int.constraint.as_ref().unwrap().max_value, Some(8));
            assert_eq!(int.constraint.as_ref().unwrap().extensible, false);
        } else {
            panic!("Top-level declaration contains other type than integer.")
        }
    }

    #[test]
    fn parses_toplevel_macro_integer_declaration() {
        let tld = top_level_declaration(r#"/** 
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
            assert_eq!(int.constraint.as_ref().unwrap().min_value, Some(0));
            assert_eq!(int.constraint.as_ref().unwrap().max_value, Some(161));
            assert_eq!(int.constraint.as_ref().unwrap().extensible, true);
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
        let tld = top_level_declaration(
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
        let tld = top_level_declaration(
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
}

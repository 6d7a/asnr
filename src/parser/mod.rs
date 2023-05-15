use nom::{
    sequence::{preceded, tuple},
    IResult,
};

use crate::grammar::token::{ToplevelDeclaration, COMMA, LEFT_BRACE, RIGHT_BRACE};

use self::{common::*, integer::*, util::map_into};

mod common;
mod enumerated;
mod integer;
mod util;

pub fn top_level_declaration<'a>(input: &'a str) -> IResult<&'a str, ToplevelDeclaration> {
    map_into(tuple((
        skip_ws(comment),
        skip_ws(identifier),
        preceded(assignment, integer),
    )))(input)
}

#[cfg(test)]
mod tests {
    use core::panic;

    use crate::grammar::token::{ASN1Type, DistinguishedValue};

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
}

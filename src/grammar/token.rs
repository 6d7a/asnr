use std::{num::{ParseIntError, self}, collections::btree_map::Range};

use nom::{IResult, error::{ParseError, Error}};

/// Comment tokens
pub const BLOCK_COMMENT_BEGIN: &'static [u8] = b"/*";
pub const BLOCK_COMMENT_CONTINUED_LINE: &'static [u8] = b"*";
pub const BLOCK_COMMENT_END: &'static [u8] = b"*/";
pub const LINE_COMMENT: &'static [u8] = b"//";

/// Bracket tokens
pub const LEFT_PARENTHESIS: &'static [u8] = b"(";
pub const RIGHT_PARENTHESIS: &'static [u8] = b")";
pub const LEFT_BRACKET: &'static [u8] = b"[";
pub const RIGHT_BRACKET: &'static [u8] = b"]";
pub const LEFT_BRACE: &'static [u8] = b"{";
pub const RIGHT_BRACE: &'static [u8] = b"}";
pub const LEFT_CHEVRON: &'static [u8] = b"<";
pub const RIGHT_CHEVRON: &'static [u8] = b">";

/// Type tokens
pub const NULL: &'static [u8] = b"NULL";
pub const BOOLEAN: &'static [u8] = b"BOOLEAN";
pub const INTEGER: &'static [u8] = b"INTEGER";
pub const REAL: &'static [u8] = b"REAL";
pub const BIT_STRING: &'static [u8] = b"BIT STRING";
pub const OCTET_STRING: &'static [u8] = b"OCTET STRING";
pub const IA5_STRING: &'static [u8] = b"IA5String";
pub const UTF8_STRING: &'static [u8] = b"UTF8String";
pub const NUMERIC_STRING: &'static [u8] = b"NumericString";
pub const VISIBLE_STRING: &'static [u8] = b"VisibleString";
pub const ENUMERATED: &'static [u8] = b"ENUMERATED";
pub const CHOICE: &'static [u8] = b"CHOICE";
pub const SEQUENCE: &'static [u8] = b"SEQUENCE";
pub const SEQUENCE_OF: &'static [u8] = b"SEQUENCE OF";
pub const SET: &'static [u8] = b"SET";
pub const SET_OF: &'static [u8] = b"SET OF";

pub const SIZE: &'static [u8] = b"SIZE";
pub const ASSIGN: &'static [u8] = b"::=";
pub const RANGE: &'static [u8] = b"..";
pub const EXTENSION: &'static [u8] = b"...";
pub const COMMA: &'static [u8] = b",";

pub trait Quote<'a> {
  fn quote(&self) -> &'a str;
}

pub enum ASN1Type {
    Null(AsnNull),
    Boolean(AsnBoolean),
    Integer(AsnInteger),
    Real(AsnReal),
    BitString(AsnBitString),
    OctetString(AsnOctetString),
    Ia5String(AsnIa5String),
    Utf8String(AsnUtf8String),
    NumericString(AsnNumericString),
    VisibleString(AsnVisibleString),
    Enumerated(AsnEnumerated),
    Choice(AsnChoice),
    Sequence(AsnSequence),
    SequenceOf(AsnSequenceOf),
    Set(AsnSet),
    SetOf(AsnSetOf),
}

pub struct AsnNull {
}

pub struct AsnBoolean {
}

#[derive(Debug, Clone, PartialEq)]
pub struct AsnInteger {
  pub constraint: Option<Constraint>
}

impl Default for AsnInteger {
    fn default() -> Self {
        Self { constraint: None }
    }
}

impl From<Constraint> for AsnInteger {
    fn from(value: Constraint) -> Self {
        Self { constraint: Some(value) }
    }
}

pub struct AsnReal {

}

pub struct AsnBitString {

}

pub struct AsnOctetString {

}

pub struct AsnIa5String {

}

pub struct AsnUtf8String {

}

pub struct AsnNumericString {

}

pub struct AsnVisibleString {

}

pub struct AsnEnumerated {

}

pub struct AsnChoice {

}

pub struct AsnSequence {

}

pub struct AsnSequenceOf {

}

pub struct AsnSet {

}

pub struct AsnSetOf {

}

#[derive(Debug)]
pub struct RangeParticle();

#[derive(Debug)]
pub struct ExtensionParticle();

#[derive(Debug, PartialEq, Clone)]
pub struct Constraint {
  pub min_value: Option<i128>,
  pub max_value: Option<i128>,
  pub extensible: bool,
}

impl Constraint {
    pub fn new(min: Option<i128>, max: Option<i128>, extensible: bool) -> Self {
      Self { min_value: min, max_value: max, extensible }
    }
}

impl<'a> From<i128> for Constraint {
  fn from(value: i128) -> Self {
    Self { min_value: Some(value), max_value: Some(value), extensible: false }
  }
}

impl<'a> From<(i128, RangeParticle, i128)> for Constraint {
  fn from(value: (i128, RangeParticle, i128)) -> Self {
    Self { min_value: Some(value.0), max_value: Some(value.2), extensible: false }
  }
}

impl<'a> From<(i128, ExtensionParticle)> for Constraint {
  fn from(value: (i128, ExtensionParticle)) -> Self {
    Self { min_value: Some(value.0), max_value: Some(value.0), extensible: true }
  }
}

impl<'a> From<(i128, RangeParticle, i128, ExtensionParticle)> for Constraint {
  fn from(value: (i128, RangeParticle, i128, ExtensionParticle)) -> Self {
    Self { min_value: Some(value.0), max_value: Some(value.2), extensible: true }
  }
}
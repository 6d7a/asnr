/// Comment tokens
pub const BLOCK_COMMENT_BEGIN: &'static [u8] = b"/*";
pub const BLOCK_COMMENT_CONTINUED_LINE: &'static [u8] = b"*";
pub const BLOCK_COMMENT_END: &'static [u8] = b"*/";
pub const LINE_COMMENT: &'static [u8] = b"//";

/// Bracket tokens
pub const LEFT_PARENTHESIS: char = '(';
pub const RIGHT_PARENTHESIS: char = ')';
pub const LEFT_BRACKET: char = '[';
pub const RIGHT_BRACKET: char = ']';
pub const LEFT_BRACE: char = '{';
pub const RIGHT_BRACE: char = '}';
pub const LEFT_CHEVRON: char = '<';
pub const RIGHT_CHEVRON: char = '>';

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
pub const COMMA: char = ',';

pub trait Quote<'a> {
    fn quote(&self) -> &'a str;
}

#[derive(Debug, PartialEq)]
pub struct ToplevelDeclaration {
    pub comments: String,
    pub name: String,
    pub r#type: ASN1Type,
}

impl From<(&str, &str, ASN1Type)> for ToplevelDeclaration {
    fn from(value: (&str, &str, ASN1Type)) -> Self {
        Self { comments: value.0.into(), name: value.1.into(), r#type: value.2 }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASN1Type {
    Null,
    Boolean,
    Integer(AsnInteger),
    Real,
    BitString,
    OctetString,
    Ia5String,
    Utf8String,
    NumericString,
    VisibleString,
    Enumerated,
    Choice,
    Sequence,
    SequenceOf,
    Set,
    SetOf,
}

pub struct AsnNull {}

pub struct AsnBoolean {}

#[derive(Debug, Clone, PartialEq)]
pub struct AsnInteger {
    pub constraint: Option<Constraint>,
    pub distinguished_values: Option<Vec<DistinguishedValue>>
}

impl Default for AsnInteger {
    fn default() -> Self {
        Self { constraint: None, distinguished_values: None }
    }
}

#[cfg(test)]
impl From<Constraint> for AsnInteger {
  fn from(value: Constraint) -> Self {
      Self { constraint: Some(value), distinguished_values: None }
  }
}


impl From<(&str, Option<Vec<DistinguishedValue>>, Option<Constraint>)> for AsnInteger {
  fn from(value: (&str, Option<Vec<DistinguishedValue>>, Option<Constraint>)) -> Self {
      Self {
          constraint: value.2,
          distinguished_values: value.1,
      }
  }
}

pub struct AsnReal {}

pub struct AsnBitString {}

pub struct AsnOctetString {}

pub struct AsnIa5String {}

pub struct AsnUtf8String {}

pub struct AsnNumericString {}

pub struct AsnVisibleString {}

pub struct AsnEnumerated {}

pub struct AsnChoice {}

pub struct AsnSequence {}

pub struct AsnSequenceOf {}

pub struct AsnSet {}

pub struct AsnSetOf {}

#[derive(Debug, Clone, PartialEq)]
pub struct DistinguishedValue {
  pub name: String,
  pub value: i128
}

impl From<(&str, i128)> for DistinguishedValue {
    fn from(value: (&str, i128)) -> Self {
        Self { name: value.0.into(), value: value.1 }
    }
}

#[derive(Debug)]
pub struct RangeParticle();

#[derive(Debug)]
pub struct ExtensionParticle();

// TODO: Add check whether min is smaller than max
#[derive(Debug, Clone, PartialEq)]
pub struct Constraint {
    pub min_value: Option<i128>,
    pub max_value: Option<i128>,
    pub extensible: bool,
}

impl Constraint {
    pub fn new(min: Option<i128>, max: Option<i128>, extensible: bool) -> Self {
        Self {
            min_value: min,
            max_value: max,
            extensible,
        }
    }
}

impl<'a> From<i128> for Constraint {
    fn from(value: i128) -> Self {
        Self {
            min_value: Some(value),
            max_value: Some(value),
            extensible: false,
        }
    }
}

impl<'a> From<(i128, RangeParticle, i128)> for Constraint {
    fn from(value: (i128, RangeParticle, i128)) -> Self {
        Self {
            min_value: Some(value.0),
            max_value: Some(value.2),
            extensible: false,
        }
    }
}

impl<'a> From<(i128, ExtensionParticle)> for Constraint {
    fn from(value: (i128, ExtensionParticle)) -> Self {
        Self {
            min_value: Some(value.0),
            max_value: Some(value.0),
            extensible: true,
        }
    }
}

impl<'a> From<(i128, RangeParticle, i128, ExtensionParticle)> for Constraint {
    fn from(value: (i128, RangeParticle, i128, ExtensionParticle)) -> Self {
        Self {
            min_value: Some(value.0),
            max_value: Some(value.2),
            extensible: true,
        }
    }
}

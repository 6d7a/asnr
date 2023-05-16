/// Comment tokens
pub const C_STYLE_BLOCK_COMMENT_BEGIN: &'static str = "/*";
pub const C_STYLE_BLOCK_COMMENT_CONTINUED_LINE: char = '*';
pub const C_STYLE_BLOCK_COMMENT_END: &'static str = "*/";
pub const C_STYLE_LINE_COMMENT: &'static str = "//";
pub const ASN1_COMMENT: &'static str = "--";

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
pub const NULL: &'static str = "NULL";
pub const BOOLEAN: &'static str = "BOOLEAN";
pub const INTEGER: &'static str = "INTEGER";
pub const REAL: &'static str = "REAL";
pub const BIT_STRING: &'static str = "BIT STRING";
pub const OCTET_STRING: &'static str = "OCTET STRING";
pub const IA5_STRING: &'static str = "IA5String";
pub const UTF8_STRING: &'static str = "UTF8String";
pub const NUMERIC_STRING: &'static str = "NumericString";
pub const VISIBLE_STRING: &'static str = "VisibleString";
pub const ENUMERATED: &'static str = "ENUMERATED";
pub const CHOICE: &'static str = "CHOICE";
pub const SEQUENCE: &'static str = "SEQUENCE";
pub const SEQUENCE_OF: &'static str = "SEQUENCE OF";
pub const SET: &'static str = "SET";
pub const SET_OF: &'static str = "SET OF";

pub const SIZE: &'static str = "SIZE";
pub const DEFAULT: &'static str = "DEFAULT";
pub const ASSIGN: &'static str = "::=";
pub const RANGE: &'static str = "..";
pub const EXTENSION: &'static str = "...";
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
    BitString(AsnBitString),
    OctetString,
    Ia5String,
    Utf8String,
    NumericString,
    VisibleString,
    Enumerated(AsnEnumerated),
    Choice,
    Sequence,
    SequenceOf,
    Set,
    SetOf,
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct AsnBitString {
  pub constraint: Option<Constraint>,
}

impl From<Option<Constraint>> for AsnBitString {
    fn from(value: Option<Constraint>) -> Self {
        AsnBitString { constraint: value }
    }
}

pub struct AsnOctetString {}

pub struct AsnIa5String {}

pub struct AsnUtf8String {}

pub struct AsnNumericString {}

pub struct AsnVisibleString {}

#[derive(Debug, Clone, PartialEq)]
pub struct AsnEnumerated {
  pub members: Vec<Enumeral>,
  pub extensible: bool
}

impl From<(Vec<Enumeral>, Option<ExtensionMarker>)> for AsnEnumerated {
    fn from(value: (Vec<Enumeral>, Option<ExtensionMarker>)) -> Self {
        AsnEnumerated { members: value.0, extensible: value.1.is_some() }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enumeral {
  pub name: String,
  pub description: Option<String>,
  pub index: u64,
}

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
pub struct RangeMarker();

#[derive(Debug)]
pub struct ExtensionMarker();

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

impl<'a> From<(i128, RangeMarker, i128)> for Constraint {
    fn from(value: (i128, RangeMarker, i128)) -> Self {
        Self {
            min_value: Some(value.0),
            max_value: Some(value.2),
            extensible: false,
        }
    }
}

impl<'a> From<(i128, ExtensionMarker)> for Constraint {
    fn from(value: (i128, ExtensionMarker)) -> Self {
        Self {
            min_value: Some(value.0),
            max_value: Some(value.0),
            extensible: true,
        }
    }
}

impl<'a> From<(i128, RangeMarker, i128, ExtensionMarker)> for Constraint {
    fn from(value: (i128, RangeMarker, i128, ExtensionMarker)) -> Self {
        Self {
            min_value: Some(value.0),
            max_value: Some(value.2),
            extensible: true,
        }
    }
}

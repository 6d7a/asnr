use crate::validator::error::{ValidatorError, ValidatorErrorType};

// Comment tokens
pub const C_STYLE_BLOCK_COMMENT_BEGIN: &'static str = "/*";
pub const C_STYLE_BLOCK_COMMENT_CONTINUED_LINE: char = '*';
pub const C_STYLE_BLOCK_COMMENT_END: &'static str = "*/";
pub const C_STYLE_LINE_COMMENT: &'static str = "//";
pub const ASN1_COMMENT: &'static str = "--";

// Bracket tokens
pub const LEFT_PARENTHESIS: char = '(';
pub const RIGHT_PARENTHESIS: char = ')';
pub const LEFT_BRACKET: char = '[';
pub const RIGHT_BRACKET: char = ']';
pub const LEFT_BRACE: char = '{';
pub const RIGHT_BRACE: char = '}';
pub const LEFT_CHEVRON: char = '<';
pub const RIGHT_CHEVRON: char = '>';

// Type tokens
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

// Value tokens
pub const TRUE: &'static str = "TRUE";
pub const FALSE: &'static str = "FALSE";

pub const SIZE: &'static str = "SIZE";
pub const DEFAULT: &'static str = "DEFAULT";
pub const OPTIONAL: &'static str = "OPTIONAL";
pub const ASSIGN: &'static str = "::=";
pub const RANGE: &'static str = "..";
pub const EXTENSION: &'static str = "...";
pub const COMMA: char = ',';
pub const SINGLE_QUOTE: char = '\'';

#[derive(Debug, PartialEq)]
pub struct ToplevelDeclaration {
    pub comments: String,
    pub name: String,
    pub r#type: ASN1Type,
}

impl From<(&str, &str, ASN1Type)> for ToplevelDeclaration {
    fn from(value: (&str, &str, ASN1Type)) -> Self {
        Self {
            comments: value.0.into(),
            name: value.1.into(),
            r#type: value.2,
        }
    }
}

/// The possible types of an ASN1 data element.
/// In addition, the `ElsewhereDeclaredType` enumeral denotes an type
/// specified in the same or an imported ASN1 specification.
#[derive(Debug, Clone, PartialEq)]
pub enum ASN1Type {
    // Null,
    Boolean,
    Integer(AsnInteger),
    // Real,
    BitString(AsnBitString),
    OctetString(AsnOctetString),
    // Ia5String,
    // Utf8String,
    // NumericString,
    // VisibleString,
    Enumerated(AsnEnumerated),
    // Choice,
    Sequence(AsnSequence),
    // SequenceOf,
    // Set,
    // SetOf,
    ElsewhereDeclaredType(DeclarationElsewhere),
}

/// The possible types of an ASN1 value.
#[derive(Debug, Clone, PartialEq)]
pub enum ASN1Value {
    Boolean(bool),
    Integer(i128),
    String(String),
}

/// Representation of an ASN1 INTEGER data element
/// with corresponding constraints and distinguished values
#[derive(Debug, Clone, PartialEq)]
pub struct AsnInteger {
    pub constraint: Option<Constraint>,
    pub distinguished_values: Option<Vec<DistinguishedValue>>,
}

impl Default for AsnInteger {
    fn default() -> Self {
        Self {
            constraint: None,
            distinguished_values: None,
        }
    }
}

#[cfg(test)]
impl From<Constraint> for AsnInteger {
    fn from(value: Constraint) -> Self {
        Self {
            constraint: Some(value),
            distinguished_values: None,
        }
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

/// Representation of an ASN1 BIT STRING data element
/// with corresponding constraints and distinguished values
/// defining the individual bits
#[derive(Debug, Clone, PartialEq)]
pub struct AsnBitString {
    pub constraint: Option<Constraint>,
    pub distinguished_values: Option<Vec<DistinguishedValue>>,
}

impl From<(Option<Vec<DistinguishedValue>>, Option<Constraint>)> for AsnBitString {
    fn from(value: (Option<Vec<DistinguishedValue>>, Option<Constraint>)) -> Self {
        AsnBitString {
            constraint: value.1,
            distinguished_values: value.0,
        }
    }
}

/// Representation of an ASN1 OCTET STRING data element
/// with corresponding constraints
#[derive(Debug, Clone, PartialEq)]
pub struct AsnOctetString {
    pub constraint: Option<Constraint>,
}

impl From<Option<Constraint>> for AsnOctetString {
    fn from(value: Option<Constraint>) -> Self {
        AsnOctetString { constraint: value }
    }
}

/// Representation of an ASN1 SEQUENCE data element
/// with corresponding members and extension information
#[derive(Debug, Clone, PartialEq)]
pub struct AsnSequence {
    pub extensible: bool,
    pub members: Vec<SequenceMember>,
}

impl From<(Vec<SequenceMember>, Option<ExtensionMarker>)> for AsnSequence {
    fn from(value: (Vec<SequenceMember>, Option<ExtensionMarker>)) -> Self {
        AsnSequence {
            extensible: value.1.is_some(),
            members: value.0,
        }
    }
}

/// Representation of an single ASN1 SEQUENCE member
#[derive(Debug, Clone, PartialEq)]
pub struct SequenceMember {
    pub name: String,
    pub r#type: ASN1Type,
    pub default_value: Option<ASN1Value>,
    pub is_optional: bool,
}

impl From<(&str, ASN1Type, Option<OptionalMarker>, Option<ASN1Value>)> for SequenceMember {
    fn from(value: (&str, ASN1Type, Option<OptionalMarker>, Option<ASN1Value>)) -> Self {
        SequenceMember {
            name: value.0.into(),
            r#type: value.1,
            is_optional: value.2.is_some() || value.3.is_some(),
            default_value: value.3,
        }
    }
}

/// Representation of an ASN1 SEQUENCE data element
/// with corresponding enumerals and extension information
#[derive(Debug, Clone, PartialEq)]
pub struct AsnEnumerated {
    pub members: Vec<Enumeral>,
    pub extensible: bool,
}

impl From<(Vec<Enumeral>, Option<ExtensionMarker>)> for AsnEnumerated {
    fn from(value: (Vec<Enumeral>, Option<ExtensionMarker>)) -> Self {
        AsnEnumerated {
            members: value.0,
            extensible: value.1.is_some(),
        }
    }
}

/// Representation of a single member/enumeral of an ASN1
/// ENUMERATED data element
#[derive(Debug, Clone, PartialEq)]
pub struct Enumeral {
    pub name: String,
    pub description: Option<String>,
    pub index: u64,
}

/// Representation of a ASN1 distinguished value,
/// as seen in some INTEGER declarations
#[derive(Debug, Clone, PartialEq)]
pub struct DistinguishedValue {
    pub name: String,
    pub value: i128,
}

impl From<(&str, i128)> for DistinguishedValue {
    fn from(value: (&str, i128)) -> Self {
        Self {
            name: value.0.into(),
            value: value.1,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct OptionalMarker();

impl From<&str> for OptionalMarker {
    fn from(_: &str) -> Self {
        OptionalMarker()
    }
}

/// Intermediate placeholder for a type declared in
/// some other part of the ASN1 specification that is
/// being parsed or in one of its imports.
#[derive(Debug, Clone, PartialEq)]
pub struct DeclarationElsewhere(pub String);

impl From<&str> for DeclarationElsewhere {
    fn from(value: &str) -> Self {
        DeclarationElsewhere(value.into())
    }
}

#[derive(Debug)]
pub struct RangeMarker();

#[derive(Debug)]
pub struct ExtensionMarker();

// TODO: Add check whether min is smaller than max
/// Representation of a constraint used for subtyping
/// in ASN1 specifications
#[derive(Debug, Clone, PartialEq)]
pub struct Constraint {
    pub min_value: Option<i128>,
    pub max_value: Option<i128>,
    pub extensible: bool,
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

//! The `asnr-grammar` crate describes the single elements
//! of the ASN1 notation.
//! It includes constants for the various ASN1 keywords
//! and types to represent the single ASN1 data elements
//! from which the generator module produces de-/encodable
//! types.
//!
#![no_std]
extern crate alloc;

pub mod subtyping;
pub mod types;

use alloc::{format, string::String, vec, vec::Vec};
use subtyping::Constraint;
use types::*;

// Comment tokens
pub const BLOCK_COMMENT_START: &'static str = "/*";
pub const BLOCK_COMMENT_END: &'static str = "*/";
pub const LINE_COMMENT: &'static str = "--";

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
pub const TELETEX_STRING: &'static str = "TeletexString";
pub const VIDEOTEX_STRING: &'static str = "VideotexString";
pub const GRAPHIC_STRING: &'static str = "GraphicString";
pub const GENERAL_STRING: &'static str = "GeneralString";
pub const UNIVERSAL_STRING: &'static str = "UniversalString";
pub const BMP_STRING: &'static str = "BMPString";
pub const PRINTABLE_STRING: &'static str = "PrintableString";
pub const ENUMERATED: &'static str = "ENUMERATED";
pub const CHOICE: &'static str = "CHOICE";
pub const SEQUENCE: &'static str = "SEQUENCE";
pub const OF: &'static str = "OF";
pub const SET: &'static str = "SET";
pub const SET_OF: &'static str = "SET OF";
pub const OBJECT_IDENTIFIER: &'static str = "OBJECT IDENTIFIER";

// Tagging tokens
pub const UNIVERSAL: &'static str = "UNIVERSAL";
pub const PRIVATE: &'static str = "PRIVATE";
pub const APPLICATION: &'static str = "APPLICATION";

// Value tokens
pub const TRUE: &'static str = "TRUE";
pub const FALSE: &'static str = "FALSE";

// Header tokens
pub const BEGIN: &'static str = "BEGIN";
pub const DEFINITIONS: &'static str = "DEFINITIONS";
pub const AUTOMATIC: &'static str = "AUTOMATIC";
pub const EXPLICIT: &'static str = "EXPLICIT";
pub const IMPLICIT: &'static str = "IMPLICIT";
pub const INSTRUCTIONS: &'static str = "INSTRUCTIONS";
pub const TAGS: &'static str = "TAGS";
pub const EXTENSIBILITY_IMPLIED: &'static str = "EXTENSIBILITY IMPLIED";

// Subtyping tokens
pub const SIZE: &'static str = "SIZE";
pub const DEFAULT: &'static str = "DEFAULT";
pub const OPTIONAL: &'static str = "OPTIONAL";
pub const WITH_COMPONENTS: &'static str = "WITH COMPONENTS";
pub const WITH_COMPONENT: &'static str = "WITH COMPONENT";
pub const UNION: &'static str = "UNION";
pub const PIPE: &'static str = "|";
pub const EXCEPT: &'static str = "EXCEPT";
pub const INTERSECTION: &'static str = "INTERSECTION";
pub const CARET: &'static str = "^";
pub const ABSENT: &'static str = "ABSENT";
pub const PRESENT: &'static str = "PRESENT";

pub const ASSIGN: &'static str = "::=";
pub const RANGE: &'static str = "..";
pub const ELLIPSIS: &'static str = "...";
pub const COMMA: char = ',';
pub const SINGLE_QUOTE: char = '\'';

/// The `Quote` trait serves to convert a structure
/// into a stringified rust representation of its initialization.
///
/// #### Example
/// Let's say we have
/// ```rust
/// # use asnr_grammar::Quote;
///
/// pub struct Foo {
///   pub bar: u8
/// }
/// // The implementation of `Quote` for `Foo` would look like this:
/// impl Quote for Foo {
///   fn quote(&self) -> String {
///     format!("Foo {{ bar: {} }}", self.bar)
///   }
/// }
/// ```
pub trait Quote {
    /// Returns a stringified representation of the implementing struct's initialization
    ///
    /// #### Example
    /// ```rust
    /// # use asnr_grammar::Quote;
    /// # pub struct Foo { pub bar: u8 }
    /// # impl Quote for Foo {
    /// #  fn quote(&self) -> String { format!("Foo {{ bar: {} }}", self.bar) }
    /// # }
    /// let foo = Foo { bar: 1 };
    /// assert_eq!("Foo { bar: 1 }".to_string(), foo.quote());
    /// ```
    fn quote(&self) -> String;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EncodingReferenceDefault(pub String);

impl From<&str> for EncodingReferenceDefault {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaggingEnvironment {
    AUTOMATIC,
    IMPLICIT,
    EXPLICIT,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExtensibilityEnvironment {
    IMPLIED,
    EXPLICIT,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub name: String,
    pub module_identifier: ObjectIdentifier,
    pub encoding_reference_default: EncodingReferenceDefault,
    pub tagging_environment: TaggingEnvironment,
    pub extensibility_environment: ExtensibilityEnvironment,
}

impl
    From<(
        &str,
        ObjectIdentifier,
        (
            EncodingReferenceDefault,
            TaggingEnvironment,
            ExtensibilityEnvironment,
        ),
    )> for Header
{
    fn from(
        value: (
            &str,
            ObjectIdentifier,
            (
                EncodingReferenceDefault,
                TaggingEnvironment,
                ExtensibilityEnvironment,
            ),
        ),
    ) -> Self {
        Self {
            name: value.0.into(),
            module_identifier: value.1,
            encoding_reference_default: value.2 .0,
            tagging_environment: value.2 .1,
            extensibility_environment: value.2 .2,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectIdentifier(pub Vec<ObjectIdentifierArc>);

impl From<Vec<ObjectIdentifierArc>> for ObjectIdentifier {
    fn from(value: Vec<ObjectIdentifierArc>) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectIdentifierArc {
    pub name: Option<String>,
    pub number: Option<u128>,
}

impl From<u128> for ObjectIdentifierArc {
    fn from(value: u128) -> Self {
        Self {
            name: None,
            number: Some(value),
        }
    }
}

impl From<&str> for ObjectIdentifierArc {
    fn from(value: &str) -> Self {
        Self {
            name: Some(value.into()),
            number: None,
        }
    }
}

impl From<(&str, u128)> for ObjectIdentifierArc {
    fn from(value: (&str, u128)) -> Self {
        Self {
            name: Some(value.0.into()),
            number: Some(value.1),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToplevelDeclaration {
    pub comments: String,
    pub name: String,
    pub r#type: ASN1Type,
}

impl From<(Vec<&str>, &str, ASN1Type)> for ToplevelDeclaration {
    fn from(value: (Vec<&str>, &str, ASN1Type)) -> Self {
        Self {
            comments: value.0.join("\n"),
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
    Null,
    Boolean,
    Integer(AsnInteger),
    // Real,
    BitString(AsnBitString),
    CharacterString(AsnCharacterString),
    Enumerated(AsnEnumerated),
    Choice(AsnChoice),
    Sequence(AsnSequence),
    SequenceOf(AsnSequenceOf),
    // Set,
    // SetOf,
    ElsewhereDeclaredType(DeclarationElsewhere),
}

/// The types of an ASN1 character strings.
#[derive(Debug, Clone, PartialEq)]
pub enum CharacterStringType {
    OctetString,
    NumericString,
    VisibleString,
    IA5String,
    TeletexString,
    VideotexString,
    GraphicString,
    GeneralString,
    UniversalString,
    UTF8String,
    BMPString,
    PrintableString,
}

impl From<&str> for CharacterStringType {
    fn from(value: &str) -> Self {
        match value {
            IA5_STRING => Self::IA5String,
            OCTET_STRING => Self::OctetString,
            NUMERIC_STRING => Self::NumericString,
            VISIBLE_STRING => Self::VisibleString,
            TELETEX_STRING => Self::TeletexString,
            VIDEOTEX_STRING => Self::VideotexString,
            GRAPHIC_STRING => Self::GraphicString,
            GENERAL_STRING => Self::GeneralString,
            UNIVERSAL_STRING => Self::UniversalString,
            BMP_STRING => Self::BMPString,
            PRINTABLE_STRING => Self::PrintableString,
            _ => Self::UTF8String,
        }
    }
}

impl Quote for ASN1Type {
    fn quote(&self) -> String {
        match self {
            ASN1Type::Null => "ASN1Type::Null".into(),
            ASN1Type::Boolean => "ASN1Type::Boolean".into(),
            ASN1Type::Integer(i) => format!("ASN1Type::Integer({})", i.quote()),
            ASN1Type::BitString(b) => format!("ASN1Type::BitString({})", b.quote()),
            ASN1Type::CharacterString(o) => format!("ASN1Type::CharacterString({})", o.quote()),
            ASN1Type::Enumerated(e) => format!("ASN1Type::Enumerated({})", e.quote()),
            ASN1Type::SequenceOf(s) => format!("ASN1Type::SequenceOf({})", s.quote()),
            ASN1Type::Sequence(s) => format!("ASN1Type::Sequence({})", s.quote()),
            ASN1Type::Choice(c) => format!("ASN1Type::Choice({})", c.quote()),
            ASN1Type::ElsewhereDeclaredType(els) => {
                format!("ASN1Type::ElsewhereDeclaredType({})", els.quote())
            }
        }
    }
}

/// The possible types of an ASN1 value.
#[derive(Debug, Clone, PartialEq)]
pub enum ASN1Value {
    Null,
    Boolean(bool),
    Integer(i128),
    String(String),
    BitString(Vec<bool>),
    EnumeratedValue(String),
    ElsewhereDeclaredValue(String),
}

impl Quote for ASN1Value {
    fn quote(&self) -> String {
        match self {
            ASN1Value::Null => String::from("ASN1Value::Null"),
            ASN1Value::Boolean(b) => format!("ASN1Value::Boolean({})", b),
            ASN1Value::Integer(i) => format!("ASN1Value::Integer({})", i),
            ASN1Value::String(s) => format!("ASN1Value::String(\"{}\".into())", s),
            ASN1Value::BitString(s) => format!(
                "ASN1Value::BitString(vec![{}])",
                s.iter()
                    .map(|b| {
                        if *b {
                            String::from("true")
                        } else {
                            String::from("false")
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            ASN1Value::EnumeratedValue(s) => {
                format!("ASN1Value::EnumeratedValue(\"{}\".into())", s)
            }
            ASN1Value::ElsewhereDeclaredValue(s) => {
                format!("ASN1Value::ElsewhereDeclaredValue(\"{}\".into())", s)
            }
        }
    }
}

/// Intermediate placeholder for a type declared in
/// some other part of the ASN1 specification that is
/// being parsed or in one of its imports.
#[derive(Debug, Clone, PartialEq)]
pub struct DeclarationElsewhere {
    pub identifier: String,
    pub constraints: Vec<Constraint>,
}

impl From<(&str, Option<Vec<Constraint>>)> for DeclarationElsewhere {
    fn from(value: (&str, Option<Vec<Constraint>>)) -> Self {
        DeclarationElsewhere {
            identifier: value.0.into(),
            constraints: value.1.unwrap_or(vec![]),
        }
    }
}

impl Quote for DeclarationElsewhere {
    fn quote(&self) -> String {
        format!(
            "DeclarationElsewhere {{ identifier: \"{}\".into(), constraints: vec![{}] }}",
            self.identifier,
            self.constraints
                .iter()
                .map(|c| c.quote())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

/// Tag classes
#[derive(Debug, Clone, PartialEq)]
pub enum TagClass {
    Universal,
    Application,
    Private,
    ContextSpecific,
}

/// Representation of a tag
#[derive(Debug, Clone, PartialEq)]
pub struct AsnTag {
    pub tag_class: TagClass,
    pub id: u64,
}

impl Quote for AsnTag {
    fn quote(&self) -> String {
        format!(
            "AsnTag {{ tag_class: TagClass::{:?}, id: {} }}",
            self.tag_class, self.id
        )
    }
}

impl From<(Option<&str>, u64)> for AsnTag {
    fn from(value: (Option<&str>, u64)) -> Self {
        let tag_class = match value.0 {
            Some("APPLICATION") => TagClass::Application,
            Some("UNIVERSAL") => TagClass::Universal,
            Some("PRIVATE") => TagClass::Private,
            _ => TagClass::ContextSpecific,
        };
        AsnTag {
            tag_class,
            id: value.1,
        }
    }
}

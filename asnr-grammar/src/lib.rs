//! The `asnr-grammar` crate describes the single elements
//! of the ASN1 notation.
//! It includes constants for the various ASN1 keywords
//! and types to represent the single ASN1 data elements
//! from which the generator module produces de-/encodable
//! types.
//!
#![no_std]
extern crate alloc;
extern crate asnr_traits;
#[macro_use]
extern crate asnr_grammar_derive;

use asnr_traits::Declare;

pub mod information_object;
pub mod parameterization;
pub mod subtyping;
pub mod types;

use alloc::{
  borrow::ToOwned,
  format,
  string::{String, ToString},
  vec,
  vec::Vec,
};
use information_object::{ToplevelInformationDeclaration, InformationObjectFieldReference};
use parameterization::Parameterization;
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
pub const END: &'static str = "END";
pub const DEFINITIONS: &'static str = "DEFINITIONS";
pub const AUTOMATIC: &'static str = "AUTOMATIC";
pub const EXPLICIT: &'static str = "EXPLICIT";
pub const IMPLICIT: &'static str = "IMPLICIT";
pub const IMPORTS: &'static str = "IMPORTS";
pub const FROM: &'static str = "FROM";
pub const INSTRUCTIONS: &'static str = "INSTRUCTIONS";
pub const TAGS: &'static str = "TAGS";
pub const EXTENSIBILITY_IMPLIED: &'static str = "EXTENSIBILITY IMPLIED";
pub const WITH_SUCCESSORS: &'static str = "WITH SUCCESSORS";
pub const SEMICOLON: char = ';';

// Information Object Class tokens
pub const AMPERSAND: char = '&';
pub const CLASS: &'static str = "CLASS";
pub const UNIQUE: &'static str = "UNIQUE";
pub const WITH_SYNTAX: &'static str = "WITH SYNTAX";
pub const AT: char = '@';
pub const DOT: char = '.';

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
pub const COLON: char = ':';
pub const SINGLE_QUOTE: char = '\'';

// invalid syntax word tokens
pub const ABSTRACT_SYNTAX: &'static str = "ABSTRACT-SYNTAX";
pub const BIT: &'static str = "BIT";
pub const CHARACTER: &'static str = "CHARACTER";
pub const CONTAINING: &'static str = "CONTAINING";
pub const DATE: &'static str = "DATE";
pub const DATE_TIME: &'static str = "DATE-TIME";
pub const DURATION: &'static str = "DURATION";
pub const EMBEDDED: &'static str = "EMBEDDED";
pub const EXTERNAL: &'static str = "EXTERNAL";
pub const INSTANCE: &'static str = "INSTANCE";
pub const MINUS_INFINITY: &'static str = "MINUS-INFINITY";
pub const NOT_A_NUMBER: &'static str = "NOT-A-NUMBER";
pub const OBJECT: &'static str = "OBJECT";
pub const OCTET: &'static str = "OCTET";
pub const OID_IRI: &'static str = "OID-IRI";
pub const PLUS_INFINITY: &'static str = "PLUS-INFINITY";
pub const RELATIVE_OID: &'static str = "RELATIVE-OID";
pub const RELATIVE_OID_IRI: &'static str = "RELATIVE-OID-IRI";
pub const TIME: &'static str = "TIME";
pub const TIME_OF_DAY: &'static str = "TIME-OF-DAY";
pub const TYPE_IDENTIFIER: &'static str = "TYPE-IDENTIFIER";

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
pub struct Import {
    pub types: Vec<String>,
    pub origin_name: String,
    pub origin_identifier: ObjectIdentifier,
    pub with_successors: bool,
}

impl From<(Vec<&str>, (&str, ObjectIdentifier, Option<&str>))> for Import {
    fn from(value: (Vec<&str>, (&str, ObjectIdentifier, Option<&str>))) -> Self {
        Self {
            types: value.0.into_iter().map(|s| String::from(s)).collect(),
            origin_name: value.1 .0.into(),
            origin_identifier: value.1 .1,
            with_successors: value.1 .2.is_some(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleReference {
    pub name: String,
    pub module_identifier: ObjectIdentifier,
    pub encoding_reference_default: Option<EncodingReferenceDefault>,
    pub tagging_environment: TaggingEnvironment,
    pub extensibility_environment: ExtensibilityEnvironment,
    pub imports: Vec<Import>,
}

impl
    From<(
        &str,
        ObjectIdentifier,
        (
            Option<EncodingReferenceDefault>,
            TaggingEnvironment,
            ExtensibilityEnvironment,
        ),
        Option<Vec<Import>>,
    )> for ModuleReference
{
    fn from(
        value: (
            &str,
            ObjectIdentifier,
            (
                Option<EncodingReferenceDefault>,
                TaggingEnvironment,
                ExtensibilityEnvironment,
            ),
            Option<Vec<Import>>,
        ),
    ) -> Self {
        Self {
            name: value.0.into(),
            module_identifier: value.1,
            encoding_reference_default: value.2 .0,
            tagging_environment: value.2 .1,
            extensibility_environment: value.2 .2,
            imports: value.3.unwrap_or(vec![]),
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

#[derive(Debug, Clone, PartialEq, Declare)]
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
pub enum ToplevelDeclaration {
    Type(ToplevelTypeDeclaration),
    Value(ToplevelValueDeclaration),
    Information(ToplevelInformationDeclaration),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToplevelValueDeclaration {
    pub comments: String,
    pub name: String,
    pub type_name: String,
    pub value: ASN1Value,
}

impl From<(Vec<&str>, &str, &str, ASN1Value)> for ToplevelValueDeclaration {
    fn from(value: (Vec<&str>, &str, &str, ASN1Value)) -> Self {
        Self {
            comments: value.0.join("\n"),
            name: value.1.into(),
            type_name: value.2.into(),
            value: value.3,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToplevelTypeDeclaration {
    pub comments: String,
    pub name: String,
    pub r#type: ASN1Type,
    pub parameterization: Option<Parameterization>,
}

impl From<(Vec<&str>, &str, Option<Parameterization>, ASN1Type)> for ToplevelTypeDeclaration {
    fn from(value: (Vec<&str>, &str, Option<Parameterization>, ASN1Type)) -> Self {
        Self {
            comments: value.0.join("\n"),
            name: value.1.into(),
            parameterization: value.2,
            r#type: value.3,
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
    Integer(Integer),
    // Real,
    BitString(BitString),
    CharacterString(CharacterString),
    Enumerated(Enumerated),
    Choice(Choice),
    Sequence(Sequence),
    SequenceOf(SequenceOf),
    // Set,
    // SetOf,
    ElsewhereDeclaredType(DeclarationElsewhere),
    InformationObjectFieldReference(InformationObjectFieldReference),
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

impl asnr_traits::Declare for ASN1Type {
    fn declare(&self) -> String {
        match self {
            ASN1Type::Null => "ASN1Type::Null".into(),
            ASN1Type::Boolean => "ASN1Type::Boolean".into(),
            ASN1Type::Integer(i) => format!("ASN1Type::Integer({})", i.declare()),
            ASN1Type::BitString(b) => format!("ASN1Type::BitString({})", b.declare()),
            ASN1Type::CharacterString(o) => format!("ASN1Type::CharacterString({})", o.declare()),
            ASN1Type::Enumerated(e) => format!("ASN1Type::Enumerated({})", e.declare()),
            ASN1Type::SequenceOf(s) => format!("ASN1Type::SequenceOf({})", s.declare()),
            ASN1Type::Sequence(s) => format!("ASN1Type::Sequence({})", s.declare()),
            ASN1Type::Choice(c) => format!("ASN1Type::Choice({})", c.declare()),
            ASN1Type::ElsewhereDeclaredType(els) => {
                format!("ASN1Type::ElsewhereDeclaredType({})", els.declare())
            }
            ASN1Type::InformationObjectFieldReference(iofr) => format!("ASN1Type::InformationObjectFieldReference({})", iofr.declare()),
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

impl ToString for ASN1Value {
    fn to_string(&self) -> String {
        match self {
            ASN1Value::Null => "ASN1_NULL".to_owned(),
            ASN1Value::Boolean(b) => format!("{}", b),
            ASN1Value::Integer(i) => format!("{}", i),
            ASN1Value::String(s) => s.clone(),
            ASN1Value::BitString(_) => todo!(),
            ASN1Value::EnumeratedValue(e) => e.clone(),
            ASN1Value::ElsewhereDeclaredValue(e) => e.clone(),
        }
    }
}

impl asnr_traits::Declare for ASN1Value {
    fn declare(&self) -> String {
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

impl asnr_traits::Declare for DeclarationElsewhere {
    fn declare(&self) -> String {
        format!(
            "DeclarationElsewhere {{ identifier: \"{}\".into(), constraints: vec![{}] }}",
            self.identifier,
            self.constraints
                .iter()
                .map(|c| c.declare())
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

impl asnr_traits::Declare for AsnTag {
    fn declare(&self) -> String {
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

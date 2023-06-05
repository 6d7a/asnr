//! The `asnr-grammar` crate describes the single elements
//! of the ASN1 notation.
//! It includes constants for the various ASN1 keywords
//! and types to represent the single ASN1 data elements
//! from which the generator module produces de-/encodable
//! types.

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
pub const OF: &'static str = "OF";
pub const SET: &'static str = "SET";
pub const SET_OF: &'static str = "SET OF";
pub const OBJECT_IDENTIFIER: &'static str = "OBJECT IDENTIFIER";

// Value tokens
pub const TRUE: &'static str = "TRUE";
pub const FALSE: &'static str = "FALSE";

// Header tokens
pub const BEGIN: &'static str = "BEGIN";
pub const DEFINITIONS: &'static str = "DEFINITIONS";
pub const AUTOMATIC: &'static str = "AUTOMATIC";
pub const EXPLICIT: &'static str = "EXPLICIT";
pub const IMPLICIT: &'static str = "IMPLICIT";
pub const TAGS: &'static str = "TAGS";
pub const EXTENSIBILITY_IMPLIED: &'static str = "EXTENSIBILITY IMPLIED";

pub const SIZE: &'static str = "SIZE";
pub const DEFAULT: &'static str = "DEFAULT";
pub const OPTIONAL: &'static str = "OPTIONAL";
pub const ASSIGN: &'static str = "::=";
pub const RANGE: &'static str = "..";
pub const EXTENSION: &'static str = "...";
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
    pub tagging_environment: TaggingEnvironment,
    pub extensibility_environment: ExtensibilityEnvironment,
}

impl
    From<(
        &str,
        ObjectIdentifier,
        (TaggingEnvironment, ExtensibilityEnvironment),
    )> for Header
{
    fn from(
        value: (
            &str,
            ObjectIdentifier,
            (TaggingEnvironment, ExtensibilityEnvironment),
        ),
    ) -> Self {
        Self {
            name: value.0.into(),
            module_identifier: value.1,
            tagging_environment: value.2 .0,
            extensibility_environment: value.2 .1,
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
    SequenceOf(AsnSequenceOf),
    // Set,
    // SetOf,
    ElsewhereDeclaredType(DeclarationElsewhere),
}

impl Quote for ASN1Type {
    fn quote(&self) -> String {
        match self {
            ASN1Type::Boolean => "ASN1Type::Boolean".into(),
            ASN1Type::Integer(i) => format!("ASN1Type::Integer({})", i.quote()),
            ASN1Type::BitString(b) => format!("ASN1Type::BitString({})", b.quote()),
            ASN1Type::OctetString(o) => format!("ASN1Type::OctetString({})", o.quote()),
            ASN1Type::Enumerated(e) => format!("ASN1Type::Enumerated({})", e.quote()),
            ASN1Type::SequenceOf(s) => format!("ASN1Type::SequenceOf({})", s.quote()),
            ASN1Type::Sequence(s) => format!("ASN1Type::Sequence({})", s.quote()),
            ASN1Type::ElsewhereDeclaredType(els) => {
                format!("ASN1Type::ElsewhereDeclaredType({})", els.quote())
            }
        }
    }
}

/// The possible types of an ASN1 value.
#[derive(Debug, Clone, PartialEq)]
pub enum ASN1Value {
    Boolean(bool),
    Integer(i128),
    String(String),
}

impl Quote for ASN1Value {
    fn quote(&self) -> String {
        match self {
            ASN1Value::Boolean(b) => format!("ASN1Value::Boolean({})", b),
            ASN1Value::Integer(i) => format!("ASN1Value::Integer({})", i),
            ASN1Value::String(s) => format!("ASN1Value::String(\"{}\".into())", s),
        }
    }
}

/// Representation of an ASN1 INTEGER data element
/// with corresponding constraints and distinguished values
#[derive(Debug, Clone, PartialEq)]
pub struct AsnInteger {
    pub constraint: Option<Constraint>,
    pub distinguished_values: Option<Vec<DistinguishedValue>>,
}

impl Quote for AsnInteger {
    fn quote(&self) -> String {
        format!(
            "AsnInteger {{ constraint: {}, distinguished_values: {} }}",
            self.constraint
                .as_ref()
                .map_or("None".to_owned(), |c| "Some(".to_owned() + &c.quote() + ")"),
            self.distinguished_values
                .as_ref()
                .map_or("None".to_owned(), |c| "Some(vec![".to_owned()
                    + &c.iter()
                        .map(|dv| dv.quote())
                        .collect::<Vec<String>>()
                        .join(",")
                    + "])"),
        )
    }
}

impl Default for AsnInteger {
    fn default() -> Self {
        Self {
            constraint: None,
            distinguished_values: None,
        }
    }
}

impl From<Constraint> for AsnInteger {
    fn from(value: Constraint) -> Self {
        Self {
            constraint: Some(value),
            distinguished_values: None,
        }
    }
}

impl From<(Option<i128>, Option<i128>, bool)> for AsnInteger {
    fn from(value: (Option<i128>, Option<i128>, bool)) -> Self {
        Self {
            constraint: Some(Constraint {
                min_value: value.0,
                max_value: value.1,
                extensible: value.2,
            }),
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

impl Quote for AsnBitString {
    fn quote(&self) -> String {
        format!(
            "AsnBitString {{ constraint: {}, distinguished_values: {} }}",
            self.constraint
                .as_ref()
                .map_or("None".to_owned(), |c| "Some(".to_owned() + &c.quote() + ")"),
            self.distinguished_values
                .as_ref()
                .map_or("None".to_owned(), |c| "Some(vec![".to_owned()
                    + &c.iter()
                        .map(|dv| dv.quote())
                        .collect::<Vec<String>>()
                        .join(",")
                    + "])"),
        )
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

impl Quote for AsnOctetString {
    fn quote(&self) -> String {
        format!(
            "AsnOctetString {{ constraint: {} }}",
            self.constraint
                .as_ref()
                .map_or("None".to_owned(), |c| "Some(".to_owned() + &c.quote() + ")"),
        )
    }
}

/// Representation of an ASN1 SEQUENCE OF data element
/// with corresponding constraints and element type info
#[derive(Debug, Clone, PartialEq)]
pub struct AsnSequenceOf {
    pub constraint: Option<Constraint>,
    pub r#type: Box<ASN1Type>,
}

impl Quote for AsnSequenceOf {
  fn quote(&self) -> String {
      format!(
          "AsnSequenceOf {{ constraint: {}, r#type: {} }}",
          self.constraint
              .as_ref()
              .map_or("None".to_owned(), |c| "Some(".to_owned() + &c.quote() + ")"),
          self.r#type.quote(),
      )
  }
}

impl From<(Option<Constraint>, ASN1Type)> for AsnSequenceOf {
    fn from(value: (Option<Constraint>, ASN1Type)) -> Self {
        Self { constraint: value.0, r#type: Box::new(value.1) }
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

impl Quote for AsnSequence {
    fn quote(&self) -> String {
        format!(
            "AsnSequence {{ extensible: {}, members: vec![{}] }}",
            self.extensible,
            self.members
                .iter()
                .map(|m| m.quote())
                .collect::<Vec<String>>()
                .join(",")
        )
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

impl Quote for SequenceMember {
    fn quote(&self) -> String {
        format!(
          "SequenceMember {{ name: \"{}\".into(), is_optional: {}, r#type: {}, default_value: {} }}",
          self.name,
          self.is_optional,
          self.r#type.quote(),
          self.default_value.as_ref().map_or("None".to_string(), |d| "Some(".to_owned()
          + &d.quote()
          + ")")
      )
    }
}

/// Representation of an ASN1 ENUMERATED data element
/// with corresponding enumerals and extension information
#[derive(Debug, Clone, PartialEq)]
pub struct AsnEnumerated {
    pub members: Vec<Enumeral>,
    pub extensible: bool,
}

impl Quote for AsnEnumerated {
    fn quote(&self) -> String {
        format!(
            "AsnEnumerated {{ members: vec![{}], extensible: {} }}",
            self.members
                .iter()
                .map(|m| m.quote())
                .collect::<Vec<String>>()
                .join(","),
            self.extensible
        )
    }
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

impl Quote for Enumeral {
    fn quote(&self) -> String {
        format!(
            "Enumeral {{ name: \"{}\".into(), description: {}, index: {} }}",
            self.name,
            self.description
                .as_ref()
                .map_or("None".to_owned(), |d| "Some(\"".to_owned()
                    + d
                    + "\".into())"),
            self.index
        )
    }
}

/// Representation of a ASN1 distinguished value,
/// as seen in some INTEGER and BIT STRING declarations
#[derive(Debug, Clone, PartialEq)]
pub struct DistinguishedValue {
    pub name: String,
    pub value: i128,
}

impl Quote for DistinguishedValue {
    fn quote(&self) -> String {
        format!(
            "DistinguishedValue {{ name: \"{}\".into(), value: {} }}",
            self.name, self.value
        )
    }
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

impl Quote for DeclarationElsewhere {
    fn quote(&self) -> String {
        format!("DeclarationElsewhere::from(\"{}\")", self.0)
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

impl Quote for Constraint {
    fn quote(&self) -> String {
        format!(
            "Constraint {{ min_value: {}, max_value: {}, extensible: {} }}",
            self.min_value
                .map_or("None".to_owned(), |m| "Some(".to_owned()
                    + &m.to_string()
                    + ")"),
            self.max_value
                .map_or("None".to_owned(), |m| "Some(".to_owned()
                    + &m.to_string()
                    + ")"),
            self.extensible
        )
    }
}

impl Constraint {
    pub fn int_type_token<'a>(&self) -> &'a str {
        match self.min_value.zip(self.max_value) {
            Some((min, max)) => match max - min {
                r if r <= u8::MAX.into() && min >= 0 => "u8",
                r if r <= u8::MAX.into() => "i8",
                r if r <= u16::MAX.into() && min >= 0 => "u16",
                r if r <= u16::MAX.into() => "i16",
                r if r <= u32::MAX.into() && min >= 0 => "u32",
                r if r <= u32::MAX.into() => "i32",
                r if r <= u64::MAX.into() && min >= 0 => "u64",
                r if r <= u64::MAX.into() => "i64",
                _ => "i128",
            },
            None => "i128",
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

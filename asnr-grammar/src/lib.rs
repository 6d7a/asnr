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

// Subtyping tokens
pub const SIZE: &'static str = "SIZE";
pub const DEFAULT: &'static str = "DEFAULT";
pub const OPTIONAL: &'static str = "OPTIONAL";
pub const WITH_COMPONENTS: &'static str = "WITH COMPONENTS";
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
    pub constraint: Option<SizeConstraint>,
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

impl From<SizeConstraint> for AsnInteger {
    fn from(value: SizeConstraint) -> Self {
        Self {
            constraint: Some(value),
            distinguished_values: None,
        }
    }
}

impl From<(Option<i128>, Option<i128>, bool)> for AsnInteger {
    fn from(value: (Option<i128>, Option<i128>, bool)) -> Self {
        Self {
            constraint: Some(SizeConstraint {
                min_value: value.0,
                max_value: value.1,
                extensible: value.2,
            }),
            distinguished_values: None,
        }
    }
}

impl From<(&str, Option<Vec<DistinguishedValue>>, Option<SizeConstraint>)> for AsnInteger {
    fn from(value: (&str, Option<Vec<DistinguishedValue>>, Option<SizeConstraint>)) -> Self {
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
    pub constraint: Option<SizeConstraint>,
    pub distinguished_values: Option<Vec<DistinguishedValue>>,
}

impl From<(Option<Vec<DistinguishedValue>>, Option<SizeConstraint>)> for AsnBitString {
    fn from(value: (Option<Vec<DistinguishedValue>>, Option<SizeConstraint>)) -> Self {
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

/// Representation of an ASN1 Character String type data element
/// with corresponding constraints. ASN1 Character String types
/// include IA5String, UTF8String, VideotexString, but also 
/// OCTET STRING, which is treated like a String and not a buffer.
#[derive(Debug, Clone, PartialEq)]
pub struct AsnCharacterString {
    pub constraint: Option<SizeConstraint>,
    pub r#type: CharacterStringType,
}

impl From<(&str, Option<SizeConstraint>)> for AsnCharacterString {
    fn from(value: (&str, Option<SizeConstraint>)) -> Self {
        AsnCharacterString {
            constraint: value.1,
            r#type: value.0.into(),
        }
    }
}

impl Quote for AsnCharacterString {
    fn quote(&self) -> String {
        format!(
            "AsnCharacterString {{ constraint: {}, r#type: CharacterStringType::{:?} }}",
            self.constraint
                .as_ref()
                .map_or("None".to_owned(), |c| "Some(".to_owned() + &c.quote() + ")"),
            self.r#type
        )
    }
}

/// Representation of an ASN1 SEQUENCE OF data element
/// with corresponding constraints and element type info
#[derive(Debug, Clone, PartialEq)]
pub struct AsnSequenceOf {
    pub constraint: Option<SizeConstraint>,
    pub r#type: Box<ASN1Type>,
}

impl Quote for AsnSequenceOf {
    fn quote(&self) -> String {
        format!(
            "AsnSequenceOf {{ constraint: {}, r#type: {} }}",
            self.constraint
                .as_ref()
                .map_or("None".to_owned(), |c| "Some(".to_owned() + &c.quote() + ")"),
            String::from("Box::new(") + &self.r#type.quote() + ")",
        )
    }
}

impl From<(Option<SizeConstraint>, ASN1Type)> for AsnSequenceOf {
    fn from(value: (Option<SizeConstraint>, ASN1Type)) -> Self {
        Self {
            constraint: value.0,
            r#type: Box::new(value.1),
        }
    }
}

/// Representation of an ASN1 SEQUENCE data element
/// with corresponding members and extension information
#[derive(Debug, Clone, PartialEq)]
pub struct AsnSequence {
    pub extensible: Option<usize>,
    pub members: Vec<SequenceMember>,
}

impl
    From<(
        Vec<SequenceMember>,
        Option<ExtensionMarker>,
        Option<Vec<SequenceMember>>,
    )> for AsnSequence
{
    fn from(
        mut value: (
            Vec<SequenceMember>,
            Option<ExtensionMarker>,
            Option<Vec<SequenceMember>>,
        ),
    ) -> Self {
        let index_of_first_extension = value.0.len();
        value.0.append(&mut value.2.unwrap_or(vec![]));
        AsnSequence {
            extensible: value.1.map(|_| index_of_first_extension),
            members: value.0,
        }
    }
}

impl Quote for AsnSequence {
    fn quote(&self) -> String {
        format!(
            "AsnSequence {{ extensible: {}, members: vec![{}] }}",
            self.extensible
                .as_ref()
                .map_or("None".to_owned(), |d| format!("Some({})", d)),
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

impl From<(&str, ASN1Type, Option<()>, Option<OptionalMarker>, Option<ASN1Value>)> for SequenceMember {
    fn from(value: (&str, ASN1Type, Option<()>, Option<OptionalMarker>, Option<ASN1Value>)) -> Self {
        SequenceMember {
            name: value.0.into(),
            r#type: value.1,
            is_optional: value.3.is_some() || value.4.is_some(),
            default_value: value.4,
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

/// Representation of an ASN1 CHOICE data element
/// with corresponding members and extension information
#[derive(Debug, Clone, PartialEq)]
pub struct AsnChoice {
    pub extensible: Option<usize>,
    pub options: Vec<ChoiceOption>,
}

impl
    From<(
        Vec<ChoiceOption>,
        Option<ExtensionMarker>,
        Option<Vec<ChoiceOption>>,
    )> for AsnChoice
{
    fn from(
        mut value: (
            Vec<ChoiceOption>,
            Option<ExtensionMarker>,
            Option<Vec<ChoiceOption>>,
        ),
    ) -> Self {
        let index_of_first_extension = value.0.len();
        value.0.append(&mut value.2.unwrap_or(vec![]));
        AsnChoice {
            extensible: value.1.map(|_| index_of_first_extension),
            options: value.0,
        }
    }
}

impl Quote for AsnChoice {
    fn quote(&self) -> String {
        format!(
            "AsnChoice {{ extensible: {}, options: vec![{}] }}",
            self.extensible
                .as_ref()
                .map_or("None".to_owned(), |d| format!("Some({})", d)),
            self.options
                .iter()
                .map(|m| m.quote())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

/// Representation of an single ASN1 CHOICE option
#[derive(Debug, Clone, PartialEq)]
pub struct ChoiceOption {
    pub name: String,
    pub r#type: ASN1Type
}

impl From<(&str, ASN1Type)> for ChoiceOption {
    fn from(value: (&str, ASN1Type)) -> Self {
      ChoiceOption {
            name: value.0.into(),
            r#type: value.1
        }
    }
}

impl Quote for ChoiceOption {
    fn quote(&self) -> String {
        format!(
          "ChoiceOption {{ name: \"{}\".into(), r#type: {} }}",
          self.name,
          self.r#type.quote(),
      )
    }
}


/// Representation of an ASN1 ENUMERATED data element
/// with corresponding enumerals and extension information
#[derive(Debug, Clone, PartialEq)]
pub struct AsnEnumerated {
    pub members: Vec<Enumeral>,
    pub extensible: Option<usize>,
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
                .as_ref()
                .map_or("None".to_owned(), |d| format!("Some({})", d)),
        )
    }
}

impl
    From<(
        Vec<Enumeral>,
        Option<ExtensionMarker>,
        Option<Vec<Enumeral>>,
    )> for AsnEnumerated
{
    fn from(
        mut value: (
            Vec<Enumeral>,
            Option<ExtensionMarker>,
            Option<Vec<Enumeral>>,
        ),
    ) -> Self {
        let index_of_first_extension = value.0.len();
        value.0.append(&mut value.2.unwrap_or(vec![]));
        AsnEnumerated {
            members: value.0,
            extensible: value.1.map(|_| index_of_first_extension),
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
pub struct SizeConstraint {
    pub min_value: Option<i128>,
    pub max_value: Option<i128>,
    pub extensible: bool,
}

impl Quote for SizeConstraint {
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

impl SizeConstraint {
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

impl<'a> From<i128> for SizeConstraint {
    fn from(value: i128) -> Self {
        Self {
            min_value: Some(value),
            max_value: Some(value),
            extensible: false,
        }
    }
}

impl<'a> From<(i128, RangeMarker, i128)> for SizeConstraint {
    fn from(value: (i128, RangeMarker, i128)) -> Self {
        Self {
            min_value: Some(value.0),
            max_value: Some(value.2),
            extensible: false,
        }
    }
}

impl<'a> From<(i128, ExtensionMarker)> for SizeConstraint {
    fn from(value: (i128, ExtensionMarker)) -> Self {
        Self {
            min_value: Some(value.0),
            max_value: Some(value.0),
            extensible: true,
        }
    }
}

impl<'a> From<(i128, RangeMarker, i128, ExtensionMarker)> for SizeConstraint {
    fn from(value: (i128, RangeMarker, i128, ExtensionMarker)) -> Self {
        Self {
            min_value: Some(value.0),
            max_value: Some(value.2),
            extensible: true,
        }
    }
}

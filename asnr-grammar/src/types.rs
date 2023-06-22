use core::{fmt::Debug};

use alloc::{borrow::ToOwned, boxed::Box, string::ToString, vec};

use crate::{subtyping::*, *};

/// Representation of an ASN1 INTEGER data element
/// with corresponding constraints and distinguished values
#[derive(Debug, Clone, PartialEq)]
pub struct Integer {
    pub constraints: Vec<ValueConstraint>,
    pub distinguished_values: Option<Vec<DistinguishedValue>>,
}

impl asnr_traits::Declare for Integer {
    fn declare(&self) -> String {
        format!(
            "Integer {{ constraints: vec![{}], distinguished_values: {} }}",
            self.constraints
                .iter()
                .map(|c| c.declare())
                .collect::<Vec<String>>()
                .join(", "),
            self.distinguished_values
                .as_ref()
                .map_or("None".to_owned(), |c| "Some(vec![".to_owned()
                    + &c.iter()
                        .map(|dv| dv.declare())
                        .collect::<Vec<String>>()
                        .join(",")
                    + "])"),
        )
    }
}

impl Default for Integer {
    fn default() -> Self {
        Self {
            constraints: vec![],
            distinguished_values: None,
        }
    }
}

impl From<ValueConstraint> for Integer {
    fn from(value: ValueConstraint) -> Self {
        Self {
            constraints: vec![value],
            distinguished_values: None,
        }
    }
}

impl From<(Option<i128>, Option<i128>, bool)> for Integer {
    fn from(value: (Option<i128>, Option<i128>, bool)) -> Self {
        Self {
            constraints: vec![ValueConstraint {
                min_value: value.0.map(|v| ASN1Value::Integer(v)),
                max_value: value.1.map(|v| ASN1Value::Integer(v)),
                extensible: value.2,
            }],
            distinguished_values: None,
        }
    }
}

impl
    From<(
        &str,
        Option<Vec<DistinguishedValue>>,
        Option<ValueConstraint>,
    )> for Integer
{
    fn from(
        value: (
            &str,
            Option<Vec<DistinguishedValue>>,
            Option<ValueConstraint>,
        ),
    ) -> Self {
        Self {
            constraints: value.2.map_or(vec![], |r| vec![r]),
            distinguished_values: value.1,
        }
    }
}

/// Representation of an ASN1 BIT STRING data element
/// with corresponding constraints and distinguished values
/// defining the individual bits
#[derive(Debug, Clone, PartialEq)]
pub struct BitString {
    pub constraints: Vec<ValueConstraint>,
    pub distinguished_values: Option<Vec<DistinguishedValue>>,
}

impl From<(Option<Vec<DistinguishedValue>>, Option<ValueConstraint>)> for BitString {
    fn from(value: (Option<Vec<DistinguishedValue>>, Option<ValueConstraint>)) -> Self {
        BitString {
            constraints: value.1.map_or(vec![], |r| vec![r]),
            distinguished_values: value.0,
        }
    }
}

impl asnr_traits::Declare for BitString {
    fn declare(&self) -> String {
        format!(
            "BitString {{ constraints: vec![{}], distinguished_values: {} }}",
            self.constraints
                .iter()
                .map(|c| c.declare())
                .collect::<Vec<String>>()
                .join(", "),
            self.distinguished_values
                .as_ref()
                .map_or("None".to_owned(), |c| "Some(vec![".to_owned()
                    + &c.iter()
                        .map(|dv| dv.declare())
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
pub struct CharacterString {
    pub constraints: Vec<Constraint>,
    pub r#type: CharacterStringType,
}

impl From<(&str, Option<ValueConstraint>)> for CharacterString {
    fn from(value: (&str, Option<ValueConstraint>)) -> Self {
        CharacterString {
            constraints: value
                .1
                .map_or(vec![], |r| vec![Constraint::ValueConstraint(r)]),
            r#type: value.0.into(),
        }
    }
}

impl asnr_traits::Declare for CharacterString {
    fn declare(&self) -> String {
        format!(
            "CharacterString {{ constraints: vec![{}], r#type: CharacterStringType::{:?} }}",
            self.constraints
                .iter()
                .map(|c| c.declare())
                .collect::<Vec<String>>()
                .join(", "),
            self.r#type
        )
    }
}

/// Representation of an ASN1 SEQUENCE OF data element
/// with corresponding constraints and element type info
#[derive(Debug, Clone, PartialEq)]
pub struct SequenceOf {
    pub constraints: Vec<Constraint>,
    pub r#type: Box<ASN1Type>,
}

impl asnr_traits::Declare for SequenceOf {
    fn declare(&self) -> String {
        format!(
            "SequenceOf {{ constraints: vec![{}], r#type: {} }}",
            self.constraints
                .iter()
                .map(|c| c.declare())
                .collect::<Vec<String>>()
                .join(", "),
            String::from("Box::new(") + &self.r#type.declare() + ")",
        )
    }
}

impl From<(Option<Vec<Constraint>>, ASN1Type)> for SequenceOf {
    fn from(value: (Option<Vec<Constraint>>, ASN1Type)) -> Self {
        Self {
            constraints: value.0.unwrap_or(vec![]),
            r#type: Box::new(value.1),
        }
    }
}

/// Representation of an ASN1 SEQUENCE data element
/// with corresponding members and extension information
#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    pub extensible: Option<usize>,
    pub constraints: Vec<Constraint>,
    pub members: Vec<SequenceMember>,
}

impl
    From<(
        (
            Vec<SequenceMember>,
            Option<ExtensionMarker>,
            Option<Vec<SequenceMember>>,
        ),
        Option<Vec<Constraint>>,
    )> for Sequence
{
    fn from(
        mut value: (
            (
                Vec<SequenceMember>,
                Option<ExtensionMarker>,
                Option<Vec<SequenceMember>>,
            ),
            Option<Vec<Constraint>>,
        ),
    ) -> Self {
        let index_of_first_extension = value.0 .0.len();
        value.0 .0.append(&mut value.0 .2.unwrap_or(vec![]));
        Sequence {
            constraints: value.1.unwrap_or(vec![]),
            extensible: value.0 .1.map(|_| index_of_first_extension),
            members: value.0 .0,
        }
    }
}

impl asnr_traits::Declare for Sequence {
    fn declare(&self) -> String {
        format!(
            "Sequence {{ constraints: vec![{}], extensible: {}, members: vec![{}] }}",
            self.constraints
                .iter()
                .map(|c| c.declare())
                .collect::<Vec<String>>()
                .join(", "),
            self.extensible
                .as_ref()
                .map_or("None".to_owned(), |d| format!("Some({})", d)),
            self.members
                .iter()
                .map(|m| m.declare())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

/// Representation of an single ASN1 SEQUENCE member
#[derive(Debug, Clone, PartialEq)]
pub struct SequenceMember {
    pub name: String,
    pub tag: Option<AsnTag>,
    pub r#type: ASN1Type,
    pub default_value: Option<ASN1Value>,
    pub is_optional: bool,
    pub constraints: Vec<Constraint>,
}

impl
    From<(
        &str,
        Option<AsnTag>,
        ASN1Type,
        Option<Vec<Constraint>>,
        Option<OptionalMarker>,
        Option<ASN1Value>,
    )> for SequenceMember
{
    fn from(
        value: (
            &str,
            Option<AsnTag>,
            ASN1Type,
            Option<Vec<Constraint>>,
            Option<OptionalMarker>,
            Option<ASN1Value>,
        ),
    ) -> Self {
        SequenceMember {
            name: value.0.into(),
            tag: value.1,
            r#type: value.2,
            is_optional: value.4.is_some() || value.5.is_some(),
            default_value: value.5,
            constraints: value.3.unwrap_or(vec![]),
        }
    }
}

impl asnr_traits::Declare for SequenceMember {
    fn declare(&self) -> String {
        format!(
          "SequenceMember {{ name: \"{}\".into(), tag: {}, is_optional: {}, r#type: {}, default_value: {}, constraints: vec![{}] }}",
          self.name,
          self.tag.as_ref().map_or(String::from("None"), |t| {
            String::from("Some(") + &t.declare() + ")"
          }),
          self.is_optional,
          self.r#type.declare(),
          self.default_value.as_ref().map_or("None".to_string(), |d| "Some(".to_owned()
          + &d.declare()
          + ")"),
          self.constraints
          .iter()
          .map(|c| c.declare())
          .collect::<Vec<String>>()
          .join(", "),
      )
    }
}

/// Representation of an ASN1 CHOICE data element
/// with corresponding members and extension information
#[derive(Debug, Clone, PartialEq)]
pub struct Choice {
    pub extensible: Option<usize>,
    pub options: Vec<ChoiceOption>,
    pub constraints: Vec<Constraint>,
}

impl
    From<(
        Vec<ChoiceOption>,
        Option<ExtensionMarker>,
        Option<Vec<ChoiceOption>>,
    )> for Choice
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
        Choice {
            extensible: value.1.map(|_| index_of_first_extension),
            options: value.0,
            constraints: vec![],
        }
    }
}

impl asnr_traits::Declare for Choice {
    fn declare(&self) -> String {
        format!(
            "Choice {{ extensible: {}, options: vec![{}], constraints: vec![{}] }}",
            self.extensible
                .as_ref()
                .map_or("None".to_owned(), |d| format!("Some({})", d)),
            self.options
                .iter()
                .map(|m| m.declare())
                .collect::<Vec<String>>()
                .join(","),
            self.constraints
                .iter()
                .map(|c| c.declare())
                .collect::<Vec<String>>()
                .join(", "),
        )
    }
}

/// Representation of an single ASN1 CHOICE option
#[derive(Debug, Clone, PartialEq)]
pub struct ChoiceOption {
    pub name: String,
    pub tag: Option<AsnTag>,
    pub r#type: ASN1Type,
    pub constraints: Vec<Constraint>,
}

impl From<(&str, Option<AsnTag>, ASN1Type, Option<Vec<Constraint>>)> for ChoiceOption {
    fn from(value: (&str, Option<AsnTag>, ASN1Type, Option<Vec<Constraint>>)) -> Self {
        ChoiceOption {
            name: value.0.into(),
            tag: value.1,
            r#type: value.2,
            constraints: value.3.unwrap_or(vec![]),
        }
    }
}

impl asnr_traits::Declare for ChoiceOption {
    fn declare(&self) -> String {
        format!(
            "ChoiceOption {{ name: \"{}\".into(), tag: {}, r#type: {}, constraints: vec![{}] }}",
            self.name,
            self.tag.as_ref().map_or(String::from("None"), |t| {
                String::from("Some(") + &t.declare() + ")"
            }),
            self.r#type.declare(),
            self.constraints
                .iter()
                .map(|c| c.declare())
                .collect::<Vec<String>>()
                .join(", "),
        )
    }
}

/// Representation of an ASN1 ENUMERATED data element
/// with corresponding enumerals and extension information
#[derive(Debug, Clone, PartialEq)]
pub struct Enumerated {
    pub members: Vec<Enumeral>,
    pub extensible: Option<usize>,
    pub constraints: Vec<Constraint>,
}

impl asnr_traits::Declare for Enumerated {
    fn declare(&self) -> String {
        format!(
            "Enumerated {{ members: vec![{}], extensible: {}, constraints: vec![{}] }}",
            self.members
                .iter()
                .map(|m| m.declare())
                .collect::<Vec<String>>()
                .join(","),
            self.extensible
                .as_ref()
                .map_or("None".to_owned(), |d| format!("Some({})", d)),
            self.constraints
                .iter()
                .map(|c| c.declare())
                .collect::<Vec<String>>()
                .join(", "),
        )
    }
}

impl
    From<(
        Vec<Enumeral>,
        Option<ExtensionMarker>,
        Option<Vec<Enumeral>>,
    )> for Enumerated
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
        Enumerated {
            members: value.0,
            extensible: value.1.map(|_| index_of_first_extension),
            constraints: vec![],
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

impl asnr_traits::Declare for Enumeral {
    fn declare(&self) -> String {
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

impl asnr_traits::Declare for DistinguishedValue {
    fn declare(&self) -> String {
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

#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxExpression {
    Required(SyntaxToken),
    Optional(Vec<SyntaxExpression>),
}

impl asnr_traits::Declare for SyntaxExpression {
    fn declare(&self) -> String {
        match self {
            SyntaxExpression::Required(r) => format!("SyntaxExpression::Required({})", r.declare()),
            SyntaxExpression::Optional(o) => format!(
                "SyntaxExpression::Optional(vec![{}])",
                o.iter()
                    .map(|s| s.declare())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxApplication {
    ObjectSetDeclaration(ObjectSet),
    ValueReference(ASN1Value),
    TypeReference(ASN1Type),
    Comma,
    Literal(String),
}

impl asnr_traits::Declare for SyntaxApplication {
    fn declare(&self) -> String {
        match self {
            SyntaxApplication::ObjectSetDeclaration(o) => {
                format!("SyntaxApplication::ObjectSetDeclaration({})", o.declare())
            }
            SyntaxApplication::ValueReference(v) => {
                format!("SyntaxApplication::ValueReference({})", v.declare())
            }
            SyntaxApplication::TypeReference(t) => {
                format!("SyntaxApplication::TypeReference({})", t.declare())
            }
            SyntaxApplication::Comma => "SyntaxApplication::Comma".into(),
            SyntaxApplication::Literal(s) => format!("SyntaxApplication::Literal(\"{s}\".into())"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxToken {
    Literal(String),
    Comma,
    Field(ObjectFieldIdentifier),
}

impl asnr_traits::Declare for SyntaxToken {
    fn declare(&self) -> String {
        match self {
            SyntaxToken::Literal(l) => format!("SyntaxToken::Literal(\"{l}\".into())"),
            SyntaxToken::Comma => "SyntaxToken::Comma".to_owned(),
            SyntaxToken::Field(o) => format!("SyntaxToken::Field({})", o.declare()),
        }
    }
}

impl From<ObjectFieldIdentifier> for SyntaxToken {
    fn from(value: ObjectFieldIdentifier) -> Self {
        Self::Field(value)
    }
}

impl From<&str> for SyntaxToken {
    fn from(value: &str) -> Self {
        if value == "," {
            Self::Comma
        } else {
            Self::Literal(value.into())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InformationObjectSyntax {
    pub expressions: Vec<SyntaxExpression>,
}

impl asnr_traits::Declare for InformationObjectSyntax {
    fn declare(&self) -> String {
        format!(
            "InformationObjectSyntax {{ expressions: vec![{}] }}",
            self.expressions
                .iter()
                .map(|s| s.declare())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InformationObjectClass {
    pub fields: Vec<InformationObjectClassField>,
    pub syntax: Option<InformationObjectSyntax>,
}

impl asnr_traits::Declare for InformationObjectClass {
    fn declare(&self) -> String {
        format!(
            "InformationObjectClass {{ fields: vec![{}], syntax: {} }}",
            self.fields
                .iter()
                .map(|f| f.declare())
                .collect::<Vec<String>>()
                .join(", "),
            self.syntax
                .as_ref()
                .map_or("None".to_owned(), |d| String::from("Some(")
                    + &d.declare()
                    + ")")
        )
    }
}

impl
    From<(
        Vec<InformationObjectClassField>,
        Option<Vec<SyntaxExpression>>,
    )> for InformationObjectClass
{
    fn from(
        value: (
            Vec<InformationObjectClassField>,
            Option<Vec<SyntaxExpression>>,
        ),
    ) -> Self {
        Self {
            fields: value.0,
            syntax: value
                .1
                .map(|expr| InformationObjectSyntax { expressions: expr }),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InformationObjectClassField {
    pub identifier: ObjectFieldIdentifier,
    pub r#type: Option<ASN1Type>,
    pub is_optional: bool,
    pub default: Option<ASN1Value>,
    pub is_unique: bool,
}

impl asnr_traits::Declare for InformationObjectClassField {
    fn declare(&self) -> String {
        format!("InformationObjectClassField {{ identifier: {}, r#type: {}, is_optional: {}, default: {}, is_unique: {} }}",
        self.identifier.declare(),
        self.r#type.as_ref().map_or("None".to_owned(), |t| String::from("Some(") + &t.declare() + ")" ),
        self.is_optional,
        self.default.as_ref().map_or("None".to_owned(), |d| String::from("Some(") + &d.declare() + ")" ),
        self.is_unique
      )
    }
}

impl
    From<(
        ObjectFieldIdentifier,
        Option<ASN1Type>,
        Option<&str>,
        Option<OptionalMarker>,
        Option<ASN1Value>,
    )> for InformationObjectClassField
{
    fn from(
        value: (
            ObjectFieldIdentifier,
            Option<ASN1Type>,
            Option<&str>,
            Option<OptionalMarker>,
            Option<ASN1Value>,
        ),
    ) -> Self {
        Self {
            identifier: value.0,
            r#type: value.1,
            is_unique: value.2.is_some(),
            is_optional: value.3.is_some() || value.4.is_some(),
            default: value.4,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectFieldIdentifier {
    SingleValue(String),
    MultipleValue(String),
}

impl asnr_traits::Declare for ObjectFieldIdentifier {
    fn declare(&self) -> String {
        match self {
            ObjectFieldIdentifier::SingleValue(s) => {
                format!("ObjectFieldIdentifier::SingleValue(\"{s}\".into())")
            }
            ObjectFieldIdentifier::MultipleValue(m) => {
                format!("ObjectFieldIdentifier::MultipleValue(\"{m}\".into())")
            }
        }
    }
}

impl ObjectFieldIdentifier {
    pub fn identifier(&self) -> String {
        match self {
            ObjectFieldIdentifier::SingleValue(s) => s.clone(),
            ObjectFieldIdentifier::MultipleValue(s) => s.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InformationObject {
    pub supertype: String,
    pub fields: InformationObjectFields,
}

impl asnr_traits::Declare for InformationObject {
    fn declare(&self) -> String {
        format!(
            "InformationObject {{ supertype: \"{}\".into(), fields: {} }}",
            self.supertype,
            self.fields.declare()
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InformationObjectFields {
    DefaultSyntax(Vec<InformationObjectField>),
    CustomSyntax(Vec<SyntaxApplication>),
}

impl asnr_traits::Declare for InformationObjectFields {
    fn declare(&self) -> String {
        match self {
            InformationObjectFields::DefaultSyntax(d) => format!(
                "InformationObjectFields::DefaultSyntax(vec![{}])",
                d.iter()
                    .map(|s| s.declare())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            InformationObjectFields::CustomSyntax(c) => format!(
                "InformationObjectFields::CustomSyntax(vec![{}])",
                c.iter()
                    .map(|s| s.declare())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectSetValue {
    Reference(String),
    Inline(InformationObjectFields),
}

impl From<&str> for ObjectSetValue {
    fn from(value: &str) -> Self {
        Self::Reference(value.into())
    }
}

impl From<InformationObjectFields> for ObjectSetValue {
    fn from(value: InformationObjectFields) -> Self {
        Self::Inline(value)
    }
}

impl asnr_traits::Declare for ObjectSetValue {
    fn declare(&self) -> String {
        match self {
            ObjectSetValue::Reference(r) => format!("ObjectSetValue::Reference(\"{r}\".into())"),
            ObjectSetValue::Inline(i) => format!("ObjectSetValue::Inline({})", i.declare()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectSet {
    pub values: Vec<ObjectSetValue>,
    pub extensible: Option<usize>,
}

impl asnr_traits::Declare for ObjectSet {
    fn declare(&self) -> String {
        format!(
            "ValueSet {{ values: vec![{}], extensible: {} }}",
            self.values
                .iter()
                .map(|v| v.declare())
                .collect::<Vec<String>>()
                .join(", "),
            self.extensible
                .map_or("None".to_owned(), |e| format!("Some({e})"))
        )
    }
}

impl
    From<(
        Vec<ObjectSetValue>,
        Option<ExtensionMarker>,
        Option<Vec<ObjectSetValue>>,
    )> for ObjectSet
{
    fn from(
        mut value: (
            Vec<ObjectSetValue>,
            Option<ExtensionMarker>,
            Option<Vec<ObjectSetValue>>,
        ),
    ) -> Self {
        let index_of_first_extension = value.0.len();
        value.0.append(&mut value.2.unwrap_or(vec![]));
        ObjectSet {
            values: value.0,
            extensible: value.1.map(|_| index_of_first_extension),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InformationObjectField {
    TypeField(TypeField),
    FixedValueField(FixedValueField),
    ObjectSetField(ObjectSetField),
}

impl asnr_traits::Declare for InformationObjectField {
    fn declare(&self) -> String {
        match self {
            InformationObjectField::TypeField(t) => {
                format!("InformationObjectField::TypeField({})", t.declare())
            }
            InformationObjectField::FixedValueField(f) => {
                format!("InformationObjectField::FixedValueField({})", f.declare())
            }
            InformationObjectField::ObjectSetField(o) => {
                format!("InformationObjectField::ObjectSetField({})", o.declare())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FixedValueField {
    pub identifier: String,
    pub value: ASN1Value,
}

impl asnr_traits::Declare for FixedValueField {
    fn declare(&self) -> String {
        format!(
            "FixedValueField {{ identifier: \"{}\".into(), value: {} }}",
            self.identifier,
            self.value.declare()
        )
    }
}

impl From<(ObjectFieldIdentifier, ASN1Value)> for InformationObjectField {
    fn from(value: (ObjectFieldIdentifier, ASN1Value)) -> Self {
        Self::FixedValueField(FixedValueField {
            identifier: value.0.identifier(),
            value: value.1,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeField {
    pub identifier: String,
    pub r#type: ASN1Type,
}

impl asnr_traits::Declare for TypeField {
    fn declare(&self) -> String {
        format!(
            "TypeField {{ identifier: \"{}\".into(), r#type: {} }}",
            self.identifier,
            self.r#type.declare()
        )
    }
}

impl From<(ObjectFieldIdentifier, ASN1Type)> for InformationObjectField {
    fn from(value: (ObjectFieldIdentifier, ASN1Type)) -> Self {
        Self::TypeField(TypeField {
            identifier: value.0.identifier(),
            r#type: value.1,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectSetField {
    pub identifier: String,
    pub value: ObjectSet,
}

impl asnr_traits::Declare for ObjectSetField {
    fn declare(&self) -> String {
        format!(
            "ObjectSetField {{ identifier: \"{}\".into(), value: {} }}",
            self.identifier,
            self.value.declare()
        )
    }
}

impl From<(ObjectFieldIdentifier, ObjectSet)> for InformationObjectField {
    fn from(value: (ObjectFieldIdentifier, ObjectSet)) -> Self {
        Self::ObjectSetField(ObjectSetField {
            identifier: value.0.identifier(),
            value: value.1,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InformationObjectFieldReference {
    pub class: String,
    pub field_path: Vec<ObjectFieldIdentifier>,
    pub constraints: Vec<Constraint>,
}

impl asnr_traits::Declare for InformationObjectFieldReference {
    fn declare(&self) -> String {
        format!("InformationObjectFieldReference {{ class: \"{}\".into(), field_path: vec![{}], constraints: vec![{}] }}",
      self.class,
    self.field_path.iter().map(|f| f.declare()).collect::<Vec<String>>().join(", "),
    self.constraints.iter().map(|c| c.declare()).collect::<Vec<String>>().join(", "))
    }
}

impl From<(&str, Vec<ObjectFieldIdentifier>, Vec<Constraint>)> for InformationObjectFieldReference {
    fn from(value: (&str, Vec<ObjectFieldIdentifier>, Vec<Constraint>)) -> Self {
        Self {
            class: value.0.into(),
            field_path: value.1,
            constraints: value.2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Declare)]
pub struct Parameterization {
    pub parameters: Vec<ParameterizationArgument>,
}

impl From<Vec<(&str, &str)>> for Parameterization {
    fn from(value: Vec<(&str, &str)>) -> Self {
        Self {
            parameters: value
                .into_iter()
                .map(|(t, i)| ParameterizationArgument {
                    r#type: t.into(),
                    name: i.into(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Declare)]
pub struct ParameterizationArgument {
    pub r#type: String,
    pub name: String,
}

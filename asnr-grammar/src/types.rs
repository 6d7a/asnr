use core::fmt::Debug;

use alloc::{borrow::ToOwned, boxed::Box, string::ToString, vec};

use crate::{subtyping::*, *};

/// Representation of an ASN1 INTEGER data element
/// with corresponding constraints and distinguished values
#[derive(Debug, Clone, PartialEq)]
pub struct Integer {
    pub constraints: Vec<ValueConstraint>,
    pub distinguished_values: Option<Vec<DistinguishedValue>>,
}

impl Quote for Integer {
    fn quote(&self) -> String {
        format!(
            "Integer {{ constraints: vec![{}], distinguished_values: {} }}",
            self.constraints
                .iter()
                .map(|c| c.quote())
                .collect::<Vec<String>>()
                .join(", "),
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

impl Quote for BitString {
    fn quote(&self) -> String {
        format!(
            "BitString {{ constraints: vec![{}], distinguished_values: {} }}",
            self.constraints
                .iter()
                .map(|c| c.quote())
                .collect::<Vec<String>>()
                .join(", "),
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

impl Quote for CharacterString {
    fn quote(&self) -> String {
        format!(
            "CharacterString {{ constraints: vec![{}], r#type: CharacterStringType::{:?} }}",
            self.constraints
                .iter()
                .map(|c| c.quote())
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

impl Quote for SequenceOf {
    fn quote(&self) -> String {
        format!(
            "SequenceOf {{ constraints: vec![{}], r#type: {} }}",
            self.constraints
                .iter()
                .map(|c| c.quote())
                .collect::<Vec<String>>()
                .join(", "),
            String::from("Box::new(") + &self.r#type.quote() + ")",
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

impl Quote for Sequence {
    fn quote(&self) -> String {
        format!(
            "Sequence {{ constraints: vec![{}], extensible: {}, members: vec![{}] }}",
            self.constraints
                .iter()
                .map(|c| c.quote())
                .collect::<Vec<String>>()
                .join(", "),
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

impl Quote for SequenceMember {
    fn quote(&self) -> String {
        format!(
          "SequenceMember {{ name: \"{}\".into(), tag: {}, is_optional: {}, r#type: {}, default_value: {}, constraints: vec![{}] }}",
          self.name,
          self.tag.as_ref().map_or(String::from("None"), |t| {
            String::from("Some(") + &t.quote() + ")"
          }),
          self.is_optional,
          self.r#type.quote(),
          self.default_value.as_ref().map_or("None".to_string(), |d| "Some(".to_owned()
          + &d.quote()
          + ")"),
          self.constraints
          .iter()
          .map(|c| c.quote())
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

impl Quote for Choice {
    fn quote(&self) -> String {
        format!(
            "Choice {{ extensible: {}, options: vec![{}], constraints: vec![{}] }}",
            self.extensible
                .as_ref()
                .map_or("None".to_owned(), |d| format!("Some({})", d)),
            self.options
                .iter()
                .map(|m| m.quote())
                .collect::<Vec<String>>()
                .join(","),
            self.constraints
                .iter()
                .map(|c| c.quote())
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

impl Quote for ChoiceOption {
    fn quote(&self) -> String {
        format!(
            "ChoiceOption {{ name: \"{}\".into(), tag: {}, r#type: {}, constraints: vec![{}] }}",
            self.name,
            self.tag.as_ref().map_or(String::from("None"), |t| {
                String::from("Some(") + &t.quote() + ")"
            }),
            self.r#type.quote(),
            self.constraints
                .iter()
                .map(|c| c.quote())
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

impl Quote for Enumerated {
    fn quote(&self) -> String {
        format!(
            "Enumerated {{ members: vec![{}], extensible: {}, constraints: vec![{}] }}",
            self.members
                .iter()
                .map(|m| m.quote())
                .collect::<Vec<String>>()
                .join(","),
            self.extensible
                .as_ref()
                .map_or("None".to_owned(), |d| format!("Some({})", d)),
            self.constraints
                .iter()
                .map(|c| c.quote())
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

#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxExpression {
    Required(SyntaxToken),
    Optional(Vec<SyntaxExpression>),
}

impl Quote for SyntaxExpression {
    fn quote(&self) -> String {
        match self {
            SyntaxExpression::Required(r) => format!("SyntaxExpression::Required({})", r.quote()),
            SyntaxExpression::Optional(o) => format!(
                "SyntaxExpression::Optional(vec![{}])",
                o.iter()
                    .map(|s| s.quote())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxApplication {
    ObjectSetDeclaration(ValueSet),
    ValueReference(ASN1Value),
    TypeReference(ASN1Type),
    Comma,
    Literal(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxToken {
    Literal(String),
    Comma,
    Field(ObjectFieldIdentifier),
}

impl Quote for SyntaxToken {
    fn quote(&self) -> String {
        match self {
            SyntaxToken::Literal(l) => format!("SyntaxToken::Literal(\"{l}\".into())"),
            SyntaxToken::Comma => "SyntaxToken::Comma".to_owned(),
            SyntaxToken::Field(o) => format!("SyntaxToken::Field({})", o.quote()),
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

impl Quote for InformationObjectSyntax {
    fn quote(&self) -> String {
        format!(
            "InformationObjectSyntax {{ expressions: vec![{}] }}",
            self.expressions
                .iter()
                .map(|s| s.quote())
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

impl Quote for InformationObjectClass {
    fn quote(&self) -> String {
        format!(
            "InformationObjectClass {{ fields: vec![{}], syntax: {} }}",
            self.fields
                .iter()
                .map(|f| f.quote())
                .collect::<Vec<String>>()
                .join(", "),
            self.syntax.as_ref()
                .map_or("None".to_owned(), |d| String::from("Some(")
                    + &d.quote()
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

impl Quote for InformationObjectClassField {
    fn quote(&self) -> String {
        format!("InformationObjectClassField {{ identifier: {}, r#type: {}, is_optional: {}, default: {}, is_unique: {} }}",
        self.identifier.quote(),
        self.r#type.as_ref().map_or("None".to_owned(), |t| String::from("Some(") + &t.quote() + ")" ),
        self.is_optional,
        self.default.as_ref().map_or("None".to_owned(), |d| String::from("Some(") + &d.quote() + ")" ),
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

impl Quote for ObjectFieldIdentifier {
    fn quote(&self) -> String {
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

#[derive(Debug, Clone, PartialEq)]
pub enum InformationObjectFields {
    DefaultSyntax(Vec<InformationObjectField>),
    CustomSyntax(Vec<SyntaxApplication>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueSet {
    pub values: Vec<ASN1Value>,
    pub extensible: Option<usize>,
}

impl
    From<(
        Vec<ASN1Value>,
        Option<ExtensionMarker>,
        Option<Vec<ASN1Value>>,
    )> for ValueSet
{
    fn from(
        mut value: (
            Vec<ASN1Value>,
            Option<ExtensionMarker>,
            Option<Vec<ASN1Value>>,
        ),
    ) -> Self {
        let index_of_first_extension = value.0.len();
        value.0.append(&mut value.2.unwrap_or(vec![]));
        ValueSet {
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

#[derive(Debug, Clone, PartialEq)]
pub struct FixedValueField {
    pub identifier: String,
    pub value: ASN1Value,
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
    pub value: ValueSet,
}

impl From<(ObjectFieldIdentifier, ValueSet)> for InformationObjectField {
    fn from(value: (ObjectFieldIdentifier, ValueSet)) -> Self {
        Self::ObjectSetField(ObjectSetField {
            identifier: value.0.identifier(),
            value: value.1,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InformationObjectFieldReference {
    pub class: String,
    pub field_name: ObjectFieldIdentifier,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct ParameterizationArgument {
    pub r#type: String,
    pub name: String,
}

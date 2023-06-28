use asnr_traits::Declare;

use crate::{subtyping::*, *, utils::walk_object_field_ref_path};
use alloc::{borrow::ToOwned, boxed::Box, format, string::String, vec, vec::Vec};

#[derive(Debug, Clone, PartialEq)]
pub struct ToplevelInformationDeclaration {
    pub comments: String,
    pub name: String,
    pub class: Option<ClassLink>,
    pub value: ASN1Information,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassLink {
    ByName(String),
    ByReference(InformationObjectClass),
}

impl From<(Vec<&str>, &str, &str, InformationObjectFields)> for ToplevelInformationDeclaration {
    fn from(value: (Vec<&str>, &str, &str, InformationObjectFields)) -> Self {
        Self {
            comments: value.0.join("\n"),
            name: value.1.into(),
            class: Some(ClassLink::ByName(value.2.into())),
            value: ASN1Information::Object(InformationObject {
                supertype: value.2.into(),
                fields: value.3,
            }),
        }
    }
}

impl From<(Vec<&str>, &str, &str, ObjectSet)> for ToplevelInformationDeclaration {
    fn from(value: (Vec<&str>, &str, &str, ObjectSet)) -> Self {
        Self {
            comments: value.0.join("\n"),
            name: value.1.into(),
            class: Some(ClassLink::ByName(value.2.into())),
            value: ASN1Information::ObjectSet(value.3),
        }
    }
}

impl From<(Vec<&str>, &str, InformationObjectClass)> for ToplevelInformationDeclaration {
    fn from(value: (Vec<&str>, &str, InformationObjectClass)) -> Self {
        Self {
            comments: value.0.join("\n"),
            name: value.1.into(),
            class: None,
            value: ASN1Information::ObjectClass(value.2),
        }
    }
}

/// The possible types of an ASN1 information object.
#[derive(Debug, Clone, PartialEq)]
pub enum ASN1Information {
    ObjectClass(InformationObjectClass),
    ObjectSet(ObjectSet),
    Object(InformationObject),
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

impl SyntaxToken {
    pub fn name_or_empty(&self) -> &str {
        match self {
            SyntaxToken::Field(ObjectFieldIdentifier::SingleValue(v))
            | SyntaxToken::Field(ObjectFieldIdentifier::MultipleValue(v)) => v.as_str(),
            _ => "",
        }
    }
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

impl InformationObjectClass {
    pub fn get_field<'a>(
        &'a self,
        path: &'a Vec<ObjectFieldIdentifier>,
    ) -> Option<&InformationObjectClassField> {
        walk_object_field_ref_path(&self.fields, path, 0)
    }
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
            "ObjectSet {{ values: vec![{}], extensible: {} }}",
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

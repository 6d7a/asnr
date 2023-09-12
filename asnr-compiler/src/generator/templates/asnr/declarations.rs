use asnr_grammar::{*, constraints::*, types::*, information_object::*};

/// The `Declare` trait serves to convert a structure
/// into a stringified rust representation of its initialization.
pub trait Declare {
    /// Returns a stringified representation of the implementing struct's initialization
    fn declare(&self) -> String;
}

impl Declare for ASN1Type {
    fn declare(&self) -> String {
        match self {
            ASN1Type::Null => "ASN1Type::Null".into(),
            ASN1Type::Boolean => "ASN1Type::Boolean".into(),
            ASN1Type::Integer(i) => format!("ASN1Type::Integer({})", i.declare()),
            ASN1Type::Real(r) => format!("ASN1Type::Real({})", r.declare()),
            ASN1Type::BitString(b) => format!("ASN1Type::BitString({})", b.declare()),
            ASN1Type::OctetString(o) => format!("ASN1Type::OctetString({})", o.declare()),
            ASN1Type::CharacterString(o) => format!("ASN1Type::CharacterString({})", o.declare()),
            ASN1Type::Enumerated(e) => format!("ASN1Type::Enumerated({})", e.declare()),
            ASN1Type::SequenceOf(s) => format!("ASN1Type::SequenceOf({})", s.declare()),
            ASN1Type::Sequence(s) => format!("ASN1Type::Sequence({})", s.declare()),
            ASN1Type::Set(s) => format!("ASN1Type::Set({})", s.declare()),
            ASN1Type::Choice(c) => format!("ASN1Type::Choice({})", c.declare()),
            ASN1Type::ElsewhereDeclaredType(els) => {
                format!("ASN1Type::ElsewhereDeclaredType({})", els.declare())
            }
            ASN1Type::InformationObjectFieldReference(iofr) => format!(
                "ASN1Type::InformationObjectFieldReference({})",
                iofr.declare()
            ),
        }
    }
}


impl Declare for ASN1Value {
    fn declare(&self) -> String {
        match self {
            ASN1Value::All => String::from("ASN1Value::All"),
            ASN1Value::Null => String::from("ASN1Value::Null"),
            ASN1Value::Real(r) => format!("ASN1Value::Real({})", r),
            ASN1Value::Boolean(b) => format!("ASN1Value::Boolean({})", b),
            ASN1Value::Integer(i) => format!("ASN1Value::Integer({})", i),
            ASN1Value::String(s) => format!("ASN1Value::String(\"{}\".into())", s),
            ASN1Value::Choice(i, v) => format!("ASN1Value::Choice({i}, Box::new({}))", v.declare()),
            ASN1Value::Sequence(fields) => format!(
                "ASN1Value::Sequence(vec![{}])",
                fields
                    .iter()
                    .map(|(id, val)| format!("({id}, Box::new({}))", val.declare()))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
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
            ASN1Value::EnumeratedValue { enumerated, enumerable } => {
                format!("ASN1Value::EnumeratedValue {{ enumerated: \"{enumerated}\".into(), enumerable: \"{enumerable}\".into() }}")
            }
            ASN1Value::ElsewhereDeclaredValue(s) => {
                format!("ASN1Value::ElsewhereDeclaredValue(\"{}\".into())", s)
            }
            ASN1Value::ObjectIdentifier(oid) => {
                format!("ASN1Value::ObjectIdentifier({})", oid.declare())
            }
        }
    }
}


impl Declare for DeclarationElsewhere {
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


impl Declare for AsnTag {
    fn declare(&self) -> String {
        format!(
            "AsnTag {{ tag_class: TagClass::{:?}, id: {}, environment: Explicitness::{:?} }}",
            self.tag_class, self.id, self.environment
        )
    }
}

impl Declare for ObjectIdentifierArc {
    fn declare(&self) -> String {
        format!(
            "ObjectIdentifierArc {{ name: {:?}.into(), number: {:?} }}",
            self.name, self.number
        )
    }
}

impl Declare for ObjectIdentifier {
    fn declare(&self) -> String {
        format!(
            "ObjectIdentifier(vec![{}])",
            self.0
                .iter()
                .map(Declare::declare)
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

impl Declare for Integer {
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

impl Declare for Real {
    fn declare(&self) -> String {
        format!(
            "Real {{ constraints: vec![{}] }}",
            self.constraints
                .iter()
                .map(|c| c.declare())
                .collect::<Vec<String>>()
                .join(", "),
        )
    }
}

impl Declare for OctetString {
    fn declare(&self) -> String {
        format!(
            "OctetString {{ constraints: vec![{}] }}",
            self.constraints
                .iter()
                .map(|c| c.declare())
                .collect::<Vec<String>>()
                .join(", "),
        )
    }
}

impl Declare for BitString {
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

impl Declare for CharacterString {
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

impl Declare for SequenceOf {
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

impl Declare for SequenceOrSet {
    fn declare(&self) -> String {
        format!(
            "SequenceOrSet {{ constraints: vec![{}], extensible: {}, members: vec![{}] }}",
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

impl Declare for SequenceOrSetMember {
    fn declare(&self) -> String {
        format!(
          "SequenceOrSetMember {{ name: \"{}\".into(), tag: {}, is_optional: {}, r#type: {}, default_value: {}, constraints: vec![{}] }}",
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

impl Declare for Choice {
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

impl Declare for ChoiceOption {
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

impl Declare for Enumerated {
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

impl Declare for Enumeral {
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

impl Declare for DistinguishedValue {
    fn declare(&self) -> String {
        format!(
            "DistinguishedValue {{ name: \"{}\".into(), value: {} }}",
            self.name, self.value
        )
    }
}

impl Declare for Constraint {
    fn declare(&self) -> String {
        match self {
            Constraint::SubtypeConstraint(r) => {
                format!("Constraint::SubtypeConstraint({})", r.declare())
            }
            Constraint::TableConstraint(t) => {
                format!("Constraint::TableConstraint({})", t.declare())
            }
            Constraint::Parameter(params) => {
                format!(
                    "Constraint::Parameter(vec![{}])",
                    params
                        .iter()
                        .map(|p| p.declare())
                        .collect::<Vec<String>>()
                        .join(",")
                )
            }
        }
    }
}

impl Declare for Parameter {
    fn declare(&self) -> String {
        match self {
            Parameter::ValueParameter(v) => format!("Parameter::ValueParameter({})", v.declare()),
            Parameter::TypeParameter(t) => format!("Parameter::TypeParameter({})", t.declare()),
            Parameter::InformationObjectParameter(i) => {
                format!("Parameter::InformationObjectParameter({})", i.declare())
            }
            Parameter::ObjectSetParameter(i) => {
                format!("Parameter::ObjectSetParameter({})", i.declare())
            }
        }
    }
}

impl Declare for CompositeConstraint {
    fn declare(&self) -> String {
        format!(
            "CompositeConstraint {{ extensible: {}, base_constraint: {}, operation: vec![{}] }}",
            self.extensible,
            self.base_constraint.declare(),
            self.operation
                .iter()
                .map(|(op, c)| format!("(SetOperation::{:?}, {})", op, c.declare()))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl Declare for InnerTypeConstraint {
    fn declare(&self) -> String {
        format!(
            "InnerTypeConstraint {{ is_partial: {}, constraints: vec![{}] }}",
            self.is_partial,
            self.constraints
                .iter()
                .map(|c| c.declare())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl Declare for ConstrainedComponent {
    fn declare(&self) -> String {
        format!(
          "ConstrainedComponent {{ identifier: \"{}\".into(), constraints: vec![{}], presence: ComponentPresence::{:?} }}",
          self.identifier,
          self.constraints
              .iter()
              .map(|c| c.declare())
              .collect::<Vec<String>>()
              .join(", "),
          self.presence
      )
    }
}

impl Declare for ValueConstraint {
    fn declare(&self) -> String {
        format!(
            "ValueConstraint {{ min_value: {}, max_value: {}, extensible: {} }}",
            self.min_value
                .as_ref()
                .map_or("None".to_owned(), |m| "Some(".to_owned()
                    + &m.declare()
                    + ")"),
            self.max_value
                .as_ref()
                .map_or("None".to_owned(), |m| "Some(".to_owned()
                    + &m.declare()
                    + ")"),
            self.extensible
        )
    }
}

impl Declare for TableConstraint {
    fn declare(&self) -> String {
        format!(
            "TableConstraint {{ object_set: {}, linked_fields: vec![{}] }}",
            self.object_set.declare(),
            self.linked_fields
                .iter()
                .map(|v| v.declare())
                .collect::<Vec<String>>()
                .join(", "),
        )
    }
}

impl Declare for RelationalConstraint {
    fn declare(&self) -> String {
        format!(
            r#"RelationalConstraint {{ field_name: "{}".into(), level: {} }}"#,
            self.field_name,
            self.level,
        )
    }
}

impl Declare for SubtypeElement {
    fn declare(&self) -> String {
        match self {
            SubtypeElement::SingleValue { value, extensible } => {
                format!(
                    "SubtypeElement::SingleValue {{ value: {}, extensible: {extensible} }}",
                    value.declare()
                )
            }
            SubtypeElement::PermittedAlphabet(permitted) => {
                format!(
                    "SubtypeElement::PermittedAlphabet(Box::new({}))",
                    permitted.declare()
                )
            }
            SubtypeElement::ContainedSubtype {
                subtype,
                extensible,
            } => {
                format!(
                    "SubtypeElement::ContainedSubtype {{ subtype: {}, extensible: {extensible} }}",
                    subtype.declare()
                )
            }
            SubtypeElement::ValueRange {
                min,
                max,
                extensible,
            } => {
                format!(
                    "SubtypeElement::ValueRange {{ min: {}, max: {}, extensible: {extensible} }}",
                    min.as_ref()
                        .map_or("None".to_owned(), |m| format!("Some({})", m.declare())),
                    max.as_ref()
                        .map_or("None".to_owned(), |m| format!("Some({})", m.declare())),
                )
            }
            SubtypeElement::SizeConstraint(i) => {
                format!("SubtypeElement::SizeConstraint(Box::new({}))", i.declare())
            }
            SubtypeElement::TypeConstraint(t) => {
                format!("SubtypeElement::TypeConstraint({})", t.declare())
            }
            SubtypeElement::SingleTypeConstraint(s) => {
                format!("SubtypeElement::SingleTypeConstraint({})", s.declare())
            }
            SubtypeElement::MultipleTypeConstraints(m) => {
                format!("SubtypeElement::MultipleTypeConstraints({})", m.declare())
            }
        }
    }
}

impl Declare for ElementSet {
    fn declare(&self) -> String {
        format!(
            "ElementSet {{ set: {}, extensible: {} }}",
            self.set.declare(),
            self.extensible
        )
    }
}

impl Declare for ElementOrSetOperation {
    fn declare(&self) -> String {
        match self {
            ElementOrSetOperation::Element(e) => {
                format!("ElementOrSetOperation::Element({})", e.declare())
            }
            ElementOrSetOperation::SetOperation(x) => {
                format!("ElementOrSetOperation::SetOperation({})", x.declare())
            }
        }
    }
}

impl Declare for SetOperation {
    fn declare(&self) -> String {
        format!(
            "SetOperation {{ base: {}, operator: SetOperator::{:?}, operant: Box::new({}) }}",
            self.base.declare(),
            self.operator,
            self.operant.declare()
        )
    }
}


impl Declare for ASN1Information {
    fn declare(&self) -> String {
        match self {
            Self::ObjectClass(c) => format!("ASN1Information::ObjectClass({})", c.declare()),
            Self::ObjectSet(s) => format!("ASN1Information::ObjectSet({})", s.declare()),
            Self::Object(o) => {
                format!("ASN1Information::Object({})", o.declare())
            }
        }
    }
}

impl Declare for InformationObjectFieldReference {
    fn declare(&self) -> String {
        format!("InformationObjectFieldReference {{ class: \"{}\".into(), field_path: vec![{}], constraints: vec![{}] }}",
      self.class,
    self.field_path.iter().map(|f| f.declare()).collect::<Vec<String>>().join(", "),
    self.constraints.iter().map(|c| c.declare()).collect::<Vec<String>>().join(", "))
    }
}

impl Declare for SyntaxExpression {
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

impl Declare for SyntaxApplication {
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

impl Declare for SyntaxToken {
    fn declare(&self) -> String {
        match self {
            SyntaxToken::Literal(l) => format!("SyntaxToken::Literal(\"{l}\".into())"),
            SyntaxToken::Comma => "SyntaxToken::Comma".to_owned(),
            SyntaxToken::Field(o) => format!("SyntaxToken::Field({})", o.declare()),
        }
    }
}

impl Declare for InformationObjectSyntax {
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

impl Declare for InformationObjectClass {
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

impl Declare for InformationObjectClassField {
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

impl Declare for ObjectFieldIdentifier {
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

impl Declare for InformationObject {
    fn declare(&self) -> String {
        format!(
            "InformationObject {{ supertype: \"{}\".into(), fields: {} }}",
            self.supertype,
            self.fields.declare()
        )
    }
}

impl Declare for InformationObjectFields {
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

impl Declare for ObjectSet {
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

impl Declare for ObjectSetValue {
    fn declare(&self) -> String {
        match self {
            ObjectSetValue::Reference(r) => format!(r#"ObjectSetValue::Reference("{r}".into())"#),
            ObjectSetValue::Inline(i) => format!("ObjectSetValue::Inline({})", i.declare()),
        }
    }
}

impl Declare for InformationObjectField {
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

impl Declare for FixedValueField {
    fn declare(&self) -> String {
        format!(
            "FixedValueField {{ identifier: \"{}\".into(), value: {} }}",
            self.identifier,
            self.value.declare()
        )
    }
}

impl Declare for TypeField {
    fn declare(&self) -> String {
        format!(
            "TypeField {{ identifier: \"{}\".into(), r#type: {} }}",
            self.identifier,
            self.r#type.declare()
        )
    }
}

impl Declare for ObjectSetField {
    fn declare(&self) -> String {
        format!(
            "ObjectSetField {{ identifier: \"{}\".into(), value: {} }}",
            self.identifier,
            self.value.declare()
        )
    }
}
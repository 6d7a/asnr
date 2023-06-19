use alloc::{borrow::ToOwned, boxed::Box, string::ToString, vec};

use crate::{subtyping::*, *};

/// Representation of an ASN1 INTEGER data element
/// with corresponding constraints and distinguished values
#[derive(Debug, Clone, PartialEq)]
pub struct AsnInteger {
    pub constraints: Vec<ValueConstraint>,
    pub distinguished_values: Option<Vec<DistinguishedValue>>,
}

impl Quote for AsnInteger {
    fn quote(&self) -> String {
        format!(
            "AsnInteger {{ constraints: vec![{}], distinguished_values: {} }}",
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

impl Default for AsnInteger {
    fn default() -> Self {
        Self {
            constraints: vec![],
            distinguished_values: None,
        }
    }
}

impl From<ValueConstraint> for AsnInteger {
    fn from(value: ValueConstraint) -> Self {
        Self {
            constraints: vec![value],
            distinguished_values: None,
        }
    }
}

impl From<(Option<i128>, Option<i128>, bool)> for AsnInteger {
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
    )> for AsnInteger
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
pub struct AsnBitString {
    pub constraints: Vec<ValueConstraint>,
    pub distinguished_values: Option<Vec<DistinguishedValue>>,
}

impl From<(Option<Vec<DistinguishedValue>>, Option<ValueConstraint>)> for AsnBitString {
    fn from(value: (Option<Vec<DistinguishedValue>>, Option<ValueConstraint>)) -> Self {
        AsnBitString {
            constraints: value.1.map_or(vec![], |r| vec![r]),
            distinguished_values: value.0,
        }
    }
}

impl Quote for AsnBitString {
    fn quote(&self) -> String {
        format!(
            "AsnBitString {{ constraints: vec![{}], distinguished_values: {} }}",
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
pub struct AsnCharacterString {
    pub constraints: Vec<Constraint>,
    pub r#type: CharacterStringType,
}

impl From<(&str, Option<ValueConstraint>)> for AsnCharacterString {
    fn from(value: (&str, Option<ValueConstraint>)) -> Self {
        AsnCharacterString {
            constraints: value
                .1
                .map_or(vec![], |r| vec![Constraint::ValueConstraint(r)]),
            r#type: value.0.into(),
        }
    }
}

impl Quote for AsnCharacterString {
    fn quote(&self) -> String {
        format!(
            "AsnCharacterString {{ constraints: vec![{}], r#type: CharacterStringType::{:?} }}",
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
pub struct AsnSequenceOf {
    pub constraints: Vec<Constraint>,
    pub r#type: Box<ASN1Type>,
}

impl Quote for AsnSequenceOf {
    fn quote(&self) -> String {
        format!(
            "AsnSequenceOf {{ constraints: vec![{}], r#type: {} }}",
            self.constraints
                .iter()
                .map(|c| c.quote())
                .collect::<Vec<String>>()
                .join(", "),
            String::from("Box::new(") + &self.r#type.quote() + ")",
        )
    }
}

impl From<(Option<Vec<Constraint>>, ASN1Type)> for AsnSequenceOf {
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
pub struct AsnSequence {
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
    )> for AsnSequence
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
        AsnSequence {
            constraints: value.1.unwrap_or(vec![]),
            extensible: value.0 .1.map(|_| index_of_first_extension),
            members: value.0 .0,
        }
    }
}

impl Quote for AsnSequence {
    fn quote(&self) -> String {
        format!(
            "AsnSequence {{ constraints: vec![{}], extensible: {}, members: vec![{}] }}",
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
pub struct AsnChoice {
    pub extensible: Option<usize>,
    pub options: Vec<ChoiceOption>,
    pub constraints: Vec<Constraint>,
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
            constraints: vec![],
        }
    }
}

impl Quote for AsnChoice {
    fn quote(&self) -> String {
        format!(
            "AsnChoice {{ extensible: {}, options: vec![{}], constraints: vec![{}] }}",
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
pub struct AsnEnumerated {
    pub members: Vec<Enumeral>,
    pub extensible: Option<usize>,
    pub constraints: Vec<Constraint>,
}

impl Quote for AsnEnumerated {
    fn quote(&self) -> String {
        format!(
            "AsnEnumerated {{ members: vec![{}], extensible: {}, constraints: vec![{}] }}",
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
pub struct InformationObjectClass<'a> {
  pub fields: Vec<InformationObjectField<'a>>
}

#[derive(Debug, Clone, PartialEq)]
pub struct InformationObjectField<'a> {
  pub identifier: ObjectFieldIdentifier<'a>,
  pub r#type: Option<ASN1Type>,
  pub is_optional: bool,
  pub is_unique: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectFieldIdentifier<'a> {
  SingleValue(&'a str),
  MultipleValue(&'a str)
}
use core::fmt::Debug;

use alloc::{borrow::ToOwned, boxed::Box, string::ToString, vec};

use crate::{constraints::*, *};

/// Representation of an ASN1 INTEGER data element
/// with corresponding constraints and distinguished values
#[derive(Debug, Clone, PartialEq)]
pub struct Integer {
    pub constraints: Vec<Constraint>,
    pub distinguished_values: Option<Vec<DistinguishedValue>>,
}

impl Integer {
    pub fn type_token(&self) -> String {
        let (min, max) =
            self.constraints
                .iter()
                .fold((i128::MAX, i128::MIN), |(mut min, mut max), c| {
                    if let Ok((cmin, cmax, _)) = c.unpack_as_value_range() {
                        if let Some(ASN1Value::Integer(i)) = cmin {
                            min = (*i).min(min);
                        };
                        if let Some(ASN1Value::Integer(i)) = cmax {
                            max = (*i).max(max);
                        };
                    };
                    (min, max)
                });
        if min > max {
            "i128".to_owned()
        } else {
            int_type_token(min, max).to_owned()
        }
    }
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

impl From<(i128, i128, bool)> for Integer {
    fn from(value: (i128, i128, bool)) -> Self {
        Self {
            constraints: vec![Constraint::SubtypeConstraint(ElementSet {
                set: ElementOrSetOperation::Element(SubtypeElement::ValueRange {
                    min: Some(ASN1Value::Integer(value.0)),
                    max: Some(ASN1Value::Integer(value.1)),
                    extensible: value.2
                }),
                extensible: value.2,
            })],
            distinguished_values: None,
        }
    }
}

impl From<(Option<i128>, Option<i128>, bool)> for Integer {
    fn from(value: (Option<i128>, Option<i128>, bool)) -> Self {
        Self {
            constraints: vec![Constraint::SubtypeConstraint(ElementSet {
                set: ElementOrSetOperation::Element(SubtypeElement::ValueRange {
                    min: value.0.map(|v| ASN1Value::Integer(v)),
                    max: value.1.map(|v| ASN1Value::Integer(v)),
                    extensible: value.2
                }),
                extensible: value.2,
            })],
            distinguished_values: None,
        }
    }
}

impl
    From<(
        &str,
        Option<Vec<DistinguishedValue>>,
        Option<Vec<Constraint>>,
    )> for Integer
{
    fn from(
        value: (
            &str,
            Option<Vec<DistinguishedValue>>,
            Option<Vec<Constraint>>,
        ),
    ) -> Self {
        Self {
            constraints: value.2.unwrap_or(vec![]),
            distinguished_values: value.1,
        }
    }
}

/// Representation of an ASN1 REAL data element
/// with corresponding constraints
#[derive(Debug, Clone, PartialEq)]
pub struct Real {
    pub constraints: Vec<Constraint>,
}

impl asnr_traits::Declare for Real {
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

impl From<Option<Vec<Constraint>>> for Real {
    fn from(value: Option<Vec<Constraint>>) -> Self {
        Self {
            constraints: value.unwrap_or(vec![])
        }
    }
}

/// Representation of an ASN1 BIT STRING data element
/// with corresponding constraints and distinguished values
/// defining the individual bits
#[derive(Debug, Clone, PartialEq)]
pub struct BitString {
    pub constraints: Vec<Constraint>,
    pub distinguished_values: Option<Vec<DistinguishedValue>>,
}

impl From<(Option<Vec<DistinguishedValue>>, Option<Vec<Constraint>>)> for BitString {
    fn from(value: (Option<Vec<DistinguishedValue>>, Option<Vec<Constraint>>)) -> Self {
        BitString {
            constraints: value.1.unwrap_or(vec![]),
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

impl From<(&str, Option<Vec<Constraint>>)> for CharacterString {
    fn from(value: (&str, Option<Vec<Constraint>>)) -> Self {
        CharacterString {
            constraints: value.1.unwrap_or(vec![]),
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
pub struct SequenceOrSet {
    pub extensible: Option<usize>,
    pub constraints: Vec<Constraint>,
    pub members: Vec<SequenceOrSetMember>,
}

impl
    From<(
        (
            Vec<SequenceOrSetMember>,
            Option<ExtensionMarker>,
            Option<Vec<SequenceOrSetMember>>,
        ),
        Option<Vec<Constraint>>,
    )> for SequenceOrSet
{
    fn from(
        mut value: (
            (
                Vec<SequenceOrSetMember>,
                Option<ExtensionMarker>,
                Option<Vec<SequenceOrSetMember>>,
            ),
            Option<Vec<Constraint>>,
        ),
    ) -> Self {
        let index_of_first_extension = value.0 .0.len();
        value.0 .0.append(&mut value.0 .2.unwrap_or(vec![]));
        SequenceOrSet {
            constraints: value.1.unwrap_or(vec![]),
            extensible: value.0 .1.map(|_| index_of_first_extension),
            members: value.0 .0,
        }
    }
}

impl asnr_traits::Declare for SequenceOrSet {
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

/// Representation of an single ASN1 SEQUENCE member
#[derive(Debug, Clone, PartialEq)]
pub struct SequenceOrSetMember {
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
    )> for SequenceOrSetMember
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
        SequenceOrSetMember {
            name: value.0.into(),
            tag: value.1,
            r#type: value.2,
            is_optional: value.4.is_some() || value.5.is_some(),
            default_value: value.5,
            constraints: value.3.unwrap_or(vec![]),
        }
    }
}

impl asnr_traits::Declare for SequenceOrSetMember {
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
    pub index: i128,
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

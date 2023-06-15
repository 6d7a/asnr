use alloc::{
    borrow::ToOwned,
    format,
    string::{String},
    vec,
    vec::Vec,
};

use crate::{Quote, ASN1Value};

#[derive(Debug, PartialEq)]
pub struct OptionalMarker();

impl From<&str> for OptionalMarker {
    fn from(_: &str) -> Self {
        OptionalMarker()
    }
}

#[derive(Debug)]
pub struct RangeMarker();

#[derive(Debug, Clone, PartialEq)]
pub struct ExtensionMarker();

#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    RangeConstraint(RangeConstraint),
    ComponentConstraint(ComponentConstraint),
    Arithmetic(ArithmeticOperator),
    ArrayComponentConstraint(ComponentConstraint),
    //CharacterConstraint(CharacterConstraint)
}

impl Quote for Constraint {
    fn quote(&self) -> String {
        match self {
            Constraint::RangeConstraint(r) => format!("Constraint::RangeConstraint({})", r.quote()),
            Constraint::ComponentConstraint(c) => {
                format!("Constraint::ComponentConstraint({})", c.quote())
            }
            Constraint::ArrayComponentConstraint(c) => {
                format!("Constraint::ArrayComponentConstraint({})", c.quote())
            }
            Constraint::Arithmetic(o) => {
                format!("Constraint::Arithmetic(ArithmeticOperator::{:?})", o)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithmeticOperator {
    Intersection,
    Union,
    Except,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentPresence {
    Absent,
    Present,
    Unspecified,
}

/// Representation of a component constraint used for subtyping
/// in ASN1 specifications
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentConstraint {
    pub is_partial: bool,
    pub constraints: Vec<ConstrainedComponent>,
}

impl Quote for ComponentConstraint {
    fn quote(&self) -> String {
        format!(
            "ComponentConstraint {{ is_partial: {}, constraints: vec![{}] }}",
            self.is_partial,
            self.constraints
                .iter()
                .map(|c| c.quote())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl
    From<(
        Option<ExtensionMarker>,
        Vec<(&str, Option<Vec<Constraint>>, Option<ComponentPresence>)>,
    )> for ComponentConstraint
{
    fn from(
        value: (
            Option<ExtensionMarker>,
            Vec<(&str, Option<Vec<Constraint>>, Option<ComponentPresence>)>,
        ),
    ) -> Self {
        ComponentConstraint {
            is_partial: value.0.is_some(),
            constraints: value
                .1
                .into_iter()
                .map(|(id, constraint, presence)| ConstrainedComponent {
                    identifier: String::from(id),
                    constraints: constraint.unwrap_or(vec![]),
                    presence: presence.unwrap_or(ComponentPresence::Unspecified),
                })
                .collect(),
        }
    }
}

/// Representation of a single component within a component constraint
/// in ASN1 specifications
#[derive(Debug, Clone, PartialEq)]
pub struct ConstrainedComponent {
    pub identifier: String,
    pub constraints: Vec<Constraint>,
    pub presence: ComponentPresence,
}

impl Quote for ConstrainedComponent {
    fn quote(&self) -> String {
        format!(
          "ConstrainedComponent {{ identifier: \"{}\".into(), constraint: vec![{}], presence: ComponentPresence::{:?} }}",
          self.identifier,
          self.constraints
              .iter()
              .map(|c| c.quote())
              .collect::<Vec<String>>()
              .join(", "),
          self.presence
      )
    }
}

/// Representation of a range constraint used for subtyping
/// in ASN1 specifications
#[derive(Debug, Clone, PartialEq)]
pub struct RangeConstraint {
    pub min_value: Option<ASN1Value>,
    pub max_value: Option<ASN1Value>,
    pub extensible: bool,
}

impl Quote for RangeConstraint {
    fn quote(&self) -> String {
        format!(
            "RangeConstraint {{ min_value: {}, max_value: {}, extensible: {} }}",
            self.min_value.as_ref()
                .map_or("None".to_owned(), |m| "Some(".to_owned()
                    + &m.quote()
                    + ")"),
            self.max_value.as_ref()
                .map_or("None".to_owned(), |m| "Some(".to_owned()
                    + &m.quote()
                    + ")"),
            self.extensible
        )
    }
}

impl<'a> From<i128> for RangeConstraint {
    fn from(value: i128) -> Self {
        Self {
            min_value: Some(ASN1Value::Integer(value)),
            max_value: Some(ASN1Value::Integer(value)),
            extensible: false,
        }
    }
}

impl<'a> From<(i128, RangeMarker, i128)> for RangeConstraint {
    fn from(value: (i128, RangeMarker, i128)) -> Self {
        Self {
            min_value: Some(ASN1Value::Integer(value.0)),
            max_value: Some(ASN1Value::Integer(value.2)),
            extensible: false,
        }
    }
}

impl<'a> From<(i128, ExtensionMarker)> for RangeConstraint {
    fn from(value: (i128, ExtensionMarker)) -> Self {
        Self {
            min_value: Some(ASN1Value::Integer(value.0)),
            max_value: Some(ASN1Value::Integer(value.0)),
            extensible: true,
        }
    }
}

impl<'a> From<(i128, RangeMarker, i128, ExtensionMarker)> for RangeConstraint {
    fn from(value: (i128, RangeMarker, i128, ExtensionMarker)) -> Self {
        Self {
            min_value: Some(ASN1Value::Integer(value.0)),
            max_value: Some(ASN1Value::Integer(value.2)),
            extensible: true,
        }
    }
}

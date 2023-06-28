use alloc::{
    borrow::ToOwned,
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::{information_object::ObjectSet, ASN1Value, Declare};

#[derive(Debug, PartialEq)]
pub struct OptionalMarker();

impl From<&str> for OptionalMarker {
    fn from(_: &str) -> Self {
        OptionalMarker()
    }
}

#[derive(Debug)]
pub struct RangeSeperator();

#[derive(Debug, Clone, PartialEq)]
pub struct ExtensionMarker();

#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    CompositeConstraint(CompositeConstraint),
    ValueConstraint(ValueConstraint),
    SizeConstraint(ValueConstraint),
    ComponentConstraint(ComponentConstraint),
    ArrayComponentConstraint(ComponentConstraint),
    TableConstraint(TableConstraint),
    //CharacterConstraint(CharacterConstraint)
}

impl asnr_traits::Declare for Constraint {
    fn declare(&self) -> String {
        match self {
            Constraint::ValueConstraint(r) => {
                format!("Constraint::ValueConstraint({})", r.declare())
            }
            Constraint::SizeConstraint(r) => format!("Constraint::SizeConstraint({})", r.declare()),
            Constraint::ComponentConstraint(c) => {
                format!("Constraint::ComponentConstraint({})", c.declare())
            }
            Constraint::ArrayComponentConstraint(c) => {
                format!("Constraint::ArrayComponentConstraint({})", c.declare())
            }
            Constraint::CompositeConstraint(o) => {
                format!("Constraint::CompositeConstraint({})", o.declare())
            }
            Constraint::TableConstraint(t) => {
                format!("Constraint::TableConstraint({})", t.declare())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetOperation {
    Intersection,
    Union,
    Except,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompositeConstraint {
    pub base_constraint: Box<Constraint>,
    pub operation: Option<(SetOperation, Box<Constraint>)>,
    pub extensible: bool,
}

impl
    From<(
        Constraint,
        Option<(SetOperation, Constraint)>,
        Option<ExtensionMarker>,
    )> for CompositeConstraint
{
    fn from(
        value: (
            Constraint,
            Option<(SetOperation, Constraint)>,
            Option<ExtensionMarker>,
        ),
    ) -> Self {
        Self {
            base_constraint: Box::new(value.0),
            operation: value.1.map(|(op, c)| (op, Box::new(c))),
            extensible: value.2.is_some(),
        }
    }
}

impl asnr_traits::Declare for CompositeConstraint {
    fn declare(&self) -> String {
        format!(
            "CompositeConstraint {{ extensible: {}, base_constraint: {}, operation: {} }}",
            self.extensible,
            self.base_constraint.declare(),
            self.operation
                .as_ref()
                .map_or("None".to_owned(), |(op, c)| format!(
                    "Some((SetOperation::{:?},{}))",
                    op,
                    c.declare()
                )),
        )
    }
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

impl asnr_traits::Declare for ComponentConstraint {
    fn declare(&self) -> String {
        format!(
            "ComponentConstraint {{ is_partial: {}, constraints: vec![{}] }}",
            self.is_partial,
            self.constraints
                .iter()
                .map(|c| c.declare())
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

impl asnr_traits::Declare for ConstrainedComponent {
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

/// Representation of a range constraint used for subtyping
/// in ASN1 specifications
#[derive(Debug, Clone, PartialEq)]
pub struct ValueConstraint {
    pub min_value: Option<ASN1Value>,
    pub max_value: Option<ASN1Value>,
    pub extensible: bool,
}

impl asnr_traits::Declare for ValueConstraint {
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

impl<'a> From<ASN1Value> for ValueConstraint {
    fn from(value: ASN1Value) -> Self {
        Self {
            min_value: Some(value.clone()),
            max_value: Some(value),
            extensible: false,
        }
    }
}

impl<'a> From<(ASN1Value, RangeSeperator, ASN1Value)> for ValueConstraint {
    fn from(value: (ASN1Value, RangeSeperator, ASN1Value)) -> Self {
        Self {
            min_value: Some(value.0),
            max_value: Some(value.2),
            extensible: false,
        }
    }
}

impl<'a> From<(ASN1Value, ExtensionMarker)> for ValueConstraint {
    fn from(value: (ASN1Value, ExtensionMarker)) -> Self {
        Self {
            min_value: Some(value.0.clone()),
            max_value: Some(value.0),
            extensible: true,
        }
    }
}

impl<'a> From<(ASN1Value, RangeSeperator, ASN1Value, ExtensionMarker)> for ValueConstraint {
    fn from(value: (ASN1Value, RangeSeperator, ASN1Value, ExtensionMarker)) -> Self {
        Self {
            min_value: Some(value.0),
            max_value: Some(value.2),
            extensible: true,
        }
    }
}

/// Representation of a table constraint used for subtyping
/// in ASN1 specifications
/// _See: ITU-T X.682 (02/2021) 10_
#[derive(Debug, Clone, PartialEq)]
pub struct TableConstraint {
    pub object_set: ObjectSet,
    pub linked_fields: Vec<RelationalConstraint>,
}

impl asnr_traits::Declare for TableConstraint {
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

impl From<(ObjectSet, Option<Vec<RelationalConstraint>>)> for TableConstraint {
    fn from(value: (ObjectSet, Option<Vec<RelationalConstraint>>)) -> Self {
        Self {
            object_set: value.0,
            linked_fields: value.1.unwrap_or(vec![]),
        }
    }
}

/// Representation of a table's relational constraint
/// _See: ITU-T X.682 (02/2021) 10.7_
#[derive(Debug, Clone, PartialEq, Declare)]
pub struct RelationalConstraint {
    pub field_name: String,
    /// The level is null if the field is in the outermost object set of the declaration.
    /// The level is 1-n counting from the innermost object set of the declaration
    pub level: usize,
}

impl From<(usize, &str)> for RelationalConstraint {
    fn from(value: (usize, &str)) -> Self {
        Self {
            field_name: value.1.into(),
            level: value.0,
        }
    }
}

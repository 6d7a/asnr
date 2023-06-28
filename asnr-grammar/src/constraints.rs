use alloc::{borrow::ToOwned, boxed::Box, format, string::String, vec, vec::Vec};

use crate::{
    error::{GrammarError, GrammarErrorType},
    information_object::ObjectSet,
    ASN1Type, ASN1Value, DeclarationElsewhere, Declare,
};

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
    SubtypeConstraint(ElementSet),
    TableConstraint(TableConstraint),
    //CharacterConstraint(CharacterConstraint)
}

impl Constraint {
    pub fn unpack_as_value_range(
        &self,
    ) -> Result<(&Option<ASN1Value>, &Option<ASN1Value>, bool), GrammarError> {
        if let Constraint::SubtypeConstraint(set) = self {
            if let ElementOrSetOperation::Element(SubtypeElement::ValueRange {
                min,
                max,
                extensible,
            }) = &set.set
            {
                return Ok((min, max, *extensible));
            }
        }
        Err(GrammarError {
            details: format!(
                "Failed to unpack constraint as value range. Constraint: {:?}",
                self
            ),
            kind: GrammarErrorType::UnpackingError,
        })
    }
}

impl asnr_traits::Declare for Constraint {
    fn declare(&self) -> String {
        match self {
            Constraint::SubtypeConstraint(r) => {
                format!("Constraint::SubtypeConstraint({})", r.declare())
            }
            Constraint::TableConstraint(t) => {
                format!("Constraint::TableConstraint({})", t.declare())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetOperator {
    Intersection,
    Union,
    Except,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompositeConstraint {
    pub base_constraint: Box<Constraint>,
    pub operation: Vec<(SetOperator, Box<Constraint>)>,
    pub extensible: bool,
}

impl
    From<(
        Constraint,
        Vec<(SetOperator, Constraint)>,
        Option<ExtensionMarker>,
    )> for CompositeConstraint
{
    fn from(
        value: (
            Constraint,
            Vec<(SetOperator, Constraint)>,
            Option<ExtensionMarker>,
        ),
    ) -> Self {
        Self {
            base_constraint: Box::new(value.0),
            operation: value
                .1
                .into_iter()
                .map(|(op, c)| (op, Box::new(c)))
                .collect(),
            extensible: value.2.is_some(),
        }
    }
}

impl asnr_traits::Declare for CompositeConstraint {
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

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentPresence {
    Absent,
    Present,
    Unspecified,
}

/// Representation of a component constraint used for subtyping
/// in ASN1 specifications
#[derive(Debug, Clone, PartialEq)]
pub struct InnerTypeConstraint {
    pub is_partial: bool,
    pub constraints: Vec<ConstrainedComponent>,
}

impl asnr_traits::Declare for InnerTypeConstraint {
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

#[derive(Debug, Clone, PartialEq)]
pub enum SubtypeElement {
    SingleValue {
        value: ASN1Value,
        extensible: bool,
    },
    ContainedSubtype {
        subtype: ASN1Type,
        extensible: bool,
    },
    ValueRange {
        min: Option<ASN1Value>,
        max: Option<ASN1Value>,
        extensible: bool,
    },
    // PermittedAlphabet
    SizeConstraint(Box<ElementOrSetOperation>),
    TypeConstraint(ASN1Type),
    SingleTypeConstraint(InnerTypeConstraint),
    MultipleTypeConstraints(InnerTypeConstraint),
    // PatternConstraint
    // PropertySettings
    // DurationRange
    // TimePointRange
    // RecurrenceRange
}

impl From<(ASN1Value, Option<ExtensionMarker>)> for SubtypeElement {
    fn from(value: (ASN1Value, Option<ExtensionMarker>)) -> Self {
        Self::SingleValue {
            value: value.0,
            extensible: value.1.is_some(),
        }
    }
}

impl From<Constraint> for SubtypeElement {
    fn from(value: Constraint) -> Self {
        match value {
            Constraint::SubtypeConstraint(set) => Self::SizeConstraint(Box::new(set.set)),
            _ => unreachable!(),
        }
    }
}

impl
    From<(
        Option<ExtensionMarker>,
        Vec<(&str, Option<Vec<Constraint>>, Option<ComponentPresence>)>,
    )> for SubtypeElement
{
    fn from(
        value: (
            Option<ExtensionMarker>,
            Vec<(&str, Option<Vec<Constraint>>, Option<ComponentPresence>)>,
        ),
    ) -> Self {
        SubtypeElement::SingleTypeConstraint(InnerTypeConstraint {
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
        })
    }
}

impl Declare for SubtypeElement {
    fn declare(&self) -> String {
        match self {
            SubtypeElement::SingleValue { value, extensible } => {
                format!(
                    "SubtypeElements::SingleValue {{ value: {}, extensible: {extensible} }}",
                    value.declare()
                )
            }
            SubtypeElement::ContainedSubtype {
                subtype,
                extensible,
            } => {
                format!(
                    "SubtypeElements::ContainedSubtype {{ subtype: {}, extensible: {extensible} }}",
                    subtype.declare()
                )
            }
            SubtypeElement::ValueRange {
                min,
                max,
                extensible,
            } => {
                format!(
                    "SubtypeElements::ValueRange {{ min: {}, max: {}, extensible: {extensible} }}",
                    min.as_ref()
                        .map_or("None".to_owned(), |m| format!("Some({})", m.declare())),
                    max.as_ref()
                        .map_or("None".to_owned(), |m| format!("Some({})", m.declare())),
                )
            }
            SubtypeElement::SizeConstraint(i) => {
                format!("SubtypeElements::SizeConstraint(Box::new({}))", i.declare())
            }
            SubtypeElement::TypeConstraint(t) => {
                format!("SubtypeElements::TypeConstraint({})", t.declare())
            }
            SubtypeElement::SingleTypeConstraint(s) => {
                format!("SubtypeElements::SingleTypeConstraint({})", s.declare())
            }
            SubtypeElement::MultipleTypeConstraints(m) => {
                format!("SubtypeElements::MultipleTypeConstraints({})", m.declare())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElementSet {
    pub set: ElementOrSetOperation,
    pub extensible: bool,
}

impl From<(ElementOrSetOperation, Option<ExtensionMarker>)> for ElementSet {
    fn from(value: (ElementOrSetOperation, Option<ExtensionMarker>)) -> Self {
        Self {
            set: value.0,
            extensible: value.1.is_some(),
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

#[derive(Debug, Clone, PartialEq)]
pub enum ElementOrSetOperation {
    Element(SubtypeElement),
    SetOperation(SetOperation),
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

#[derive(Debug, Clone, PartialEq)]
pub struct SetOperation {
    pub base: SubtypeElement, //TODO: Handle exclusions
    pub operator: SetOperator,
    pub operant: Box<ElementOrSetOperation>,
}

impl From<(SubtypeElement, SetOperator, ElementOrSetOperation)> for SetOperation {
    fn from(value: (SubtypeElement, SetOperator, ElementOrSetOperation)) -> Self {
        Self {
            base: value.0,
            operator: value.1,
            operant: Box::new(value.2),
        }
    }
}

impl Declare for SetOperation {
    fn declare(&self) -> String {
        format!(
            "SetOperation {{ base: {}, operator: SetOperator::{:?}, operant: {} }}",
            self.base.declare(),
            self.operator,
            self.operant.declare()
        )
    }
}

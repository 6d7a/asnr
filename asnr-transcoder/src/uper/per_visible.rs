use core::ops::{Add, AddAssign};

use asnr_grammar::{
    constraints::{
        Constraint, ElementOrSetOperation, SetOperation, SetOperator, SubtypeElement,
    },
    ASN1Value, types::Enumerated,
};

use crate::error::DecodingError;

trait PerVisible {
    fn per_visible(&self) -> bool;
}

pub struct PerVisibleIntegerConstraints {
    min: Option<i128>,
    max: Option<i128>,
    extensible: bool,
}

impl Default for PerVisibleIntegerConstraints {
    fn default() -> Self {
        Self { min: None, max: None, extensible: false }
    }
}

impl PerVisibleIntegerConstraints {
    pub fn bit_size(&self) -> Option<usize> {
        self.min.zip(self.max).map(|(min, max)| {
            let range = max - min;
            let mut power = 1;
            while (range + 1) > 2_i128.pow(power) {
                power += 1;
            }
            power as usize
        })
    }

    pub fn is_extensible(&self) -> bool {
      self.extensible
    }

    pub fn min<I: num::Integer + num::FromPrimitive>(&self) -> Option<I> {
      self.min.map(|m| I::from_i128(m)).flatten()
    }

    pub fn as_enum_constraint(&mut self, enumerated: &Enumerated) {
        *self += PerVisibleIntegerConstraints {
            min: Some(0),
            max: Some(enumerated.members.len() as i128 - 1),
            extensible: enumerated.extensible.is_some()
        };
    }
}

impl AddAssign<PerVisibleIntegerConstraints> for PerVisibleIntegerConstraints {
    fn add_assign(&mut self, rhs: PerVisibleIntegerConstraints) {
        self.min = self.min.max(rhs.min);
        self.max = match (self.max, rhs.max) {
          (Some(m1), Some(m2)) => Some(m1.min(m2)),
          (None, Some(m)) | (Some(m), None) => Some(m),
          _ => None
        };
        self.extensible = self.extensible || rhs.extensible;
    }
}

impl TryFrom<Constraint> for PerVisibleIntegerConstraints {
    type Error = DecodingError;

    fn try_from(value: Constraint) -> Result<PerVisibleIntegerConstraints, DecodingError> {
        match value {
            Constraint::SubtypeConstraint(c) => match c.set {
                ElementOrSetOperation::Element(e) => Some(e).try_into(),
                ElementOrSetOperation::SetOperation(s) => fold_constraint_set(&s)?.try_into(),
            },
            Constraint::TableConstraint(_) => Ok(Self::default()),
        }
    }
}

impl TryFrom<Option<SubtypeElement>> for PerVisibleIntegerConstraints {
    type Error = DecodingError;
    fn try_from(
        value: Option<SubtypeElement>,
    ) -> Result<PerVisibleIntegerConstraints, DecodingError> {
        match value {
            None => Ok(Self::default()),
            Some(SubtypeElement::SingleValue { value, extensible }) => {
                let val = value.unwrap_as_integer()?;
                Ok(Self {
                    min: Some(val),
                    max: Some(val),
                    extensible,
                })
            }
            Some(SubtypeElement::ValueRange {
                min,
                max,
                extensible,
            }) => Ok(Self {
                min: min.map(|i| i.unwrap_as_integer()).transpose()?,
                max: max.map(|i| i.unwrap_as_integer()).transpose()?,
                extensible,
            }),
            _ => unreachable!()
        }
    }
}

impl PerVisible for Constraint {
    fn per_visible(&self) -> bool {
        match self {
            Constraint::TableConstraint(_) => false,
            Constraint::SubtypeConstraint(s) => s.set.per_visible(),
        }
    }
}

impl PerVisible for ElementOrSetOperation {
    fn per_visible(&self) -> bool {
        match self {
            ElementOrSetOperation::Element(e) => e.per_visible(),
            ElementOrSetOperation::SetOperation(o) => {
                o.operant.per_visible() || o.operant.per_visible()
            }
        }
    }
}

impl PerVisible for SubtypeElement {
    fn per_visible(&self) -> bool {
        match self {
            SubtypeElement::SingleValue {
                value: _,
                extensible: _,
            } => true,
            SubtypeElement::ContainedSubtype {
                subtype: _,
                extensible: _,
            } => true,
            SubtypeElement::ValueRange {
                min: _,
                max: _,
                extensible: _,
            } => true,
            SubtypeElement::SizeConstraint(s) => s.per_visible(),
            _ => false,
        }
    }
}

/// 10.3.21	If a constraint that is PER-visible is part of an INTERSECTION construction,
/// then the resulting constraint is PER-visible, and consists of the INTERSECTION of
/// all PER-visible parts (with the non-PER-visible parts ignored).  
/// If a constraint which is not PER-visible is part of a UNION construction,
/// then the resulting constraint is not PER-visible.  
/// If a constraint has an EXCEPT clause, the EXCEPT and the following value set is completely ignored,
/// whether the value set following the EXCEPT is PER-visible or not.
fn fold_constraint_set(set: &SetOperation) -> Result<Option<SubtypeElement>, DecodingError> {
    let folded_operant = match &*set.operant {
        ElementOrSetOperation::Element(e) => e.per_visible().then(|| e.clone()),
        ElementOrSetOperation::SetOperation(s) => fold_constraint_set(s)?,
    };
    match set.operator {
        SetOperator::Intersection => match (&set.base, folded_operant) {
            (b, _) if !b.per_visible() => Ok(None),
            (b, None) => Ok(Some(b.clone())),
            (b, Some(f)) if !f.per_visible() => Ok(Some(b.clone())),
            (
                SubtypeElement::SingleValue {
                    value: _,
                    extensible: _,
                },
                _,
            ) => Ok(Some(set.base.clone())),
            (
                _,
                Some(SubtypeElement::SingleValue {
                    value: v,
                    extensible: x,
                }),
            ) => Ok(Some(SubtypeElement::SingleValue {
                value: v,
                extensible: x,
            })),
            (
                SubtypeElement::ValueRange {
                    min: min1,
                    max: max1,
                    extensible: x1,
                },
                Some(SubtypeElement::ValueRange {
                    min: min2,
                    max: max2,
                    extensible: x2,
                }),
            ) => {
                let min1_opt = min1.as_ref().map(|m| m.unwrap_as_integer()).transpose()?;
                let max1_opt = max1.as_ref().map(|m| m.unwrap_as_integer()).transpose()?;
                let min2_opt = min2.map(|m| m.unwrap_as_integer()).transpose()?;
                let max2_opt = max2.map(|m| m.unwrap_as_integer()).transpose()?;
                let min = min1_opt
                    .map_or(min2_opt, |min1_int| {
                        min2_opt.map(|min2_int| min1_int.max(min2_int))
                    })
                    .map(|i| ASN1Value::Integer(i));
                let max = max1_opt
                    .map_or(max2_opt, |max1_int| {
                        max2_opt.map(|max2_int| max1_int.min(max2_int))
                    })
                    .map(|i| ASN1Value::Integer(i));
                Ok(Some(SubtypeElement::ValueRange {
                    min,
                    max,
                    extensible: *x1 || x2,
                }))
            }
            _ => unreachable!(),
        },
        SetOperator::Union => match (&set.base, folded_operant) {
            (b, _) if !b.per_visible() => Ok(None),
            (_, None) => Ok(None),
            (_, Some(f)) if !f.per_visible() => Ok(None),
            (
                SubtypeElement::SingleValue {
                    value: v1,
                    extensible: x1,
                },
                Some(SubtypeElement::SingleValue {
                    value: v2,
                    extensible: x2,
                }),
            ) => {
                if v1 == &v2 {
                    Ok(Some(SubtypeElement::SingleValue {
                        value: v2,
                        extensible: *x1 || x2,
                    }))
                } else {
                    let v1_int = v1.unwrap_as_integer()?;
                    let v2_int = v2.unwrap_as_integer()?;
                    Ok(Some(SubtypeElement::ValueRange {
                        min: Some(ASN1Value::Integer(v1_int.min(v2_int))),
                        max: Some(ASN1Value::Integer(v1_int.max(v2_int))),
                        extensible: *x1 || x2,
                    }))
                }
            }
            (
                SubtypeElement::ValueRange {
                    min,
                    max,
                    extensible,
                },
                Some(SubtypeElement::SingleValue {
                    value: v,
                    extensible: x,
                }),
            ) => {
                let min_opt = min.as_ref().map(|m| m.unwrap_as_integer()).transpose()?;
                let max_opt = max.as_ref().map(|m| m.unwrap_as_integer()).transpose()?;
                let v_int = v.unwrap_as_integer()?;
                Ok(Some(SubtypeElement::ValueRange {
                    min: min_opt.map(|min_int| ASN1Value::Integer(min_int.min(v_int))),
                    max: max_opt.map(|max_int| ASN1Value::Integer(max_int.min(v_int))),
                    extensible: *extensible || x,
                }))
            }
            (
                SubtypeElement::SingleValue {
                    value: v,
                    extensible: x,
                },
                Some(SubtypeElement::ValueRange {
                    min,
                    max,
                    extensible,
                }),
            ) => {
                let min_opt = min.as_ref().map(|m| m.unwrap_as_integer()).transpose()?;
                let max_opt = max.as_ref().map(|m| m.unwrap_as_integer()).transpose()?;
                let v_int = v.unwrap_as_integer()?;
                Ok(Some(SubtypeElement::ValueRange {
                    min: min_opt.map(|min_int| ASN1Value::Integer(min_int.min(v_int))),
                    max: max_opt.map(|max_int| ASN1Value::Integer(max_int.min(v_int))),
                    extensible: extensible || *x,
                }))
            }
            (
                SubtypeElement::ValueRange {
                    min: min1,
                    max: max1,
                    extensible: x1,
                },
                Some(SubtypeElement::ValueRange {
                    min: min2,
                    max: max2,
                    extensible: x2,
                }),
            ) => {
                let min1_opt = min1.as_ref().map(|m| m.unwrap_as_integer()).transpose()?;
                let max1_opt = max1.as_ref().map(|m| m.unwrap_as_integer()).transpose()?;
                let min2_opt = min2.map(|m| m.unwrap_as_integer()).transpose()?;
                let max2_opt = max2.map(|m| m.unwrap_as_integer()).transpose()?;
                let min = min1_opt
                    .map_or(min2_opt, |min1_int| {
                        min2_opt.map(|min2_int| min1_int.min(min2_int))
                    })
                    .map(|i| ASN1Value::Integer(i));
                let max = max1_opt
                    .map_or(max2_opt, |max1_int| {
                        max2_opt.map(|max2_int| max1_int.max(max2_int))
                    })
                    .map(|i| ASN1Value::Integer(i));
                Ok(Some(SubtypeElement::ValueRange {
                    min,
                    max,
                    extensible: *x1 || x2,
                }))
            }
            _ => unreachable!(),
        },
        SetOperator::Except => {
            if set.base.per_visible() {
                Ok(Some(set.base.clone()))
            } else {
                Ok(None)
            }
        }
    }
}

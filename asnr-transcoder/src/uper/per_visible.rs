
use asnr_grammar::{constraints::{Constraint, ElementSet, ElementOrSetOperation, SubtypeElement, SetOperation}};

use crate::error::DecodingError;

trait PerVisible {
    fn per_visible(&self) -> bool;
}

pub struct PerVisibleIntegerConstraints {
    min: Option<i128>,
    max: Option<i128>,
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
}

// impl From<&Vec<Constraint>> for PerVisibleIntegerConstraints {
//     fn from(value: &Vec<Constraint>) -> Self {
//         //TODO: Process serial constraints
//         match value.iter().filter(|c| c.per_visible()).next() {
//             Some(c) => match c {
//                 Constraint::ValueConstraint(v) => PerVisibleIntegerConstraints {
//                     min: (!v.extensible)
//                         .then(|| {
//                             if let Some(ASN1Value::Integer(min)) = v.min_value {
//                                 Some(min)
//                             } else {
//                                 None
//                             }
//                         })
//                         .flatten(),
//                     max: (!v.extensible)
//                         .then(|| {
//                             if let Some(ASN1Value::Integer(max)) = v.min_value {
//                                 Some(max)
//                             } else {
//                                 None
//                             }
//                         })
//                         .flatten(),
//                 },
//                 _ => unreachable!(),
//             },
//             None => PerVisibleIntegerConstraints {
//                 min: None,
//                 max: None,
//             },
//         }
//     }
// }

impl PerVisible for Constraint {
    fn per_visible(&self) -> bool {
        match self {
          Constraint::TableConstraint(_) => false,
          Constraint::SubtypeConstraint(s) => s.set.per_visible()
        }
    }
}

impl PerVisible for ElementOrSetOperation {
    fn per_visible(&self) -> bool {
        match self {
            ElementOrSetOperation::Element(e) => e.per_visible(),
            ElementOrSetOperation::SetOperation(o) => {
                o.operant.per_visible() || o.operant.per_visible()
            },
        }
    }
}

impl PerVisible for SubtypeElement {
    fn per_visible(&self) -> bool {
        match self {
            SubtypeElement::SingleValue { value: _, extensible: _ } => true,
            SubtypeElement::ContainedSubtype { subtype: _, extensible: _ } => true,
            SubtypeElement::ValueRange { min: _, max: _, extensible: _ } => true,
            SubtypeElement::SizeConstraint(s) => s.per_visible(),
            _ => false
        }
    }
}

fn fold_constraint_set(set: &SetOperation) -> Result<SubtypeElement, DecodingError> {
    todo!()
}
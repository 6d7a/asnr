use alloc::vec::Vec;
use asnr_grammar::{constraints::Constraint, ASN1Value};

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
          _ => true
        }
    }
}

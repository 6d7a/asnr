use asnr_grammar::{constraints::{Constraint, ElementOrSetOperation, SubtypeElement}, ASN1Value};

use crate::generator::error::{GeneratorError, GeneratorErrorType};

pub fn format_constraint_generics(constraints: &Vec<Constraint>) -> Result<Option<String>, GeneratorError> {
    if constraints.len() > 1 {
        return Err(GeneratorError {
            top_level_declaration: None,
            details: "rasn representations of types with more than one constraint are currently not supported! Please use asnr as a framework.".into(),
            kind: GeneratorErrorType::Unsupported
        });
    }
    if let Some(Constraint::SubtypeConstraint(set)) = constraints.first() {
        if let ElementOrSetOperation::Element(SubtypeElement::ValueRange { min: Some(ASN1Value::Integer(min)), max: Some(ASN1Value::Integer(max)), extensible: false }) = set.set {
            return Ok(Some(format!("<{}, {}>", min, max)));
        } else if let ElementOrSetOperation::Element(SubtypeElement::ValueRange { min: _, max: _, extensible: _ }) = set.set {
            return Err(GeneratorError {
                top_level_declaration: None,
                details: "rasn representations of semi-constraint and extensible integers are currently not supported! Please use asnr as a framework.".into(),
                kind: GeneratorErrorType::Unsupported
            });
        } else if let ElementOrSetOperation::Element(SubtypeElement::SingleValue { value: ASN1Value::Integer(i), extensible: false }) = set.set {
            return Ok(Some(format!("<{}, {}>", i, i)));
        } else if let ElementOrSetOperation::Element(SubtypeElement::SingleValue { value: _, extensible: _ }) = set.set {
            return Err(GeneratorError {
                top_level_declaration: None,
                details: "rasn representations of semi-constraint and extensible integers are currently not supported! Please use asnr as a framework.".into(),
                kind: GeneratorErrorType::Unsupported
            });
        } 
    }
    Ok(None)
}
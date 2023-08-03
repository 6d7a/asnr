use alloc::{format, string::String, vec::Vec};
use asnr_traits::Declare;

#[derive(Debug, Clone, PartialEq, Declare)]
pub struct Parameterization {
    pub parameters: Vec<ParameterizationArgument>,
}

impl From<Vec<(&str, Option<&str>)>> for Parameterization {
    fn from(value: Vec<(&str, Option<&str>)>) -> Self {
        Self {
            parameters: value
                .into_iter()
                .map(|(t, i)| ParameterizationArgument {
                    r#type: t.into(),
                    name: i.map(|n| n.into()),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Declare)]
pub struct ParameterizationArgument {
    pub r#type: String,
    pub name: Option<String>,
}

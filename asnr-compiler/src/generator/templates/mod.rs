use crate::generator::templates::asnr::util::rustify_name;

pub(crate) mod asnr;
pub(crate) mod rasn;

pub fn inner_name(name: &String, parent_name: &String) -> String {
    format!("{}_{}", parent_name, rustify_name(&name))
}
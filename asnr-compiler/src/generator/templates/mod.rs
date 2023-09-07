use asnr_grammar::utils::to_rust_title_case;

pub(crate) mod asnr;
pub(crate) mod rasn;

pub fn inner_name(name: &String, parent_name: &String) -> String {
    format!("{}{}", parent_name, to_rust_title_case(&name))
}
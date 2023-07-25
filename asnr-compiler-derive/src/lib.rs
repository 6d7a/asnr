extern crate proc_macro;

use proc_macro::TokenStream;

const DUMMY_HEADER: &'static str = r#"DUMMY { dummy(999) header(999)}

DEFINITIONS AUTOMATIC TAGS::= BEGIN
"#;
const DUMMY_FOOTER: &'static str = r#"END"#;

#[proc_macro]
pub fn asn1(item: TokenStream) -> TokenStream {
    let mut literal_asn1 = item.to_string();
    literal_asn1.remove(0);
    literal_asn1.pop();
    if !literal_asn1.contains("BEGIN") {
        literal_asn1 = String::from(DUMMY_HEADER) + &literal_asn1 + DUMMY_FOOTER;
    }
    asnr_compiler::Asnr::compiler()
        .add_asn_literal(&literal_asn1)
        .compile_to_string()
        .unwrap()
        .0
        .parse()
        .unwrap()
}

#[proc_macro]
pub fn asn1_no_std(item: TokenStream) -> TokenStream {
    let mut literal_asn1 = item.to_string();
    literal_asn1.remove(0);
    literal_asn1.pop();
    if !literal_asn1.contains("BEGIN") {
        literal_asn1 = String::from(DUMMY_HEADER) + &literal_asn1 + DUMMY_FOOTER;
    }
    asnr_compiler::Asnr::compiler()
        .add_asn_literal(&literal_asn1)
        .no_std(true)
        .compile_to_string()
        .unwrap()
        .0
        .parse()
        .unwrap()
}

#[proc_macro]
pub fn asn1_internal_tests(item: TokenStream) -> TokenStream {
    let mut literal_asn1 = item.to_string();
    literal_asn1.remove(0);
    literal_asn1.pop();
    if !literal_asn1.contains("BEGIN") {
        literal_asn1 = String::from(DUMMY_HEADER) + &literal_asn1 + DUMMY_FOOTER;
    }
    asnr_compiler::Asnr::compiler()
        .add_asn_literal(&literal_asn1)
        .no_std(true)
        .compile_to_string()
        .unwrap()
        .0
        .replace("asnr_transcoder", "crate")
        .parse()
        .unwrap()
}
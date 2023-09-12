extern crate proc_macro;

use asnr_compiler::Framework;
use proc_macro::TokenStream;
use syn::{Token, Path, parse::Parse, parse_macro_input, LitStr};

const DUMMY_HEADER: &'static str = r#"DUMMY { dummy(999) header(999)}

DEFINITIONS AUTOMATIC TAGS::= BEGIN
"#;
const DUMMY_FOOTER: &'static str = r#"END"#;

struct MacroInput {
    asn: LitStr,
    _comma1: Option<Token![,]>,
    framework: Option<Path>,
    _comma2: Option<Token![,]>,
    crate_root: Option<Path>,
}

impl Parse for MacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            asn: input.parse()?,
            _comma1: input.parse().ok(),
            framework: input.parse().ok(),
            _comma2: input.parse().ok(),
            crate_root: input.parse().ok(),
        })
    }
}


#[proc_macro]
pub fn asn1(input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(input as MacroInput);
    let framework = config.framework.map_or(
        Framework::Asnr, 
        |path| { 
            if path.segments.last().unwrap().ident.to_string() == "Rasn" {
                Framework::Rasn
            } else {
                Framework::Asnr
            } 
        }
    );

    let literal_asn1 = match config.asn.value() {
        v if v.contains("BEGIN") => v,
        v => String::from(DUMMY_HEADER) + &v + DUMMY_FOOTER
    };

    asnr_compiler::Asnr::new()
        .add_asn_literal(literal_asn1)
        .framework(framework)
        .compile_to_string()
        .map(|(res,_)| { if let Some(path) = config.crate_root {
            res.replace(
                "asnr_transcoder", 
                &path.segments
                .into_iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<String>>()
                .join("::")
            )
        } else {
            res
        }})
        .unwrap()
        .parse()
        .unwrap()
}
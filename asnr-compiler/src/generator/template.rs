use asnr_grammar::{ASN1Type, ToplevelDeclaration};

use super::error::{GeneratorError, GeneratorErrorType};

pub fn integer_template<'a>(
    tld: ToplevelDeclaration,
    custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Type::Integer(ref int) = tld.r#type {
        let name = tld.name;
        let integer_type = int
            .constraint
            .as_ref()
            .map_or("i128", |c| c.int_type_token());
        let min_str = int.constraint.as_ref().map(|c| c.min_value).flatten();
        let max_str = int.constraint.as_ref().map(|c| c.max_value).flatten();
        let extensible = int.constraint.as_ref().map_or(false, |c| c.extensible);
        let comments = if tld.comments.is_empty() {
            String::from("")
        } else {
            String::from("/* ") + &tld.comments + "*/\n"
        };
        let derive = custom_derive.unwrap_or("#[derive(Debug, Clone, PartialEq)]");
        Ok(format!(
            r#"
        {comments}{derive}
        pub struct {name}(pub {integer_type});

        impl Decode for {name} {{
            fn decode<'a, D>(decoder: D, input: &'a [u8]) -> nom::IResult<&'a [u8], Self>
            where
                D: asnr_transcoder::Decoder,
                Self: Sized,
            {{
                decoder
                    .decode_integer(({:?}, {:?}, {extensible}).into(), input)
                    .map(|(remaining, res)| (remaining, Self(res)))
            }}
        }}
        "#,
        min_str, max_str
        ))
    } else {
        Err(GeneratorError::new(
            tld,
            "Expected INTEGER top-level declaration",
            GeneratorErrorType::Asn1TypeMismatch,
        ))
    }
}

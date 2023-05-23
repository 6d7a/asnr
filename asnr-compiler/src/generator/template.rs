use asnr_grammar::{ASN1Type, Quote, ToplevelDeclaration};

use super::{
    error::{GeneratorError, GeneratorErrorType},
    util::{format_comments, format_enumeral, format_enumeral_from_int},
};

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
        let comments = format_comments(&tld.comments);
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
                    .decode_integer({}, input)
                    .map(|(remaining, res)| (remaining, Self(res)))
            }}
        }}
        "#,
            int.quote()
        ))
    } else {
        Err(GeneratorError::new(
            tld,
            "Expected INTEGER top-level declaration",
            GeneratorErrorType::Asn1TypeMismatch,
        ))
    }
}

pub fn boolean_template<'a>(
    tld: ToplevelDeclaration,
    custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Type::Boolean = tld.r#type {
        let name = tld.name;
        let comments = format_comments(&tld.comments);
        let derive = custom_derive.unwrap_or("#[derive(Debug, Clone, PartialEq)]");
        Ok(format!(
            r#"
      {comments}{derive}
      pub struct {name}(pub bool);

      impl Decode for {name} {{
          fn decode<'a, D>(decoder: D, input: &'a [u8]) -> nom::IResult<&'a [u8], Self>
          where
              D: asnr_transcoder::Decoder,
              Self: Sized,
          {{
              decoder
                  .decode_boolean(input)
                  .map(|(remaining, res)| (remaining, Self(res)))
          }}
      }}
      "#
        ))
    } else {
        Err(GeneratorError::new(
            tld,
            "Expected BOOLEAN top-level declaration",
            GeneratorErrorType::Asn1TypeMismatch,
        ))
    }
}

pub fn enumerated_template<'a>(
    tld: ToplevelDeclaration,
    custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Type::Enumerated(ref enumerated) = tld.r#type {
        let name = tld.name;
        let comments = format_comments(&tld.comments);
        let derive = custom_derive.unwrap_or("#[derive(Debug, Clone, PartialEq)]");
        Ok(format!(
            r#"
      {comments}{derive}
      pub enum {name} {{
        {}
      }}

      impl TryFrom<i128> for {name} {{
          type Error = DecodingError;
      
          fn try_from(v: i128) -> Result<Self, Self::Error> {{
              match v {{
                  {}
                  _ => Err(
                    DecodingError::new(
                      &format!("Invalid enumerated index decoding {name}. Received index {{}}",v), DecodingErrorType::InvalidEnumeratedIndex
                    )
                  ),
              }}
          }}
      }}

      impl Decode for {name} {{
          fn decode<'a, D>(decoder: D, input: &'a [u8]) -> nom::IResult<&'a [u8], Self>
          where
              D: asnr_transcoder::Decoder,
              Self: Sized,
          {{
              decoder.decode_enumerated(
                {}, 
                input
              )
          }}
      }}
      "#,
            enumerated
                .members
                .iter()
                .map(format_enumeral)
                .collect::<Vec<String>>()
                .join("\n\t"),
            enumerated
                .members
                .iter()
                .map(format_enumeral_from_int)
                .collect::<Vec<String>>()
                .join("\n\t\t  "),
            enumerated.quote()
        ))
    } else {
        Err(GeneratorError::new(
            tld,
            "Expected BOOLEAN top-level declaration",
            GeneratorErrorType::Asn1TypeMismatch,
        ))
    }
}

#[cfg(test)]
mod tests {
    use asnr_grammar::{ASN1Type, AsnEnumerated, Enumeral, ToplevelDeclaration};

    use crate::generator::template::enumerated_template;

    #[test]
    fn generates_enumerated_from_template() {
        let enum_tld = ToplevelDeclaration {
            name: "TestEnum".into(),
            comments: "".into(),
            r#type: ASN1Type::Enumerated(AsnEnumerated {
                members: vec![
                    Enumeral {
                        name: "forward".into(),
                        description: Some("This means forward".into()),
                        index: 1,
                    },
                    Enumeral {
                        name: "backward".into(),
                        description: Some("This means backward".into()),
                        index: 2,
                    },
                    Enumeral {
                        name: "unavailable".into(),
                        description: Some("This means nothing".into()),
                        index: 3,
                    },
                ],
                extensible: false,
            }),
        };
        println!("{}", enumerated_template(enum_tld, None).unwrap())
    }
}

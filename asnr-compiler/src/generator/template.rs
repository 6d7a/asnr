use asnr_grammar::{ASN1Type, Quote, ToplevelDeclaration};

use super::{
    error::{GeneratorError, GeneratorErrorType},
    util::{
        format_comments, format_distinguished_values, format_enumeral, format_enumeral_from_int,
        rustify_name, flatten_nested_sequence_members, format_sequence_members,
    },
};

pub fn integer_template<'a>(
    tld: ToplevelDeclaration,
    custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Type::Integer(ref int) = tld.r#type {
        let name = rustify_name(&tld.name);
        let integer_type = int
            .constraint
            .as_ref()
            .map_or("i128", |c| c.int_type_token());
        let comments = format_comments(&tld.comments);
        let derive = custom_derive.unwrap_or("#[derive(Debug, Clone, PartialEq)]");
        Ok(format!(
            r#"
{comments}{derive}
pub struct {name}(pub {integer_type});{}

impl Decode for {name} {{
    fn decode<'a, D>(decoder: D, input: &'a [u8]) -> nom::IResult<&'a [u8], Self>
    where
        D: Decoder,
        Self: Sized,
    {{
        decoder
            .decode_integer({})(input)
            .map(|(remaining, res)| (remaining, Self(res)))
    }}
}}
"#,
            format_distinguished_values(&tld),
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

pub fn bit_string_template<'a>(
    tld: ToplevelDeclaration,
    custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Type::BitString(ref bitstr) = tld.r#type {
        let name = rustify_name(&tld.name);
        let comments = format_comments(&tld.comments);
        let derive = custom_derive.unwrap_or("#[derive(Debug, Clone, PartialEq)]");
        Ok(format!(
            r#"
{comments}{derive}
pub struct {name}(pub Vec<bool>);{}

impl Decode for {name} {{
  fn decode<'a, D>(decoder: D, input: &'a [u8]) -> nom::IResult<&'a [u8], Self>
  where
      D: Decoder,
      Self: Sized,
  {{
      decoder
          .decode_bit_string({})(input)
          .map(|(remaining, res)| (remaining, Self(res)))
  }}
}}
"#,
            format_distinguished_values(&tld),
            bitstr.quote()
        ))
    } else {
        Err(GeneratorError::new(
            tld,
            "Expected BIT STRING top-level declaration",
            GeneratorErrorType::Asn1TypeMismatch,
        ))
    }
}

pub fn octet_string_template<'a>(
  tld: ToplevelDeclaration,
  custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
  if let ASN1Type::OctetString(ref octstr) = tld.r#type {
      let name = rustify_name(&tld.name);
      let comments = format_comments(&tld.comments);
      let derive = custom_derive.unwrap_or("#[derive(Debug, Clone, PartialEq)]");
      Ok(format!(
          r#"
{comments}{derive}
pub struct {name}(pub String);

impl Decode for {name} {{
fn decode<'a, D>(decoder: D, input: &'a [u8]) -> nom::IResult<&'a [u8], Self>
where
    D: Decoder,
    Self: Sized,
{{
    decoder
        .decode_octet_string({})(input)
        .map(|(remaining, res)| (remaining, Self(res)))
}}
}}
"#,
          octstr.quote()
      ))
  } else {
      Err(GeneratorError::new(
          tld,
          "Expected OCTET STRING top-level declaration",
          GeneratorErrorType::Asn1TypeMismatch,
      ))
  }
}

pub fn boolean_template<'a>(
    tld: ToplevelDeclaration,
    custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
    if let ASN1Type::Boolean = tld.r#type {
        let name = rustify_name(&tld.name);
        let comments = format_comments(&tld.comments);
        let derive = custom_derive.unwrap_or("#[derive(Debug, Clone, PartialEq)]");
        Ok(format!(
            r#"
{comments}{derive}
pub struct {name}(pub bool);

impl Decode for {name} {{
    fn decode<'a, D>(decoder: D, input: &'a [u8]) -> nom::IResult<&'a [u8], Self>
    where
        D: Decoder,
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
        let name = rustify_name(&tld.name);
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
        D: Decoder,
        Self: Sized,
    {{
        decoder.decode_enumerated({})( 
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


pub fn sequence_template<'a>(
  tld: ToplevelDeclaration,
  custom_derive: Option<&'a str>,
) -> Result<String, GeneratorError> {
  if let ASN1Type::Sequence(ref seq) = tld.r#type {
      let name = rustify_name(&tld.name);
      let comments = format_comments(&tld.comments);
      let derive = custom_derive.unwrap_or("#[derive(Debug, Clone, PartialEq)]");
      let inner_members = flatten_nested_sequence_members(&seq.members).join("\n");
      let members = format_sequence_members(&seq.members);
      Ok(format!(
          r#"
{inner_members}

{comments}{derive}
pub struct {name} {{
  {members}
}}

impl Decode for {name} {{
fn decode<'a, D>(decoder: D, input: &'a [u8]) -> nom::IResult<&'a [u8], Self>
where
    D: Decoder,
    Self: Sized,
{{
    decoder
        .decode_sequence({})(input)
        .map(|(remaining, res)| (remaining, res))
}}
}}
"#,
          seq.quote()
      ))
  } else {
      Err(GeneratorError::new(
          tld,
          "Expected SEQUENCE top-level declaration",
          GeneratorErrorType::Asn1TypeMismatch,
      ))
  }
}

#[cfg(test)]
mod tests {
    use asnr_grammar::{
        ASN1Type, AsnBitString, AsnEnumerated, AsnInteger, Constraint, DistinguishedValue,
        Enumeral, ToplevelDeclaration, AsnSequence, SequenceMember, DeclarationElsewhere, ASN1Value,
    };

    use crate::generator::template::{bit_string_template, enumerated_template, integer_template, sequence_template};

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

    #[test]
    fn generates_bitstring_from_template() {
        let bs_tld = ToplevelDeclaration {
            name: "BitString".into(),
            comments: "".into(),
            r#type: ASN1Type::BitString(AsnBitString {
                constraint: Some(Constraint {
                    max_value: Some(8),
                    min_value: Some(8),
                    extensible: true,
                }),
                distinguished_values: Some(vec![
                    DistinguishedValue {
                        name: "firstBit".into(),
                        value: 0,
                    },
                    DistinguishedValue {
                        name: "secondBit".into(),
                        value: 0,
                    },
                    DistinguishedValue {
                        name: "thirdBit".into(),
                        value: 0,
                    },
                ]),
            }),
        };
        println!("{}", bit_string_template(bs_tld, None).unwrap())
    }

    #[test]
    fn generates_integer_from_template() {
        let int_tld = ToplevelDeclaration {
            name: "TestInt".into(),
            comments: "".into(),
            r#type: ASN1Type::Integer(AsnInteger {
                constraint: Some(Constraint {
                    max_value: Some(1),
                    min_value: Some(8),
                    extensible: false,
                }),
                distinguished_values: Some(vec![
                    DistinguishedValue {
                        name: "negativeOutOfRange".into(),
                        value: -16898,
                    },
                    DistinguishedValue {
                        name: "positiveOutOfRange".into(),
                        value: 16898,
                    },
                    DistinguishedValue {
                        name: "invalid".into(),
                        value: 16899,
                    },
                ]),
            }),
        };
        println!("{}", integer_template(int_tld, None).unwrap())
    }

    #[test]
    fn generates_sequence_from_template() {
        let seq_tld = ToplevelDeclaration {
            name: "Sequence".into(),
            comments: "".into(),
            r#type: ASN1Type::Sequence(AsnSequence {
                extensible: false,
                members: vec![SequenceMember {
                  name: "nested".into(),
                  r#type: ASN1Type::Sequence(AsnSequence {
                      extensible: true,
                      members: vec![
                          SequenceMember {
                              name: "wow".into(),
                              r#type: ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere(
                                  "Wow".into()
                              )),
                              default_value: None,
                              is_optional: false
                          },
                          SequenceMember {
                              name: "this-is-annoying".into(),
                              r#type: ASN1Type::Boolean,
                              default_value: Some(ASN1Value::Boolean(true)),
                              is_optional: true
                          },
                          SequenceMember {
                              name: "another".into(),
                              r#type: ASN1Type::Sequence(AsnSequence {
                                  extensible: false,
                                  members: vec![SequenceMember {
                                      name: "inner".into(),
                                      r#type: ASN1Type::BitString(AsnBitString {
                                          constraint: Some(Constraint {
                                              min_value: Some(1),
                                              max_value: Some(1),
                                              extensible: true
                                          }),
                                          distinguished_values: None
                                      }),
                                      default_value: Some(ASN1Value::String("0".into())),
                                      is_optional: true
                                  }]
                              }),
                              default_value: None,
                              is_optional: true
                          }
                      ]
                  }),
                  default_value: None,
                  is_optional: false
              }],
            }),
        };
        println!("{}", sequence_template(seq_tld, None).unwrap())
    }
}

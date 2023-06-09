use asnr_grammar::*;

use super::builder::StringifiedNameType;

pub fn imports_and_generic_types(derive: Option<&str>) -> String {
    format!(
        r#"// This file has been auto-generated by ASNR
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_variables)]

use core::any::Any;
use alloc::{{format, vec, vec::Vec, string::String, boxed::Box}};
use asnr_grammar::{{*, types::*, constraints::*, information_object::*}};
use asnr_transcoder::{{error::{{DecodingError, DecodingErrorType}}, Decode, Decoder, DecodeMember, DecoderForIndex, Describe}};
use nom::{{AsBytes, IResult}};

/// This empty struct represents the ASN1 NULL value. 
pub struct ASN1_NULL;
pub struct ASN1_ALL(pub dyn Any);
{}
pub struct ASN1_OPEN(pub Vec<u8>);

impl<'a, I: AsBytes + 'a> Decode<'a, I> for ASN1_OPEN {{
  {DECODE_SIGNATURE}
  {{ 
    match ASN1_OPEN::decoder::<D>() {{
      Ok(mut decoder) => decoder(input),
      Err(_e) => Err(nom::Err::Error(nom::error::Error {{
        input,
        code: nom::error::ErrorKind::Fail,
      }}))
    }}
  }}

  {DECODER_SIGNATURE}
  {{
    Ok(Box::new(|input| D::decode_open_type(input).map(|(remaining, res)| (remaining, Self(res)))))
  }}
}}
"#,
        derive.unwrap_or(DERIVE_DEFAULT)
    )
}

pub const DERIVE_DEFAULT: &str = "#[derive(Debug, Clone, PartialEq, Default)]";

pub const DECODE_SIGNATURE: &str = r#"fn decode<D>(input: I) -> IResult<I, Self>
where
    D: Decoder<'a, I>,
    Self: Sized,"#;

pub const DECODER_SIGNATURE: &str = r#"fn decoder<D>() -> Result<Box<dyn FnMut(I) -> IResult<I, Self> + 'a>, DecodingError>
    where
        D: Decoder<'a, I>,
        Self: Sized,"#;

pub fn type_reference_value_template(
    comments: String,
    name: String,
    type_name: String,
    value: ASN1Value,
) -> String {
    format!(
        r#"
    {comments}
    pub const {name}: {type_name} = {};
    "#,
        value.to_string()
    )
}

pub fn typealias_template(
    comments: String,
    derive: &str,
    name: String,
    alias: String,
    descriptor: String,
) -> String {
    format!(
        r#"
    {comments}{derive}
    pub struct {name}(pub {alias});

    impl Describe for {name} {{
      fn describe() -> ASN1Type {{
        {descriptor}
      }}
    }}
    
    impl<'a, I: AsBytes + 'a> Decode<'a, I> for {name} {{
      {DECODE_SIGNATURE}
      {{
        match {name}::decoder::<D>() {{
          Ok(mut decoder) => decoder(input),
          Err(_e) => Err(nom::Err::Error(nom::error::Error {{
            input,
            code: nom::error::ErrorKind::Fail,
          }}))
        }}
      }}

      {DECODER_SIGNATURE}
      {{
        let mut inner_decoder = {alias}::decoder::<D>()?;
        Ok(Box::new(move |input| (*inner_decoder)(input).map(|(r, v)|(r, Self(v)))))
      }}
    }}
    "#
    )
}

pub fn integer_value_template(
    comments: String,
    name: String,
    vtype: &str,
    value: String,
) -> String {
    format!(
        r#"{comments}
pub const {name}: {vtype} = {value};
"#
    )
}

pub fn integer_template(
    comments: String,
    derive: &str,
    name: String,
    integer_type: String,
    distinguished_values: String,
    int_descriptor: String,
) -> String {
    format!(
        r#"
{comments}{derive}
pub struct {name}(pub {integer_type});{distinguished_values}

impl<'a, I: AsBytes + 'a> Decode<'a, I> for {name} {{
  {DECODE_SIGNATURE}
  {{
    match {name}::decoder::<D>() {{
      Ok(mut decoder) => decoder(input),
      Err(_e) => Err(nom::Err::Error(nom::error::Error {{
        input,
        code: nom::error::ErrorKind::Fail,
      }}))
    }}
  }}

  {DECODER_SIGNATURE}
  {{
    let mut int_decoder = D::decode_integer({int_descriptor})?;
    Ok(Box::new(move |input| (*int_decoder)(input).map(|(remaining, res)| (remaining, Self(res)))))
  }}
}}
"#
    )
}

pub fn bit_string_template(
    comments: String,
    derive: &str,
    name: String,
    distinguished_values: String,
    bitstr_descriptor: String,
) -> String {
    format!(
        r#"
{comments}{derive}
pub struct {name}(pub Vec<bool>);{distinguished_values}

impl<'a, I: AsBytes + 'a> Decode<'a, I> for {name} {{
  {DECODE_SIGNATURE}
  {{
    match {name}::decoder::<D>() {{
      Ok(mut decoder) => decoder(input),
      Err(_e) => Err(nom::Err::Error(nom::error::Error {{
        input,
        code: nom::error::ErrorKind::Fail,
      }}))
    }}
  }}

  {DECODER_SIGNATURE}
  {{
    let mut bitstring_decoder = D::decode_bit_string({bitstr_descriptor})?;
    Ok(Box::new(move |input| (*bitstring_decoder)(input).map(|(remaining, res)| (remaining, Self(res)))))
  }}
}}
"#,
    )
}

pub fn char_string_template(
    comments: String,
    derive: &str,
    name: String,
    charstr_descriptor: String,
) -> String {
    format!(
        r#"
{comments}{derive}
pub struct {name}(pub String);

impl<'a, I: AsBytes + 'a> Decode<'a, I> for {name} {{
  {DECODE_SIGNATURE}
  {{
    match {name}::decoder::<D>() {{
      Ok(mut decoder) => decoder(input),
      Err(_e) => Err(nom::Err::Error(nom::error::Error {{
        input,
        code: nom::error::ErrorKind::Fail,
      }}))
    }}
  }}

  {DECODER_SIGNATURE}
  {{
    let mut charstring_decoder = D::decode_character_string({charstr_descriptor})?;
    Ok(Box::new(move |input| (*charstring_decoder)(input).map(|(remaining, res)| (remaining, Self(res)))))
  }}
}}
"#,
    )
}

pub fn boolean_template(comments: String, derive: &str, name: String) -> String {
    format!(
        r#"
{comments}{derive}
pub struct {name}(pub bool);

impl<'a, I: AsBytes + 'a> Decode<'a, I> for {name} {{
  {DECODE_SIGNATURE}
  {{
    match {name}::decoder::<D>() {{
      Ok(mut decoder) => decoder(input),
      Err(_e) => Err(nom::Err::Error(nom::error::Error {{
        input,
        code: nom::error::ErrorKind::Fail,
      }}))
    }}
  }}
  
  {DECODER_SIGNATURE}
  {{
    Ok(Box::new(|input| D::decode_boolean(input).map(|(remaining, res)| (remaining, Self(res)))))
  }}
}}
"#
    )
}

pub fn null_value_template(comments: String, name: String) -> String {
    format!(
        r#"{comments}
pub const {name}: ASN1_NULL = ASN1_NULL;
"#
    )
}

pub fn null_template(comments: String, derive: &str, name: String) -> String {
    format!(
        r#"
{comments}{derive}
pub struct {name};

impl<'a, I: AsBytes + 'a> Decode<'a, I> for {name} {{
  {DECODE_SIGNATURE}
  {{
    match {name}::decoder::<D>() {{
      Ok(mut decoder) => decoder(input),
      Err(_e) => Err(nom::Err::Error(nom::error::Error {{
        input,
        code: nom::error::ErrorKind::Fail,
      }}))
    }}
  }}
  
  {DECODER_SIGNATURE}
  {{
    Ok(Box::new(|input| D::decode_null(input)))
  }}
}}
"#
    )
}

pub fn enumerated_template(
    comments: String,
    derive: &str,
    name: String,
    enumerals: String,
    enumerals_from_int: String,
    enum_descriptor: String,
) -> String {
    format!(
        r#"
  {comments}{derive}
  pub enum {name} {{
    #[default]
    {enumerals}
  }}
  
  impl TryFrom<i128> for {name} {{
    type Error = DecodingError;
  
    fn try_from(v: i128) -> Result<Self, Self::Error> {{
      match v {{
          {enumerals_from_int}
          _ => Err(
            DecodingError::new(
              &format!("Invalid enumerated index decoding {name}. Received index {{}}",v), DecodingErrorType::InvalidEnumeratedIndex
            )
          ),
      }}
    }}
  }}
  
  impl<'a, I: AsBytes + 'a> Decode<'a, I> for {name} {{
    {DECODE_SIGNATURE}
    {{
      match {name}::decoder::<D>() {{
        Ok(mut decoder) => decoder(input),
        Err(_e) => Err(nom::Err::Error(nom::error::Error {{
          input,
          code: nom::error::ErrorKind::Fail,
        }}))
      }}
    }}
    
    {DECODER_SIGNATURE}
    {{
      D::decode_enumerated({enum_descriptor})
    }}
  }}
  "#,
    )
}

pub fn sequence_template(
    comments: String,
    derive: &str,
    inner_members: String,
    name: String,
    member_declaration: String,
    extension_decl: String,
    decode_member_body: String,
    extension_decoder: String,
    seq_descriptor: String,
) -> String {
    format!(
        r#"
  {inner_members}
  
  {comments}{derive}
  pub struct {name} {{
    {member_declaration}{extension_decl}
  }}
  
  impl<'a, I: AsBytes + 'a> DecodeMember<'a, I> for {name} {{
    fn decode_member_at_index<D>(&mut self, index: usize, input: I) -> Result<I, nom::Err<nom::error::Error<I>>>
      where
          D: Decoder<'a, I>,
          Self: Sized,
    {{
      let mut input = input;
      match index {{
        {decode_member_body}
        _ => {extension_decoder}
      }}
      Ok(input)
    }}
  }}
  
  impl<'a, I: AsBytes + 'a> Decode<'a, I> for {name} {{
    {DECODE_SIGNATURE}
    {{
      match {name}::decoder::<D>() {{
        Ok(mut decoder) => decoder(input),
        Err(_e) => Err(nom::Err::Error(nom::error::Error {{
          input,
          code: nom::error::ErrorKind::Fail,
        }}))
      }}
    }}

    {DECODER_SIGNATURE}
    {{
      D::decode_sequence({seq_descriptor})
    }}
  }}
  "#
    )
}

pub fn sequence_of_template(
    comments: String,
    derive: &str,
    name: String,
    anonymous_item: String,
    member_type: String,
    seq_of_descriptor: String,
) -> String {
    format!(
        r#"{anonymous_item}

{comments}{derive}
pub struct {name}(pub Vec<{member_type}>);

impl<'a, I: AsBytes + 'a> Decode<'a, I> for {name} {{
  {DECODE_SIGNATURE}
  {{
    match {name}::decoder::<D>() {{
      Ok(mut decoder) => decoder(input),
      Err(_e) => Err(nom::Err::Error(nom::error::Error {{
        input,
        code: nom::error::ErrorKind::Fail,
      }}))
    }}
  }}

  {DECODER_SIGNATURE}
  {{
    let mut seq_of_decoder = D::decode_sequence_of({seq_of_descriptor}, |i| {{ {member_type}::decode::<D>(i) }})?;
    Ok(Box::new(move |input| (*seq_of_decoder)(input).map(|(remaining, res)| (remaining, Self(res)))))
  }}
}}
"#
    )
}

pub fn default_choice(option: &StringifiedNameType) -> String {
    format!(
        "Self::{name}({rtype}::default())",
        name = option.name,
        rtype = option.r#type
    )
}

pub fn choice_template(
    comments: String,
    derive: &str,
    name: String,
    anonymous_option: String,
    default_option: String,
    options: String,
    options_from_int: String,
    choice_descriptor: String,
) -> String {
    format!(
        r#"{anonymous_option}

{comments}{derive}
pub enum {name} {{
  {options}
}}

impl<'a, I: AsBytes + 'a> DecoderForIndex<'a, I> for {name} {{
  fn decoder_for_index<D>(v: i128) -> Result<fn(&D, I) -> IResult<I, Self>, DecodingError> where D: Decoder<'a, I>, Self: Sized {{
    match v {{
        {options_from_int}
        _ => Err(
          DecodingError::new(
            &format!("Invalid choice index decoding {name}. Received index {{}}",v), DecodingErrorType::InvalidChoiceIndex
          )
        ),
    }}
  }}
}}

impl Default for {name} {{
  fn default() -> Self {{
    {default_option}
  }}
}}

impl<'a, I: AsBytes + 'a> Decode<'a, I> for {name} {{
  {DECODE_SIGNATURE}
  {{
    match {name}::decoder::<D>() {{
      Ok(mut decoder) => decoder(input),
      Err(_e) => Err(nom::Err::Error(nom::error::Error {{
        input,
        code: nom::error::ErrorKind::Fail,
      }}))
    }}
  }}

  {DECODER_SIGNATURE}
  {{
    D::decode_choice({choice_descriptor})
  }}
}}
"#,
    )
}

pub fn information_object_class_template(
    comments: String,
    name: String,
    information_object_class_descriptor: String,
) -> String {
    format!(
        r#"{comments}
pub trait {name} {{
  fn descriptor() -> InformationObjectClass {{
    {information_object_class_descriptor}
  }}
}}
"#
    )
}

pub fn information_object_template(
    comments: String,
    derive: &str,
    inner_members: String,
    name: String,
    member_declaration: String,
    extension_decl: String,
    decode_member_body: String,
    extension_decoder: String,
    information_object_descriptor: String,
) -> String {
    format!(
        r#"
{inner_members}

{comments}{derive}
pub struct {name} {{
{member_declaration}{extension_decl}
}}

impl<'a, I: AsBytes + 'a> DecodeMember<'a, I> for {name} {{
fn decode_member_at_index<D>(&mut self, index: usize, input: I) -> Result<I, nom::Err<nom::error::Error<I>>>
  where
      D: Decoder<'a, I>,
      Self: Sized,
{{
  let mut input = input;
  match index {{
    {decode_member_body}
    _ => {extension_decoder}
  }}
  Ok(input)
}}
}}

impl<'a, I: AsBytes + 'a> Decode<'a, I> for {name} {{
  {DECODE_SIGNATURE}
  {{
    match {name}::decoder::<D>() {{
      Ok(mut decoder) => decoder(input),
      Err(_e) => Err(nom::Err::Error(nom::error::Error {{
        input,
        code: nom::error::ErrorKind::Fail,
      }}))
    }}
  }}

  {DECODER_SIGNATURE}
  {{
    D::decode_information_object({information_object_descriptor})
  }}
}}
"#
    )
}

pub fn information_object_set_template(
    comments: String,
    name: String,
    options: String,
    key_type: String,
    for_key_branches: String,
) -> String {
    format!(
        r#"{comments}
pub enum {name} {{
  {options}
}}

impl<I: AsBytes> DecoderForKey<I, {key_type}> for {name} {{
  fn decoder_for_key<I, D>(
    key: {key_type},
  ) -> Result<fn(&D, I) -> IResult<I, Self>, DecodingError>
  where
    D: Decoder,
    T: PartialEq,
    Self: Sized, 
  {{
    match key {{
      {for_key_branches}
    }}
  }}
}}
"#
    )
}

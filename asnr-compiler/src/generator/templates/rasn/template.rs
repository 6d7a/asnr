use asnr_grammar::*;

use super::builder::StringifiedNameType;

pub const RASN_DERIVES: &'static str =
    "#[derive(AsnType, Encode, Decode, Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]";
pub const RASN_EXTENSIBLE: &'static str = "#[non_exhaustive]";

pub fn imports_and_generic_types() -> String {
    format!(
        r#"#![no_std]

        extern crate alloc;
        
        use rasn::prelude::*;"#
    )
}

// pub fn type_reference_value_template(
//     comments: String,
//     name: String,
//     type_name: String,
//     value: ASN1Value,
// ) -> String {
//     format!(
//         r#"
//     {comments}
//     pub const {name}: {type_name} = {};
//     "#,
//         value.to_string()
//     )
// }

// pub fn typealias_template(
//     comments: String,
//     derive: &str,
//     name: String,
//     alias: String,
//     descriptor: String,
// ) -> String {
//     format!(
//         r#"
//     {comments}
//     pub type {name} = {alias}
//     "#
//     )
// }

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
    name: String,
    constraint_annotations: String,
    tag_annotations: String,
) -> String {
    format!(
        r#"
{comments}
#[derive(AsnType, Debug, Clone, Copy, Decode, Encode, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[rasn(delegate{constraint_annotations}{tag_annotations})]
pub struct {name}(pub Integer);
"#
    )
}

pub fn bit_string_template(
    comments: String,
    name: String,
    constraint_annotations: String,
    tag_annotations: String,
) -> String {
    format!(
        r#"
{comments}
#[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[rasn(delegate{constraint_annotations})]
pub struct {name}(pub BitString);
"#
    )
}

pub fn char_string_template(
    comments: String,
    name: String,
    constraint_annotations: String,
    alphabet_annotations: String,
    tag_annotations: String,
) -> String {
    format!(
        r#"
{comments}
#[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[rasn(delegate{constraint_annotations}{alphabet_annotations})]
pub struct {name}(pub BitString);
"#
    )
}

pub fn boolean_template(comments: String, name: String, tag_annotations: String) -> String {
    format!(
        r#"
{comments}
#[derive(AsnType, Debug, Clone, Copy, Decode, Encode, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[rasn(delegate)]
pub struct {name}(pub Boolean);
"#
    )
}

pub fn null_value_template(comments: String, name: String) -> String {
    format!(
        r#"{comments}
pub const {name} = ();
"#
    )
}

pub fn null_template(comments: String, name: String, tag_annotations: String) -> String {
    format!(
        r#"
{comments}
#[derive(AsnType, Debug, Clone, Copy, Decode, Encode, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[rasn(delegate)]
pub struct {name}(());
"#
    )
}

pub fn enumerated_template(
    comments: String,
    name: String,
    extensible: &str,
    enum_members: String,
    tag_annotations: String,
) -> String {
    format!(
        r#"
{comments}
#[derive(AsnType, Debug, Clone, Copy, Decode, Encode, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[rasn(enumerated)]{extensible}
pub enum {name} {{
    {enum_members}
}}
"#
    )
}

// pub fn sequence_template(
//     comments: String,
//     derive: &str,
//     inner_members: String,
//     name: String,
//     member_declaration: String,
//     extension_decl: String,
//     decode_member_body: String,
//     extension_decoder: String,
//     seq_descriptor: String,
// ) -> String {
//     format!(
//         r#"
//   {inner_members}

//   {comments}{derive}
//   pub struct {name} {{
//     {member_declaration}{extension_decl}
//   }}

//   impl<'a, I: AsBytes + Debug + 'a> DecodeMember<'a, I> for {name} {{
//     fn decode_member_at_index<D>(&mut self, index: usize, input: I) -> Result<I, DecodingError<I>>
//       where
//           D: Decoder<'a, I>,
//           Self: Sized,
//     {{
//       let mut input = input;
//       match index {{
//         {decode_member_body}
//         _ => {extension_decoder}
//       }}
//       Ok(input)
//     }}
//   }}

//   impl<'a, I: AsBytes + Debug + 'a> Decode<'a, I> for {name} {{
//     {DECODE_SIGNATURE}
//     {{
//       {name}::decoder::<D>()?(input)
//     }}

//     {DECODER_SIGNATURE}
//     {{
//       D::decode_sequence({seq_descriptor})
//     }}
//   }}
//   "#
//     )
// }

// pub fn sequence_of_template(
//     comments: String,
//     derive: &str,
//     name: String,
//     anonymous_item: String,
//     member_type: String,
//     seq_of_descriptor: String,
// ) -> String {
//     format!(
//         r#"{anonymous_item}

// {comments}{derive}
// pub struct {name}(pub Vec<{member_type}>);

// impl<'a, I: AsBytes + Debug + 'a> Decode<'a, I> for {name} {{
//   {DECODE_SIGNATURE}
//   {{
//     {name}::decoder::<D>()?(input)
//   }}

//   {DECODER_SIGNATURE}
//   {{
//     let mut seq_of_decoder = D::decode_sequence_of({seq_of_descriptor}, {member_type}::decode::<D>)?;
//     Ok(Box::new(move |input| (*seq_of_decoder)(input).map(|(remaining, res)| (remaining, Self(res)))))
//   }}
// }}
// "#
//     )
// }

// pub fn default_choice(option: &StringifiedNameType) -> String {
//     format!(
//         "Self::{name}({rtype}::default())",
//         name = option.name,
//         rtype = option.r#type
//     )
// }

// pub fn choice_template(
//     comments: String,
//     derive: &str,
//     name: String,
//     anonymous_option: String,
//     default_option: String,
//     options: String,
//     options_from_int: String,
//     unknown_index_case: String,
//     choice_descriptor: String,
// ) -> String {
//     format!(
//         r#"{anonymous_option}

// {comments}{derive}
// pub enum {name} {{
//   {options}
// }}

// impl<'a, I: AsBytes + Debug + 'a> DecoderForIndex<'a, I> for {name} {{
//   fn decoder_for_index<D>(v: i128) -> Result<fn(I) -> IResult<I, Self>, DecodingError> where D: Decoder<'a, I>, Self: Sized {{
//     match v {{
//         {options_from_int}
//         {unknown_index_case}
//     }}
//   }}
// }}

// impl Default for {name} {{
//   fn default() -> Self {{
//     {default_option}
//   }}
// }}

// impl<'a, I: AsBytes + Debug + 'a> Decode<'a, I> for {name} {{
//   {DECODE_SIGNATURE}
//   {{
//     {name}::decoder::<D>()?(input)
//   }}

//   {DECODER_SIGNATURE}
//   {{
//     D::decode_choice({choice_descriptor})
//   }}
// }}
// "#,
//     )
// }

// pub fn information_object_class_template(
//     comments: String,
//     name: String,
//     information_object_class_descriptor: String,
// ) -> String {
//     format!(
//         r#"{comments}
// pub trait {name} {{
//   fn descriptor() -> InformationObjectClass {{
//     {information_object_class_descriptor}
//   }}
// }}
// "#
//     )
// }

// pub fn information_object_template(
//     comments: String,
//     derive: &str,
//     inner_members: String,
//     name: String,
//     member_declaration: String,
//     extension_decl: String,
//     decode_member_body: String,
//     extension_decoder: String,
//     information_object_descriptor: String,
// ) -> String {
//     format!(
//         r#"
// {inner_members}

// {comments}{derive}
// pub struct {name} {{
// {member_declaration}{extension_decl}
// }}

// impl<'a, I: AsBytes + Debug + 'a> DecodeMember<'a, I> for {name} {{
// fn decode_member_at_index<D>(&mut self, index: usize, input: I) -> Result<I, nom::Err<nom::error::Error<I>>>
//   where
//       D: Decoder<'a, I>,
//       Self: Sized,
// {{
//   let mut input = input;
//   match index {{
//     {decode_member_body}
//     _ => {extension_decoder}
//   }}
//   Ok(input)
// }}
// }}

// impl<'a, I: AsBytes + Debug + 'a> Decode<'a, I> for {name} {{
//   {DECODE_SIGNATURE}
//   {{
//     match {name}::decoder::<D>() {{
//       Ok(mut decoder) => decoder(input),
//       Err(_e) => Err(nom::Err::Error(nom::error::Error {{
//         input,
//         code: nom::error::ErrorKind::Fail,
//       }}))
//     }}
//   }}

//   {DECODER_SIGNATURE}
//   {{
//     D::decode_information_object({information_object_descriptor})
//   }}
// }}
// "#
//     )
// }

// pub fn information_object_set_template(
//     comments: String,
//     name: String,
//     options: String,
//     key_type: String,
//     for_key_branches: String,
// ) -> String {
//     format!(
//         r#"{comments}
// pub enum {name} {{
//   {options}
// }}

// impl<I: AsBytes + Debug> DecoderForKey<I, {key_type}> for {name} {{
//   fn decoder_for_key<I, D>(
//     key: {key_type},
//   ) -> Result<fn(I) -> IResult<I, Self>, DecodingError>
//   where
//     D: Decoder,
//     T: PartialEq,
//     Self: Sized,
//   {{
//     match key {{
//       {for_key_branches}
//     }}
//   }}
// }}
// "#
//     )
// }

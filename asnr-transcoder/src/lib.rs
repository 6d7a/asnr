//! The `asnr-transcoder` library encodes and decodes data elements resulting from compiling
//! an ASN1 specification with the `asnr-compiler`.
//!
//! The transcoder aims to be suitable for `no_std` environments and `wasm-unknown` targets.
//! For a start, the asnr transcoder will provide support for UPER encoding rules,
//! but you can inject your own custom transcoder by implementing the `Decoder` and `Encoder` traits.
//!
#![no_std]
extern crate alloc;

pub mod error;
//#[cfg(feature = "uper")]
pub mod uper;

use core::any::Any;

use alloc::{string::String, vec::Vec};
use asnr_grammar::{types::*, ASN1Type};
use error::DecodingError;
use nom::IResult;

pub trait Decode {
    fn decode<'a, D>(decoder: &D, input: &'a [u8]) -> IResult<&'a [u8], Self>
    where
        D: Decoder,
        Self: Sized;
}

pub trait Describe {
  fn describe() -> ASN1Type;
}

pub trait DecodeMember {
    fn decode_member_at_index<'a, D>(
        &mut self,
        index: usize,
        decoder: &D,
        input: &'a [u8],
    ) -> Result<(&'a [u8], ()), DecodingError>
    where
        D: Decoder,
        Self: Sized;
}

pub trait DecoderForIndex {
    fn decoder_for_index<'a, D>(
        v: i128,
    ) -> Result<fn(&D, &'a [u8]) -> IResult<&'a [u8], Self>, DecodingError>
    where
        D: Decoder,
        Self: Sized;
}

pub trait DecoderForKey<T> {
  fn decoder_for_key<'a, D>(
      key: T,
  ) -> Result<fn(&D, &'a [u8]) -> IResult<&'a [u8], Self>, DecodingError>
  where
      D: Decoder,
      T: PartialEq,
      Self: Sized;
}

pub trait Decoder {
    fn decode_integer<'a, O: num::Integer + num::FromPrimitive>(
        &self,
        integer: Integer,
    ) -> fn(&'a [u8]) -> IResult<&'a [u8], O>;
    fn decode_enumerated<'a, O: TryFrom<i128>>(
        &self,
        enumerated: Enumerated,
    ) -> fn(&'a [u8]) -> IResult<&'a [u8], O>;
    fn decode_choice<'a, O: DecoderForIndex>(
        &self,
        choice: Choice,
    ) -> fn(&'a [u8]) -> IResult<&'a [u8], O>;
    fn decode_null<'a, N>(&self, input: &'a [u8]) -> IResult<&'a [u8], N>;
    fn decode_boolean<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], bool>;
    fn decode_bit_string<'a>(
        &self,
        bit_string: BitString,
    ) -> fn(&'a [u8]) -> IResult<&'a [u8], Vec<bool>>;
    fn decode_character_string<'a>(
        &self,
        char_string: CharacterString,
    ) -> fn(&'a [u8]) -> IResult<&'a [u8], String>;
    fn decode_sequence<'a, T: DecodeMember>(
        &self,
        sequence: Sequence,
    ) -> fn(&'a [u8]) -> IResult<&'a [u8], T>;
    fn decode_sequence_of<'a, T: Decode>(
        &self,
        sequence_of: SequenceOf,
        member_decoder: impl FnMut(&Self, &'a [u8]) -> IResult<&'a [u8], T>,
    ) -> fn(&'a [u8]) -> IResult<&'a [u8], Vec<T>>;
    fn decode_unknown_extension<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], &'a [u8]>;
}

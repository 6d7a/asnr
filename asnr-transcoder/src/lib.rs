//! The `asnr-transcoder` library encodes and decodes data elements resulting from compiling
//! an ASN1 specification with the `asnr-compiler`.
//!
//! The transcoder aims to be suitable for `no_std` environments and `wasm-unknown` targets.
//! For a start, the asnr transcoder will provide support for UPER encoding rules,
//! but you can inject your own custom transcoder by implementing the `Decoder` and `Encoder` traits.
//!
pub mod error;
//#[cfg(feature = "uper")]
pub mod uper;

use asnr_grammar::*;
use error::DecodingError;
use nom::IResult;
use num::{FromPrimitive, Integer};

mod generated;

pub trait Decode {
    fn decode<'a, D>(decoder: &D, input: &'a [u8]) -> IResult<&'a [u8], Self>
    where
        D: Decoder,
        Self: Sized;
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

pub trait Decoder {
    fn decode_integer<'a, O: Integer + FromPrimitive>(
        &self,
        integer: AsnInteger,
    ) -> fn(&'a [u8]) -> IResult<&'a [u8], O>;
    fn decode_enumerated<'a, O: TryFrom<i128>>(
        &self,
        enumerated: AsnEnumerated,
    ) -> fn(&'a [u8]) -> IResult<&'a [u8], O>;
    fn decode_choice<'a, O: TryFrom<i128>>(
      &self,
      enumerated: AsnEnumerated,
  ) -> fn(&'a [u8]) -> IResult<&'a [u8], O>;
    fn decode_boolean<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], bool>;
    fn decode_bit_string<'a>(
        &self,
        bit_string: AsnBitString,
    ) -> fn(&'a [u8]) -> IResult<&'a [u8], Vec<bool>>;
    fn decode_character_string<'a>(
        &self,
        char_string: AsnCharacterString,
    ) -> fn(&'a [u8]) -> IResult<&'a [u8], String>;
    fn decode_sequence<'a, T: DecodeMember>(
        &self,
        sequence: AsnSequence,
    ) -> fn(&'a [u8]) -> IResult<&'a [u8], T>;
    fn decode_sequence_of<'a, T: Decode>(
        &self,
        sequence_of: AsnSequenceOf,
        member_decoder: impl FnMut(&Self, &'a [u8]) -> IResult<&'a [u8], T>,
    ) -> fn(&'a [u8]) -> IResult<&'a [u8], Vec<T>>;
    fn decode_unknown_extension<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], &'a [u8]>;
}

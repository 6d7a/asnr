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
mod generated;
#[cfg(feature = "uper")]
pub mod uper;

use alloc::{string::String, vec::Vec};
use asnr_grammar::{types::*, ASN1Type};
use error::DecodingError;
use nom::{AsBytes, IResult};

pub trait Decode {
    fn decode<I, D>(decoder: &D, input: I) -> IResult<I, Self>
    where
        I: AsBytes,
        D: Decoder,
        Self: Sized;
}

pub trait Describe {
    fn describe() -> ASN1Type;
}

pub trait DecodeMember {
    fn decode_member_at_index<I, D>(
        &mut self,
        index: usize,
        decoder: &D,
        input: I,
    ) -> Result<(I, ()), DecodingError>
    where
        I: AsBytes,
        D: Decoder,
        Self: Sized;
}

pub trait DecoderForIndex {
    fn decoder_for_index<I: AsBytes, D>(v: i128) -> Result<fn(&D, I) -> IResult<I, Self>, DecodingError>
    where
        D: Decoder,
        Self: Sized;
}

pub trait DecoderForKey<T> {
    fn decoder_for_key<I: AsBytes, D>(key: T) -> Result<fn(&D, I) -> IResult<I, Self>, DecodingError>
    where
        D: Decoder,
        T: PartialEq,
        Self: Sized;
}

pub trait Decoder {
    fn decode_open_type<I: AsBytes>(&self, input: I) -> IResult<I, Vec<u8>>;
    fn decode_integer<I: AsBytes, O: num::Integer + num::FromPrimitive>(
        &self,
        integer: Integer,
    ) -> fn(I) -> IResult<I, O>;
    fn decode_enumerated<I: AsBytes, O: TryFrom<i128>>(
        &self,
        enumerated: Enumerated,
    ) -> fn(I) -> IResult<I, O>;
    fn decode_choice<I: AsBytes, O: DecoderForIndex>(&self, choice: Choice) -> fn(I) -> IResult<I, O>;
    fn decode_null<I: AsBytes, N>(&self, input: I) -> IResult<I, N>;
    fn decode_boolean<I: AsBytes>(&self, input: I) -> IResult<I, bool>;
    fn decode_bit_string<I: AsBytes>(&self, bit_string: BitString) -> fn(I) -> IResult<I, Vec<bool>>;
    fn decode_character_string<I: AsBytes>(
        &self,
        char_string: CharacterString,
    ) -> fn(I) -> IResult<I, String>;
    fn decode_sequence<I: AsBytes, T: DecodeMember>(&self, sequence: Sequence) -> fn(I) -> IResult<I, T>;
    fn decode_sequence_of<I: AsBytes, T: Decode>(
        &self,
        sequence_of: SequenceOf,
        member_decoder: impl FnMut(&Self, I) -> IResult<I, T>,
    ) -> fn(I) -> IResult<I, Vec<T>>;
    fn decode_unknown_extension<I: AsBytes>(&self, input: I) -> IResult<I, Vec<u8>>;
}

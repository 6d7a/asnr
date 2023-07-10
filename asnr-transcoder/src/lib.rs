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
#[cfg(feature = "uper")]
pub mod uper;
mod generated;

use alloc::{boxed::Box, string::String, vec::Vec};
use asnr_grammar::{types::*, ASN1Type};
use error::DecodingError;
use nom::{AsBytes, IResult};

pub trait Decode<I: AsBytes> {
    fn decode<D>(input: I) -> IResult<I, Self>
    where
        D: Decoder<I>,
        Self: Sized;

    fn decoder<D>() -> Result<Box<dyn FnMut(I) -> IResult<I, Self>>, DecodingError>
    where
        D: Decoder<I>,
        Self: Sized;
}

pub trait Describe {
    fn describe() -> ASN1Type;
}

pub trait DecodeMember<I: AsBytes> {
    fn decode_member_at_index<D>(
        &mut self,
        index: usize,
        input: I,
    ) -> Result<I, nom::Err<nom::error::Error<I>>>
    where
        D: Decoder<I>,
        Self: Sized;
}

pub trait DecoderForIndex<I: AsBytes> {
    fn decoder_for_index<D>(v: i128) -> Result<fn(&D, I) -> IResult<I, Self>, DecodingError>
    where
        D: Decoder<I>,
        Self: Sized;
}

pub trait DecoderForKey<I: AsBytes, T> {
    fn decoder_for_key<D>(key: T) -> Result<fn(&D, I) -> IResult<I, Self>, DecodingError>
    where
        D: Decoder<I>,
        T: PartialEq,
        Self: Sized;
}

pub trait Decoder<I: AsBytes> {
    fn decode_open_type(input: I) -> IResult<I, Vec<u8>>;
    fn decode_integer<O>(
        integer: Integer,
    ) -> Result<Box<dyn FnMut(I) -> IResult<I, O>>, DecodingError>
    where
        O: num::Integer + num::FromPrimitive + Copy;
    fn decode_enumerated<O: TryFrom<i128>>(
        enumerated: Enumerated,
    ) -> Result<Box<dyn FnMut(I) -> IResult<I, O>>, DecodingError>;
    fn decode_choice<O: DecoderForIndex<I>>(choice: Choice) -> Result<Box<dyn FnMut(I) -> IResult<I, O>>, DecodingError>;
    fn decode_null<N: Default>(input: I) -> IResult<I, N>;
    fn decode_boolean(input: I) -> IResult<I, bool>;
    fn decode_bit_string(bit_string: BitString) -> Result<Box<dyn FnMut(I) -> IResult<I, Vec<bool>>>, DecodingError>;
    fn decode_character_string(char_string: CharacterString) -> Result<Box<dyn FnMut(I) -> IResult<I, String>>, DecodingError>;
    fn decode_sequence<T: DecodeMember<I> + Default>(
        sequence: Sequence,
    ) -> Result<Box<dyn FnMut(I) -> IResult<I, T>>, DecodingError>;
    fn decode_sequence_of<T: Decode<I>>(
        sequence_of: SequenceOf,
        member_decoder: impl FnMut(I) -> IResult<I, T>,
    ) -> Result<Box<dyn FnMut(I) -> IResult<I, Vec<T>>>, DecodingError>;
    fn decode_unknown_extension(input: I) -> IResult<I, Vec<u8>>;
}

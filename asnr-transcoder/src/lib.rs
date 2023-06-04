//! The `asnr-transcoder` library encodes and decodes data elements resulting from compiling
//! an ASN1 specification with the `asnr-compiler`.
//!
//! The transcoder aims to be suitable for `no_std` environments and `wasm-unknown` targets.
//! For a start, the asnr transcoder will provide support for UPER encoding rules,
//! but you can inject your own custom transcoder by implementing the `Decoder` and `Encoder` traits.
//!
pub mod error;

use asnr_grammar::*;
use nom::IResult;
use num::Integer;

pub trait Decode {
    fn decode<'a, D>(decoder: D, input: &'a [u8]) -> IResult<&'a [u8], Self>
    where
        D: Decoder,
        Self: Sized;
}

pub trait Decoder {
    fn decode_integer<'a, O: Integer>(
        &self,
        integer: AsnInteger,
        input: &'a [u8],
    ) -> IResult<&'a [u8], O>;
    fn decode_enumerated<'a, O>(
        &self,
        enumerated: AsnEnumerated,
        input: &'a [u8],
    ) -> IResult<&'a [u8], O>;
    fn decode_boolean<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], bool>;
    fn decode_bit_string<'a>(
        &self,
        bit_string: AsnBitString,
        input: &'a [u8],
    ) -> IResult<&'a [u8], Vec<bool>>;
    fn decode_octet_string<'a>(
        &self,
        bit_string: AsnBitString,
        input: &'a [u8],
    ) -> IResult<&'a [u8], String>;
    fn decode_sequence<'a, T>(
        &self,
        sequence: AsnSequence,
        input: &'a [u8],
    ) -> IResult<&'a [u8], T>;
}

//! The `asnr-transcoder` library encodes and decodes data elements resulting from compiling
//! an ASN1 specification with the `asnr-compiler`.
//!
//! The transcoder aims to be suitable for `no_std` environments and `wasm-unknown` targets.
//! For a start, the asnr transcoder will provide support for UPER encoding rules,
//! but you can inject your own custom transcoder by implementing the `Decoder` and `Encoder` traits.
//!
use asnr_grammar::AsnInteger;
use nom::IResult;
use num::Integer;

pub trait Decode {
    fn decode<'a, D>(decoder: D, input: &'a [u8]) -> IResult<&'a [u8], Self>
    where
        D: Decoder,
        Self: Sized;
}

pub trait Decoder {
    fn decode_integer<'a, O: Integer>(&self, integer: AsnInteger, input: &'a [u8]) -> IResult<&'a [u8], O>;
    fn decode_boolean<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], bool>;
    fn decode_bitstring<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], &'a str>;
    fn decode_sequence<'a, T>(&self, input: &'a [u8]) -> IResult<&'a [u8], T>;
}

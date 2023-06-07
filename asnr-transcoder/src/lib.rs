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
use nom::IResult;
use num::{FromPrimitive, Integer};

pub trait Decode {
    fn decode<'a, D>(decoder: &D, input: &'a [u8]) -> IResult<&'a [u8], Self>
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
    fn decode_boolean<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], bool>;
    fn decode_bit_string<'a>(
        &self,
        bit_string: AsnBitString,
    ) -> fn(&'a [u8]) -> IResult<&'a [u8], Vec<bool>>;
    fn decode_octet_string<'a>(
        &self,
        bit_string: AsnOctetString,
    ) -> fn(&'a [u8]) -> IResult<&'a [u8], String>;
    fn decode_extension_marker<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], bool>;
    fn decode_unknown_extension<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], Vec<u8>>;
    fn decode_sequence_of_size<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], usize>;
}
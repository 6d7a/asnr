//! The `asnr-parser` library encodes and decodes data elements resulting from compiling
//! an ASN1 specification with the `asnr-compiler`.
//! 
//! The parser aims to be suitable for `no_std` environments and `wasm-unknown` targets.
//! For a start, the asnr parser will provide support for UPER encoding rules, 
//! but you can inject your own custom parser by implementing the `Decode` and `Encode` traits.
//! 
use nom::IResult;
use num::Integer;

pub trait Decode {
  fn decode_integer<'a, O: Integer>(&self, input: &'a [u8]) -> IResult<&'a [u8], O>;
  fn decode_boolean<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], bool>;
  fn decode_bitstring<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], &'a str>;
  fn decode_sequence<'a, T>(&self, input: &'a [u8]) -> IResult<&'a [u8], T>;
}
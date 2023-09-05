//! # ASNR Transcoder
//! The transcoder crate handles the actual encoding and decoding of data at runtime.
//! It aims to be suitable for `no_std` environments and `wasm-unknown` targets.
//! For a start, the asnr transcoder will provide support for UPER encoding rules, 
//! but transcoding can be easily customized by implementing the crate's `Encoder` and `Decoder` traits.
//! 
//! The ASNR transcoder de- and encodes messages by composing functions that handle the
//! de-/encoding of generic ASN1 types like SEQUENCEs or INTEGERs. In the current implementation,
//! that choice has led to a lot of boxing and unboxing, but I hope to find a more efficient solution
//! in the future. The advantage of this design is that authors of custom encoders and decoders have
//! pretty much all of the information concerning the data element as it's specified in an 
//! ASN1 specification, including constraints, even comments up to a certain degree. 
//! 
//! ## Usage
//! Let's consider the following ASN1 Sequence:
//! ```asn1
//! ExampleSequence ::= SEQUENCE {
//!   member-1 IA5String (SIZE (1..24)),
//!   member-2 INTEGER (0..15),
//!   ...,
//!   extension BOOLEAN OPTIONAL
//! }
//! ```
//! 
//! ```rust
//! # use asnr_compiler_derive::asn1;
//! # asn1!(r#"ExampleSequence ::= SEQUENCE { member-1 IA5String (SIZE (1..24)), member-2 INTEGER (0..15), ..., extension BOOLEAN OPTIONAL }"#);
//! use asnr_transcoder::uper::Uper;
//! /// import your generated ASN1 representations
//! 
//! fn decode_example_sequence(binary: &[u8]) -> ExampleSequence {
//!   Uper::decode(binary).unwrap()
//! }
//! 
//! fn encode_example_sequence() -> Vec<u8> {
//!   let example_sequence = ExampleSequence {
//!     // ASN1-built-in types are represented as new types within SEQUENCEs
//!     member_1: InnerExampleSequenceMember1("Hello, World!".into()),
//!     member_2: InnerExampleSequenceMember2(8),
//!     extension: None
//!   };
//!   Uper::encode(example_sequence).unwrap()
//! }
//! ```
#![cfg_attr(not(test), no_std)]
extern crate alloc;

pub mod error;
#[cfg(feature = "uper")]
pub mod uper;
pub use nom::AsBytes;

use alloc::{boxed::Box, string::String, vec::Vec};
use asnr_grammar::{types::*, ASN1Type};
use core::fmt::Debug;
use error::{DecodingError, EncodingError};


pub type IResult<I, T> = Result<(I, T), DecodingError<I>>;

pub trait Decode<'a, I: AsBytes + Debug + 'a> {
    fn decode<D>(input: I) -> IResult<I, Self>
    where
        D: Decoder<'a, I>,
        Self: Sized;

    fn decoder<D>() -> Result<Box<dyn Fn(I) -> IResult<I, Self> + 'a>, DecodingError<I>>
    where
        D: Decoder<'a, I>,
        Self: Sized;
}

pub trait Encode<T, O: Extend<T> + Debug + 'static> {
    fn encode_self<E>(self, output: O) -> Result<O, EncodingError>
    where
        E: Encoder<T, O>,
        Self: Sized,
    {
        Self::encode::<E>(self, output)
    }

    fn encode<E>(encodable: Self, output: O) -> Result<O, EncodingError>
    where
        E: Encoder<T, O>,
        Self: Sized;

    fn encoder<E>() -> Result<Box<dyn Fn(Self, O) -> Result<O, EncodingError>>, EncodingError>
    where
        E: Encoder<T, O>,
        Self: Sized;
}

pub trait Describe {
    fn describe() -> ASN1Type;
}

pub trait DecodeMember<'a, I: AsBytes + Debug + 'a> {
    fn decode_member_at_index<D>(&mut self, index: usize, input: I) -> Result<I, DecodingError<I>>
    where
        D: Decoder<'a, I>,
        Self: Sized;
}

pub trait DecoderForIndex<'a, I: AsBytes + Debug + 'a> {
    fn decoder_for_index<D>(v: i128) -> Result<fn(I) -> IResult<I, Self>, DecodingError<I>>
    where
        D: Decoder<'a, I>,
        Self: Sized;
}

pub trait EncoderForIndex<T, O: Extend<T> + Debug + 'static> {
    fn encoder_for_index<E>(
        index: i128,
    ) -> Result<fn(&Self, O) -> Result<O, EncodingError>, EncodingError>
    where
        E: Encoder<T, O>,
        Self: Sized;
}

pub trait HasOptionalField {
    fn has_optional_field(&self, index: usize) -> bool;
}

pub trait DecoderForKey<'a, I: AsBytes + Debug + 'a, T> {
    fn decoder_for_key<D>(key: T) -> Result<fn(I) -> IResult<I, Self>, DecodingError<I>>
    where
        D: Decoder<'a, I>,
        T: PartialEq,
        Self: Sized;
}

pub trait Decoder<'a, I: AsBytes + Debug + 'a> {
    fn decode_open_type(input: I) -> IResult<I, Vec<u8>>;
    fn decode_integer<O>(
        integer: Integer,
    ) -> Result<Box<dyn Fn(I) -> IResult<I, O>>, DecodingError<I>>
    where
        O: num::Integer + num::FromPrimitive + Copy;
    fn decode_enumerated<O: TryFrom<i128>>(
        enumerated: Enumerated,
    ) -> Result<Box<dyn Fn(I) -> IResult<I, O>>, DecodingError<I>>;
    fn decode_choice<O: DecoderForIndex<'a, I>>(
        choice: Choice,
    ) -> Result<Box<dyn Fn(I) -> IResult<I, O>>, DecodingError<I>>;
    fn decode_null<N: Default>(input: I) -> IResult<I, N>;
    fn decode_boolean(input: I) -> IResult<I, bool>;
    fn decode_bit_string(
        bit_string: BitString,
    ) -> Result<Box<dyn Fn(I) -> IResult<I, Vec<bool>>>, DecodingError<I>>;
    fn decode_octet_string(
        octet_string: OctetString,
    ) -> Result<Box<dyn Fn(I) -> IResult<I, Vec<u8>>>, DecodingError<I>>;
    fn decode_character_string(
        char_string: CharacterString,
    ) -> Result<Box<dyn Fn(I) -> IResult<I, String>>, DecodingError<I>>;
    fn decode_sequence<T: DecodeMember<'a, I> + Default>(
        sequence: SequenceOrSet,
    ) -> Result<Box<dyn Fn(I) -> IResult<I, T>>, DecodingError<I>>;
    fn decode_sequence_of<T: Decode<'a, I> + 'a + Sized>(
        sequence_of: SequenceOf,
        member_decoder: fn(I) -> IResult<I, T>,
    ) -> Result<Box<dyn Fn(I) -> IResult<I, Vec<T>> + 'a>, DecodingError<I>>;
    fn decode_unknown_extension(input: I) -> IResult<I, Vec<u8>>;
}

pub trait Encoder<T, O: Extend<T> + Debug + 'static> {
    fn encode_integer<I>(
        integer: Integer,
    ) -> Result<Box<dyn Fn(I, O) -> Result<O, EncodingError>>, EncodingError>
    where
        I: num::Integer + num::ToPrimitive + num::FromPrimitive + Copy;
    fn encode_boolean(value: bool, output: O) -> Result<O, EncodingError>;
    fn encode_null(output: O) -> Result<O, EncodingError>;
    fn encode_bit_string(
        bit_string: BitString,
    ) -> Result<Box<dyn Fn(Vec<bool>, O) -> Result<O, EncodingError>>, EncodingError>;
    fn encode_octet_string(
        octet_string: OctetString,
    ) -> Result<Box<dyn Fn(&[u8], O) -> Result<O, EncodingError>>, EncodingError>;
    fn encode_character_string(
        character_string: CharacterString,
    ) -> Result<Box<dyn Fn(&str, O) -> Result<O, EncodingError>>, EncodingError>;
    fn encode_sequence<S: EncoderForIndex<T, O> + Debug + HasOptionalField>(
        sequence: SequenceOrSet,
    ) -> Result<Box<dyn Fn(S, O) -> Result<O, EncodingError>>, EncodingError>;
    fn encode_enumerated<E: Encode<T, O> + Debug>(
        enumerated: Enumerated,
    ) -> Result<Box<dyn Fn(E, O) -> Result<O, EncodingError>>, EncodingError>;
    fn encode_choice<C: EncoderForIndex<T, O> + Debug>(
        choice: Choice,
    ) -> Result<Box<dyn Fn(C, O) -> Result<O, EncodingError>>, EncodingError>;
    fn encode_sequence_of<M: Encode<T, O>>(
        sequence_of: SequenceOf,
    ) -> Result<Box<dyn Fn(Vec<M>, O) -> Result<O, EncodingError>>, EncodingError>;
    fn encode_open_type(input: &[u8], output: O) -> Result<O, EncodingError>;
}

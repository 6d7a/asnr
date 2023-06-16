use alloc::{string::String, vec::Vec};

use crate::{Decoder, DecoderForIndex};

use super::Uper;

impl Decoder for Uper {
    fn decode_integer<'a, O: num::Integer + num::FromPrimitive>(
        &self,
        _integer: asnr_grammar::types::AsnInteger,
    ) -> fn(&'a [u8]) -> nom::IResult<&'a [u8], O> {
        move |input: &[u8]| {
            nom::bytes::complete::take(1_u8)(input)
                .map(|(r, m)| (r, O::from_i128(m[0] as i128).unwrap()))
        }
    }

    fn decode_enumerated<'a, O: TryFrom<i128>>(
        &self,
        _enumerated: asnr_grammar::types::AsnEnumerated,
    ) -> fn(&'a [u8]) -> nom::IResult<&'a [u8], O> {
        todo!()
    }

    fn decode_boolean<'a>(&self, _input: &'a [u8]) -> nom::IResult<&'a [u8], bool> {
        todo!()
    }

    fn decode_bit_string<'a>(
        &self,
        _bit_string: asnr_grammar::types::AsnBitString,
    ) -> fn(&'a [u8]) -> nom::IResult<&'a [u8], Vec<bool>> {
        todo!()
    }

    fn decode_character_string<'a>(
        &self,
        _bit_string: asnr_grammar::types::AsnCharacterString,
    ) -> fn(&'a [u8]) -> nom::IResult<&'a [u8], String> {
        todo!()
    }

    fn decode_sequence<'a, T: crate::DecodeMember>(
        &self,
        _sequence: asnr_grammar::types::AsnSequence,
    ) -> fn(&'a [u8]) -> nom::IResult<&'a [u8], T> {
        todo!()
    }

    fn decode_sequence_of<'a, T: crate::Decode>(
        &self,
        _sequence_of: asnr_grammar::types::AsnSequenceOf,
        _member_decoder: impl FnMut(&Self, &'a [u8]) -> nom::IResult<&'a [u8], T>,
    ) -> fn(&'a [u8]) -> nom::IResult<&'a [u8], Vec<T>> {
        todo!()
    }

    fn decode_unknown_extension<'a>(&self, _input: &'a [u8]) -> nom::IResult<&'a [u8], &'a [u8]> {
        todo!()
    }

    fn decode_choice<'a, O: DecoderForIndex>(
      &self,
      _choice: asnr_grammar::types::AsnChoice,
      ) -> fn(&'a [u8]) -> nom::IResult<&'a [u8], O> {
        todo!()
    }
}

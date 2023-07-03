use alloc::{string::String, vec::Vec};
use bitvec::{prelude::Msb0, vec::BitVec};
use bitvec_nom::BSlice;
use nom::{
    bytes::complete::take,
    combinator::{map, map_res},
    IResult,
};
use num::{FromPrimitive, Integer};

use crate::{uper::per_visible::PerVisibleIntegerConstraints, Decoder, DecoderForIndex};

use super::Uper;

type BitIn<'a> = BSlice<'a, u8, Msb0>;
type BitOut = BitVec<u8, Msb0>;

enum LengthDeterminant {
    Content(usize),
    ContentFragment(usize),
}

impl<'a> Decoder<BitIn<'a>> for Uper {
    fn decode_open_type(&self, input: BitIn<'a>) -> IResult<BitIn<'a>, Vec<u8>> {
        todo!()
    }

    fn decode_integer<O: num::Integer + num::FromPrimitive>(
        &self,
        integer: asnr_grammar::types::Integer,
    ) -> fn(BitIn<'a>) -> IResult<BitIn<'a>, O> {
        todo!()
    }

    fn decode_enumerated<O: TryFrom<i128>>(
        &self,
        enumerated: asnr_grammar::types::Enumerated,
    ) -> fn(BitIn<'a>) -> IResult<BitIn<'a>, O> {
        todo!()
    }

    fn decode_choice<O: DecoderForIndex<BitIn<'a>>>(&self, choice: asnr_grammar::types::Choice) -> fn(BitIn<'a>) -> IResult<BitIn<'a>, O> {
        todo!()
    }

    fn decode_null<N>(&self, input: BitIn<'a>) -> IResult<BitIn<'a>, N> {
        todo!()
    }

    fn decode_boolean(&self, input: BitIn<'a>) -> IResult<BitIn<'a>, bool> {
        todo!()
    }

    fn decode_bit_string(&self, bit_string: asnr_grammar::types::BitString) -> fn(BitIn<'a>) -> IResult<BitIn<'a>, Vec<bool>> {
        todo!()
    }

    fn decode_character_string(
        &self,
        char_string: asnr_grammar::types::CharacterString,
    ) -> fn(BitIn<'a>) -> IResult<BitIn<'a>, String> {
        todo!()
    }

    fn decode_sequence<T: crate::DecodeMember<BitIn<'a>>>(&self, sequence: asnr_grammar::types::Sequence) -> fn(BitIn<'a>) -> IResult<BitIn<'a>, T> {
        todo!()
    }

    fn decode_sequence_of<T: crate::Decode<BitIn<'a>>>(
        &self,
        sequence_of: asnr_grammar::types::SequenceOf,
        member_decoder: impl FnMut(&Self, BitIn<'a>) -> IResult<BitIn<'a>, T>,
    ) -> fn(BitIn<'a>) -> IResult<BitIn<'a>, Vec<T>> {
        todo!()
    }

    fn decode_unknown_extension(&self, input: BitIn<'a>) -> IResult<BitIn<'a>, Vec<u8>> {
        todo!()
    }
}

fn decode_length_determinant(input: BitIn) -> IResult<BitIn, LengthDeterminant> {
    let (input, longer_than_127) = read_bit(input)?;
    if longer_than_127 {
        let (input, longer_than_15999) = read_bit(input)?;
        if longer_than_15999 {
            let (input, size_factor) = read_int::<usize>(6)(input)?;
            //TODO: Check that size factor is in range 1..=4
            return Ok((
                input,
                LengthDeterminant::ContentFragment(16000 * size_factor),
            ));
        }
        return map(read_int::<usize>(14), |i| LengthDeterminant::Content(i))(input);
    }
    map(read_int::<usize>(7), |i| LengthDeterminant::Content(i))(input)
}

fn read_bit(input: BitIn) -> IResult<BitIn, bool> {
    map(take(1u8), |is_true: BitIn| match is_true.first() {
        Some(bit) => *bit.as_ref(),
        None => unreachable!(),
    })(input)
}

fn read_int<O>(bits: usize) -> impl FnMut(BitIn) -> IResult<BitIn, O>
where
    O: Integer + FromPrimitive,
{
    move |input| {
        map_res(take(bits), |int_bits: BitIn| {
            O::from_u64(bits_to_int(int_bits)).ok_or("err")
        })(input)
    }
}

fn bits_to_int(input: BitIn) -> u64 {
    let mut int = 0;
    for bit in input.0 {
        int = int << 1;
        if bit == true {
            int += 1;
        }
    }
    return int;
}

#[cfg(test)]
mod tests {

    use bitvec::prelude::*;
    use bitvec_nom::BSlice;

    use crate::uper::decoder::bits_to_int;

    #[test]
    fn bit_to_int() {
        let bits = bits![u8, Msb0; 1, 0, 1];
        assert_eq!(5u64, bits_to_int(BSlice::from(bits)))
    }
}

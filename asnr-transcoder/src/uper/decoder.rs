use alloc::{borrow::ToOwned, boxed::Box, string::String, vec::Vec};
use bitvec::{prelude::Msb0, vec::BitVec};
use bitvec_nom::BSlice;
use nom::{
    bytes::complete::take,
    combinator::{map, map_res},
    error::Error,
    multi::{length_data, length_value},
    AsBytes, IResult,
};
use num::{FromPrimitive, Integer};

use crate::{
    error::{DecodingError, DecodingErrorType},
    uper::per_visible::PerVisibleIntegerConstraints,
    Decoder, DecoderForIndex,
};

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

    fn decode_integer<O>(
        &self,
        integer: asnr_grammar::types::Integer,
    ) -> Result<Box<dyn FnMut(BitIn<'a>) -> IResult<BitIn<'a>, O>>, DecodingError>
    where
        O: num::Integer + num::FromPrimitive + Copy,
    {
        let mut constraints = PerVisibleIntegerConstraints::default();
        for c in integer.constraints {
            constraints += c.try_into()?
        }
        if constraints.is_extensible() {
            if let Some(bit_length) = constraints.bit_size() {
                Ok(Box::new(move |input: BitIn<'a>| -> IResult<BitIn<'a>, O> {
                    let (input, is_extended) = read_bit(input)?;
                    if is_extended {
                        decode_varlength_integer(input, None)
                    } else {
                        map(read_int(bit_length), |i: O| i + constraints.min().unwrap())(input)
                    }
                }))
            } else {
                Ok(Box::new(move |input: BitIn<'a>| -> IResult<BitIn<'a>, O> {
                    let (input, is_extended) = read_bit(input)?;
                    if is_extended {
                        decode_varlength_integer(input, None)
                    } else {
                        decode_varlength_integer(input, constraints.min())
                    }
                }))
            }
        } else {
            if let Some(bit_length) = constraints.bit_size() {
                Ok(Box::new(move |input: BitIn<'a>| -> IResult<BitIn<'a>, O> {
                    map(read_int(bit_length), |i: O| i + constraints.min().unwrap())(input)
                }))
            } else {
                Ok(Box::new(move |input: BitIn<'a>| -> IResult<BitIn<'a>, O> {
                    decode_varlength_integer(input, constraints.min())
                }))
            }
        }
    }

    fn decode_enumerated<O: TryFrom<i128>>(
        &self,
        enumerated: asnr_grammar::types::Enumerated,
    ) -> fn(BitIn<'a>) -> IResult<BitIn<'a>, O> {
        todo!()
    }

    fn decode_choice<O: DecoderForIndex<BitIn<'a>>>(
        &self,
        choice: asnr_grammar::types::Choice,
    ) -> fn(BitIn<'a>) -> IResult<BitIn<'a>, O> {
        todo!()
    }

    fn decode_null<N>(&self, input: BitIn<'a>) -> IResult<BitIn<'a>, N> {
        todo!()
    }

    fn decode_boolean(&self, input: BitIn<'a>) -> IResult<BitIn<'a>, bool> {
        todo!()
    }

    fn decode_bit_string(
        &self,
        bit_string: asnr_grammar::types::BitString,
    ) -> fn(BitIn<'a>) -> IResult<BitIn<'a>, Vec<bool>> {
        todo!()
    }

    fn decode_character_string(
        &self,
        char_string: asnr_grammar::types::CharacterString,
    ) -> fn(BitIn<'a>) -> IResult<BitIn<'a>, String> {
        todo!()
    }

    fn decode_sequence<T: crate::DecodeMember<BitIn<'a>>>(
        &self,
        sequence: asnr_grammar::types::Sequence,
    ) -> fn(BitIn<'a>) -> IResult<BitIn<'a>, T> {
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

fn decode_varlength_integer<O: num::Integer + num::FromPrimitive + Copy>(
    input: BitIn,
    min: Option<O>,
) -> IResult<BitIn, O> {
    let (input, length_det) = decode_length_determinant(input)?;
    match length_det {
        LengthDeterminant::Content(size) => {
            map_res(take(8 * size), |buffer: BitIn| match (min, size) {
                (Some(m), s) if m >= O::from_u8(0).unwrap() => {
                    integer_from_bits::<O>(buffer, s, false).map(|i| i + m)
                }
                (Some(m), s) if m < O::from_u8(0).unwrap() => {
                    integer_from_bits::<O>(buffer, s, true).map(|i| i + m)
                }
                (_, s) => integer_from_bits::<O>(buffer, s, true),
            })(input)
        }
        LengthDeterminant::ContentFragment(_size) => Err(nom::Err::Error(Error {
            input: input,
            code: nom::error::ErrorKind::Digit,
        })),
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

fn integer_from_bits<I: num::Integer + num::FromPrimitive>(
    input: BitIn,
    byte_length: usize,
    signed: bool,
) -> Result<I, DecodingError> {
    let error = DecodingError {
        details: "Error parsing integer buffer.".into(),
        kind: DecodingErrorType::GenericParsingError,
    };
    if signed {
        match byte_length {
            s if s == 1 => match input.as_bytes().try_into() {
                Ok(int) => I::from_i8(i8::from_be_bytes(int)).ok_or(error),
                Err(_e) => Err(error),
            },
            s if s <= 2 => match input.as_bytes().try_into() {
                Ok(int) => I::from_i16(i16::from_be_bytes(int)).ok_or(error),
                Err(_e) => Err(error),
            },
            s if s <= 4 => match input.as_bytes().try_into() {
                Ok(int) => I::from_i32(i32::from_be_bytes(int)).ok_or(error),
                Err(_e) => Err(error),
            },
            s if s <= 8 => match input.as_bytes().try_into() {
                Ok(int) => I::from_i64(i64::from_be_bytes(int)).ok_or(error),
                Err(_e) => Err(error),
            },
            s if s <= 16 => match input.as_bytes().try_into() {
                Ok(int) => I::from_i128(i128::from_be_bytes(int)).ok_or(error),
                Err(_e) => Err(error),
            },
            _ => Err(DecodingError {
                details: "ASNR currently does not support integers longer than 128 bits.".into(),
                kind: DecodingErrorType::Unsupported,
            }),
        }
    } else {
        match byte_length {
            s if s == 1 => match input.as_bytes().try_into() {
                Ok(int) => I::from_u8(u8::from_be_bytes(int)).ok_or(error),
                Err(_e) => Err(error),
            },
            s if s <= 2 => match input.as_bytes().try_into() {
                Ok(int) => I::from_u16(u16::from_be_bytes(int)).ok_or(error),
                Err(_e) => Err(error),
            },
            s if s <= 4 => match input.as_bytes().try_into() {
                Ok(int) => I::from_u32(u32::from_be_bytes(int)).ok_or(error),
                Err(_e) => Err(error),
            },
            s if s <= 8 => match input.as_bytes().try_into() {
                Ok(int) => I::from_u64(u64::from_be_bytes(int)).ok_or(error),
                Err(_e) => Err(error),
            },
            s if s <= 16 => match input.as_bytes().try_into() {
                Ok(int) => I::from_u128(u128::from_be_bytes(int)).ok_or(error),
                Err(_e) => Err(error),
            },
            _ => Err(DecodingError {
                details: "ASNR currently does not support integers longer than 128 bits.".into(),
                kind: DecodingErrorType::Unsupported,
            }),
        }
    }
}

#[cfg(test)]
mod tests {

    use alloc::vec;
    use bitvec::prelude::*;
    use bitvec_nom::BSlice;

    use crate::uper::decoder::*;
    use asnr_grammar::{constraints::*, types::Integer, *};

    #[test]
    fn bit_to_int() {
        let bits = bits![u8, Msb0; 1, 0, 1];
        assert_eq!(5u64, bits_to_int(BSlice::from(bits)))
    }

    #[test]
    fn decodes_varlength_integer() {
        assert_eq!(
            decode_varlength_integer::<i128>(
                BSlice::from(bits![u8, Msb0; 0,0,0,0,0,0,1,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0]),
                None
            )
            .unwrap()
            .1,
            4096
        );
        assert_eq!(
            decode_varlength_integer::<i128>(
                BSlice::from(bits![u8, Msb0; 0,0,0,0,0,0,0,1,0,1,1,1,1,1,1,1]),
                None
            )
            .unwrap()
            .1,
            127
        );
        assert_eq!(
            decode_varlength_integer::<i128>(
                BSlice::from(bits![u8, Msb0; 0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0]),
                None
            )
            .unwrap()
            .1,
            -128
        );
        assert_eq!(
            decode_varlength_integer::<i128>(
                BSlice::from(bits![u8, Msb0; 0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0]),
                None
            )
            .unwrap()
            .1,
            128
        );
        assert_eq!(
            decode_varlength_integer::<i128>(
                BSlice::from(bits![u8, Msb0; 0,0,0,0,0,0,1,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,1]),
                Some(-1)
            )
            .unwrap()
            .1,
            4096
        );
        assert_eq!(
            decode_varlength_integer::<i128>(
                BSlice::from(bits![u8, Msb0; 0,0,0,0,0,0,0,1,0,1,1,1,1,1,1,0]),
                Some(1)
            )
            .unwrap()
            .1,
            127
        );
        assert_eq!(
            decode_varlength_integer::<i128>(
                BSlice::from(bits![u8, Msb0; 0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0]),
                Some(0)
            )
            .unwrap()
            .1,
            128
        );
    }

    #[test]
    fn decodes_constrained_int() {
        let uper = Uper {};
        let mut decoder_1 = uper
            .decode_integer::<i128>(Integer {
                distinguished_values: None,
                constraints: vec![Constraint::SubtypeConstraint(ElementSet {
                    set: ElementOrSetOperation::Element(SubtypeElement::ValueRange {
                        min: Some(ASN1Value::Integer(3)),
                        max: Some(ASN1Value::Integer(6)),
                        extensible: false,
                    }),
                    extensible: false,
                })],
            })
            .unwrap();
        assert_eq!(decoder_1(BSlice::from(bits![u8, Msb0; 0,0])).unwrap().1, 3);
        assert_eq!(decoder_1(BSlice::from(bits![u8, Msb0; 0,1])).unwrap().1, 4);
        assert_eq!(decoder_1(BSlice::from(bits![u8, Msb0; 1,0])).unwrap().1, 5);
        assert_eq!(decoder_1(BSlice::from(bits![u8, Msb0; 1,1])).unwrap().1, 6);
    }
}

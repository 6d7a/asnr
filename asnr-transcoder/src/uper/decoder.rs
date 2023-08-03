use alloc::{boxed::Box, string::String, vec, vec::Vec};
use asnr_grammar::types::Sequence;
use bitvec::{bits, bitvec, prelude::Msb0, vec::BitVec};
use bitvec_nom::BSlice;
use nom::{bytes::complete::take, combinator::map, error::Error, AsBytes};
use num::{FromPrimitive, Integer};

use crate::{
    error::{DecodingError, DecodingErrorType},
    uper::per_visible::PerVisibleRangeConstraints,
    Decode, DecodeMember, Decoder, DecoderForIndex, IResult,
};

use super::{per_visible::PerVisibleAlphabetConstraints, AsBytesDummy, BitIn, Uper};

enum LengthDeterminant {
    Content(usize),
    ContentFragment(usize),
}

impl LengthDeterminant {
    pub fn _collect_value<'a>(
        &self,
        input: BitIn<'a>,
        factor: usize,
    ) -> IResult<BitIn<'a>, BitVec<u8, Msb0>> {
        Self::_recursive_collect(self, input, factor, bitvec![u8, Msb0;])
    }

    fn _recursive_collect<'a>(
        &self,
        input: BitIn<'a>,
        factor: usize,
        mut temp: BitVec<u8, Msb0>,
    ) -> IResult<BitIn<'a>, BitVec<u8, Msb0>> {
        match self {
            LengthDeterminant::Content(c) => {
                let input = map(take(c * factor), |res: BitIn| {
                    temp.extend_from_bitslice(res.0)
                })(input)?
                .0;
                Ok((input, temp))
            }
            LengthDeterminant::ContentFragment(f) => {
                let input = map(take(f * factor), |res: BitIn| {
                    temp.extend_from_bitslice(res.0)
                })(input)?
                .0;
                let (input, length_det) = decode_length_determinant(input)?;
                length_det._recursive_collect(input, factor, temp)
            }
        }
    }
}

impl<'a> Decoder<'a, BitIn<'a>> for Uper {
    fn decode_open_type(input: BitIn<'a>) -> IResult<BitIn<'a>, Vec<u8>> {
        let (input, ext_length) = decode_varlength_integer::<usize>(input, Some(0))?;
        Ok(map(take(8 * ext_length), |buffer: BitIn| {
            buffer.as_bytes().to_vec()
        })(input)?)
    }

    fn decode_integer<O>(
        integer: asnr_grammar::types::Integer,
    ) -> Result<Box<dyn Fn(BitIn<'a>) -> IResult<BitIn<'a>, O>>, DecodingError<BitIn<'a>>>
    where
        O: num::Integer + num::FromPrimitive + Copy,
    {
        let mut constraints = PerVisibleRangeConstraints::default();
        for c in &integer.constraints {
            constraints += c
                .try_into()
                .map_err(|e: DecodingError<[u8; 0]>| DecodingError {
                    input: None,
                    details: e.details,
                    kind: e.kind,
                })?
        }
        if constraints.is_extensible() {
            if constraints.bit_length().is_some() {
                Ok(Box::new(move |input: BitIn<'a>| -> IResult<BitIn<'a>, O> {
                    let (input, is_extended) = read_bit(input)?;
                    if is_extended {
                        decode_varlength_integer(input, None)
                    } else {
                        decode_unextensible_int(&constraints, input)
                    }
                }))
            } else {
                Ok(Box::new(move |input: BitIn<'a>| -> IResult<BitIn<'a>, O> {
                    let (input, is_extended) = read_bit(input)?;
                    decode_varlength_integer(
                        input,
                        if is_extended { None } else { constraints.min() },
                    )
                }))
            }
        } else {
            Ok(Box::new(move |input: BitIn<'a>| -> IResult<BitIn<'a>, O> {
                decode_unextensible_int(&constraints, input)
            }))
        }
    }

    fn decode_enumerated<O: TryFrom<i128>>(
        enumerated: asnr_grammar::types::Enumerated,
    ) -> Result<Box<dyn Fn(BitIn) -> IResult<BitIn, O>>, DecodingError<BitIn<'a>>> {
        let mut constraints = PerVisibleRangeConstraints::from(&enumerated);
        for c in &enumerated.constraints {
            constraints += c
                .try_into()
                .map_err(|e: DecodingError<[u8; 0]>| DecodingError {
                    input: None,
                    details: e.details,
                    kind: e.kind,
                })?
        }
        if constraints.is_extensible() {
            if let Some(bit_length) = constraints.bit_length() {
                Ok(Box::new(move |input: BitIn| -> IResult<BitIn, O> {
                    let (input, is_extended) = read_bit(input)?;
                    if is_extended {
                        let (input, i) = decode_normally_small_number(input)?;
                        let index = O::try_from((i + enumerated.extensible.unwrap()) as i128)
                            .map_err(|_| DecodingError {
                                details: "Failed to convert index to generic integer type.".into(),
                                input: Some(input),
                                kind: DecodingErrorType::GenericParsingError,
                            })?;
                        Ok((input, index))
                    } else {
                        decode_enum_index_into(bit_length, input)
                    }
                }))
            } else {
                unreachable!()
            }
        } else {
            if let Some(bit_length) = constraints.bit_length() {
                Ok(Box::new(move |input: BitIn| {
                    decode_enum_index_into(bit_length, input)
                }))
            } else {
                unreachable!()
            }
        }
    }

    fn decode_choice<O: DecoderForIndex<'a, BitIn<'a>>>(
        choice: asnr_grammar::types::Choice,
    ) -> Result<Box<dyn Fn(BitIn<'a>) -> IResult<BitIn<'a>, O>>, DecodingError<BitIn<'a>>> {
        let mut constraints = PerVisibleRangeConstraints::from(&choice);
        for c in &choice.constraints {
            constraints += c
                .try_into()
                .map_err(|e: DecodingError<[u8; 0]>| DecodingError {
                    input: None,
                    details: e.details,
                    kind: e.kind,
                })?
        }
        if constraints.is_extensible() {
            if let Some(bit_length) = constraints.bit_length() {
                Ok(Box::new(move |input: BitIn| -> IResult<BitIn, O> {
                    let (mut input, is_extended) = read_bit(input)?;
                    if is_extended {
                        let mut index;
                        (input, index) = decode_normally_small_number(input)?;
                        index = index + choice.extensible.unwrap();
                        let (mut inner_input, ext_length) =
                            decode_varlength_integer::<usize>(input, Some(0))?;
                        (input, inner_input) = take(8 * ext_length)(inner_input)?;
                        O::decoder_for_index::<Uper>(index as i128).map_err(|_| {
                            nom::Err::Error(Error {
                                input,
                                code: nom::error::ErrorKind::OneOf,
                            })
                        })?(inner_input)
                    } else {
                        decode_choice_index_into(bit_length, input)
                    }
                }))
            } else {
                unreachable!()
            }
        } else {
            if let Some(bit_length) = constraints.bit_length() {
                Ok(Box::new(move |input: BitIn| {
                    decode_choice_index_into(bit_length, input)
                }))
            } else {
                unreachable!()
            }
        }
    }

    fn decode_null<N: Default>(input: BitIn<'a>) -> IResult<BitIn<'a>, N> {
        Ok((input, N::default()))
    }

    fn decode_boolean(input: BitIn<'a>) -> IResult<BitIn<'a>, bool> {
        read_bit(input)
    }

    fn decode_bit_string(
        bit_string: asnr_grammar::types::BitString,
    ) -> Result<Box<dyn Fn(BitIn<'a>) -> IResult<BitIn<'a>, Vec<bool>>>, DecodingError<BitIn<'a>>>
    {
        let mut constraints = PerVisibleRangeConstraints::default_unsigned();
        for c in &bit_string.constraints {
            constraints += c
                .try_into()
                .map_err(|e: DecodingError<AsBytesDummy>| DecodingError {
                    input: None,
                    details: e.details,
                    kind: e.kind,
                })?
        }
        if constraints.is_extensible() {
            Ok(Box::new(
                move |input: BitIn<'a>| -> IResult<BitIn<'a>, Vec<bool>> {
                    let (input, is_extended) = read_bit(input)?;
                    let (input, length_det) = size_length_det(is_extended, &constraints, input)?;
                    n_times(input, read_bit, length_det)
                },
            ))
        } else {
            Ok(Box::new(move |input| {
                let (input, length_det) = size_length_det(false, &constraints, input)?;
                n_times(input, read_bit, length_det)
            }))
        }
    }

    fn decode_character_string(
        char_string: asnr_grammar::types::CharacterString,
    ) -> Result<Box<dyn Fn(BitIn<'a>) -> IResult<BitIn<'a>, String>>, DecodingError<BitIn<'a>>>
    {
        let mut range_constraints = PerVisibleRangeConstraints::default_unsigned();
        let mut permitted_alphabet = PerVisibleAlphabetConstraints::default_for(char_string.r#type);
        for c in &char_string.constraints {
            range_constraints +=
                c.try_into()
                    .map_err(|e: DecodingError<AsBytesDummy>| DecodingError {
                        input: None,
                        details: e.details,
                        kind: e.kind,
                    })?;
            PerVisibleAlphabetConstraints::try_new(c, char_string.r#type)?
                .map(|mut p| permitted_alphabet += &mut p);
        }
        permitted_alphabet.finalize();
        if range_constraints.is_extensible() {
            Ok(Box::new(
                move |input: BitIn<'a>| -> IResult<BitIn<'a>, String> {
                    let (input, is_extended) = if permitted_alphabet.is_known_multiplier_string() {
                      read_bit(input)?
                    } else {
                      (input, true)
                    };
                    let (input, length_det) =
                        size_length_det(is_extended, &range_constraints, input)?;
                    decode_sized_string(&permitted_alphabet, length_det, input)
                },
            ))
        } else {
            Ok(Box::new(move |input| {
                let (input, length_det) = size_length_det(false, &range_constraints, input)?;
                decode_sized_string(&permitted_alphabet, length_det, input)
            }))
        }
    }

    fn decode_sequence<T: DecodeMember<'a, BitIn<'a>> + Default>(
        sequence: asnr_grammar::types::Sequence,
    ) -> Result<Box<dyn Fn(BitIn<'a>) -> IResult<BitIn, T>>, DecodingError<BitIn<'a>>> {
        if let Some(extension_index) = sequence.extensible {
            Ok(Box::new(move |input| {
                let (input, is_extended) = read_bit(input)?;
                let (mut input, mut instance) = decode_unextended_sequence::<T>(&sequence, input)?;
                input = if is_extended {
                    let (mut input, length) = decode_normally_small_number(input)?;
                    let mut extension_presence = vec![];
                    for _ in 0..length {
                        let parsed = read_bit(input)?;
                        input = parsed.0;
                        extension_presence.push(parsed.1);
                    }
                    for (index, present) in extension_presence.iter().enumerate() {
                        if *present {
                            let (mut inner_input, length_det) = decode_length_determinant(input)?;
                            match length_det {
                                LengthDeterminant::Content(ext_length) => {
                                    (input, inner_input) = take(8 * ext_length)(inner_input)?;
                                    let _ = instance.decode_member_at_index::<Uper>(
                                        index + extension_index,
                                        inner_input,
                                    )?;
                                }
                                LengthDeterminant::ContentFragment(_) => {
                                    todo!()
                                }
                            }
                        }
                    }
                    input
                } else {
                    input
                };
                Ok((input, instance))
            }))
        } else {
            Ok(Box::new(move |input| {
                decode_unextended_sequence(&sequence, input)
            }))
        }
    }

    fn decode_sequence_of<T: Decode<'a, BitIn<'a>> + 'a>(
        sequence_of: asnr_grammar::types::SequenceOf,
        member_decoder: fn(BitIn<'a>) -> IResult<BitIn<'a>, T>,
    ) -> Result<
        Box<dyn Fn(BitIn<'a>) -> IResult<BitIn<'a>, Vec<T>> + 'a>,
        DecodingError<BitIn<'a>>,
    > {
        let mut constraints = PerVisibleRangeConstraints::default();
        for c in &sequence_of.constraints {
            constraints += c
                .try_into()
                .map_err(|e: DecodingError<[u8; 0]>| DecodingError {
                    input: None,
                    details: e.details,
                    kind: e.kind,
                })?
        }
        constraints.as_unsigned_constraint();
        if constraints.is_extensible() {
            Ok(Box::new(
                move |input: BitIn<'a>| -> IResult<BitIn<'a>, Vec<T>> {
                    let (input, is_extended) = read_bit(input)?;
                    let (input, length_det) = size_length_det(is_extended, &constraints, input)?;
                    n_times(input, member_decoder, length_det)
                },
            ))
        } else {
            Ok(Box::new(
                move |input: BitIn<'a>| -> IResult<BitIn<'a>, Vec<T>> {
                    let (input, length_det) = decode_unextensible_int(&constraints, input)?;
                    n_times(input, member_decoder, length_det)
                },
            ))
        }
    }

    fn decode_unknown_extension(input: BitIn<'a>) -> IResult<BitIn<'a>, Vec<u8>> {
        Ok((
            BSlice::from(bits![static u8, Msb0;]),
            input.as_bytes().to_vec(),
        ))
    }
}

fn size_length_det<'a>(
    is_extended: bool,
    constraints: &PerVisibleRangeConstraints,
    input: BitIn<'a>,
) -> IResult<BitIn<'a>, usize> {
    if constraints
        .range_width()?
        .map(|w| (w <= 65536).then(|| w))
        .flatten()
        .is_some()
        && !is_extended
    {
        decode_unextensible_int::<usize>(&*constraints, input)
    } else {
        match decode_length_determinant(input)? {
            (input, LengthDeterminant::Content(len)) => Ok((input, len)),
            _ => Err(DecodingError {
                input: Some(input),
                details: "Size counts larger than 65536 items are not supported yet!".into(),
                kind: DecodingErrorType::Unsupported,
            }),
        }
    }
}

fn decode_unextended_sequence<'a, T: DecodeMember<'a, BitIn<'a>> + Default>(
    sequence: &Sequence,
    mut input: BitIn<'a>,
) -> IResult<BitIn<'a>, T> {
    let mut member_presence = vec![];
    for (_, m) in sequence
        .members
        .iter()
        .enumerate()
        .filter(|(i, _)| sequence.extensible.map_or(true, |x| i < &x))
    {
        if m.is_optional {
            let parsed = read_bit(input)?;
            input = parsed.0;
            member_presence.push(parsed.1);
        } else {
            member_presence.push(true)
        }
    }
    let mut instance = T::default();
    for (index, present) in member_presence.iter().enumerate() {
        if *present {
            input = instance.decode_member_at_index::<Uper>(index, input)?;
        }
    }
    Ok((input, instance))
}

fn decode_enum_index_into<'a, O: TryFrom<i128>>(
    bit_length: usize,
    input: BitIn<'a>,
) -> IResult<BitIn<'a>, O> {
    let (input, i) = read_int::<i128>(bit_length)(input)?;
    let index = O::try_from(i).map_err(|_| DecodingError {
        input: Some(input),
        details: "Failed to convert index to generic integer type.".into(),
        kind: DecodingErrorType::GenericParsingError,
    })?;
    Ok((input, index))
}

fn decode_choice_index_into<'a, O: DecoderForIndex<'a, BitIn<'a>>>(
    bit_length: usize,
    input: BitIn<'a>,
) -> IResult<BitIn<'a>, O> {
    let (input, index) = read_int::<i128>(bit_length)(input)?;
    O::decoder_for_index::<Uper>(index).map_err(|_| {
        nom::Err::Error(Error {
            input,
            code: nom::error::ErrorKind::OneOf,
        })
    })?(input)
}

fn decode_unextensible_int<'a, O>(
    constraints: &PerVisibleRangeConstraints,
    input: BitIn<'a>,
) -> IResult<BitIn<'a>, O>
where
    O: num::Integer + num::FromPrimitive + Copy,
{
    if let (Some(bit_length), Some(min)) = (constraints.bit_length(), constraints.min::<usize>()) {
        let (input, i) = read_int::<usize>(bit_length)(input)?;
        Ok((
            input,
            O::from_usize(i + min).ok_or(DecodingError {
                details: "Failed to wrap in original integer type.".into(),
                input: None,
                kind: DecodingErrorType::GenericParsingError,
            })?,
        ))
    } else {
        decode_varlength_integer(input, constraints.min())
    }
}

fn decode_sized_string<'a>(
    permitted_alphabet: &PerVisibleAlphabetConstraints,
    length_det: usize,
    input: BitIn<'a>,
) -> IResult<BitIn<'a>, String> {
    let bit_size = permitted_alphabet.bit_length();
    if bit_size == 0 {
        return decode_sized_string(
            &permitted_alphabet.fall_back_to_standard_charset(),
            length_det,
            input,
        );
    }
    let (input, mut buffer) = take(bit_size * length_det)(input)?;
    if permitted_alphabet.is_known_multiplier_string() {
        let mut char_vec = vec![];
        while let Ok((new_buffer, i)) = read_int::<usize>(bit_size)(buffer) {
            char_vec.push(permitted_alphabet.get_char_by_index(i).map_err(
                |e: DecodingError<[u8; 0]>| DecodingError {
                    input: None,
                    details: e.details,
                    kind: e.kind,
                },
            )?);
            buffer = new_buffer;
        }
        Ok((input, char_vec.into_iter().collect()))
    } else {
        Ok((
            input,
            String::from_utf8_lossy(buffer.as_bytes()).into_owned(),
        ))
    }
}

fn decode_varlength_integer<O: num::Integer + num::FromPrimitive + Copy>(
    input: BitIn,
    min: Option<O>,
) -> IResult<BitIn, O> {
    let (input, length_det) = decode_length_determinant(input)?;
    match length_det {
        LengthDeterminant::Content(size) => {
            let (input, buffer) = take(8 * size)(input)?;
            match (min, size) {
                (Some(m), s) => Ok((
                    input,
                    integer_from_bits::<O>(buffer, s, false).map(|i| i + m)?,
                )),
                (_, s) => Ok((input, integer_from_bits::<O>(buffer, s, true)?)),
            }
        }
        LengthDeterminant::ContentFragment(_size) => Err(DecodingError {
            input: Some(input),
            details: "Variable-length integers larger than 65536 bytes are not supported yet!"
                .into(),
            kind: DecodingErrorType::Unsupported,
        }),
    }
}

fn decode_normally_small_number(input: BitIn) -> IResult<BitIn, usize> {
    let (input, over_63) = read_bit(input)?;
    if over_63 {
        let (input, length_det) = decode_length_determinant(input)?;
        if let LengthDeterminant::Content(i) = length_det {
            Ok((input, i))
        } else {
            Err(DecodingError {
                input: Some(input),
                details: "Normally-small numbers larger than 63 are not supported yet!".into(),
                kind: DecodingErrorType::Unsupported,
            })
        }
    } else {
        read_int::<usize>(6)(input)
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
                LengthDeterminant::ContentFragment(16384 * size_factor),
            ));
        }
        let (input, size) = read_int::<usize>(14)(input)?;
        return Ok((input, LengthDeterminant::Content(size)));
    }
    let (input, size) = read_int::<usize>(7)(input)?;
    Ok((input, LengthDeterminant::Content(size)))
}

fn read_bit(input: BitIn) -> IResult<BitIn, bool> {
    let (input, bool_buffer) = take(1u8)(input)?;
    Ok((input, bool_buffer[0]))
}

fn read_int<O>(bits: usize) -> impl FnMut(BitIn) -> IResult<BitIn, O>
where
    O: Integer + FromPrimitive,
{
    move |input| {
        let (input, int_buffer) = take(bits)(input)?;
        Ok((
            input,
            O::from_u64(bits_to_int(int_buffer)).ok_or(DecodingError {
                details: "Failed to convert index to generic integer type.".into(),
                input: Some(input),
                kind: DecodingErrorType::GenericParsingError,
            })?,
        ))
    }
}

fn n_times<'a, T>(
    input: BitIn<'a>,
    parser: fn(BitIn<'a>) -> IResult<BitIn<'a>, T>,
    n: usize,
) -> IResult<BitIn<'a>, Vec<T>> {
    let mut vector = vec![];
    let mut input = input;
    for _ in 0..n {
        let (new_input, item) = parser(input)?;
        vector.push(item);
        input = new_input;
    }
    Ok((input, vector))
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
) -> Result<I, DecodingError<BitIn>> {
    let error = DecodingError {
        details: "Error parsing integer buffer.".into(),
        kind: DecodingErrorType::GenericParsingError,
        input: Some(input),
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
                input: Some(input),
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
                input: Some(input),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use asnr_compiler_derive::asn1_internal_tests;

    use alloc::{format, vec};
    use bitvec::prelude::*;
    use bitvec_nom::BSlice;
    use core::fmt::Debug;

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
        let mut decoder = Uper::decode_integer::<i128>(Integer {
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
        assert_eq!(
            decoder(BSlice::from(bits![static u8, Msb0; 0,0]))
                .unwrap()
                .1,
            3
        );
        assert_eq!(
            decoder(BSlice::from(bits![static u8, Msb0; 0,1]))
                .unwrap()
                .1,
            4
        );
        assert_eq!(
            decoder(BSlice::from(bits![static u8, Msb0; 1,0]))
                .unwrap()
                .1,
            5
        );
        assert_eq!(
            decoder(BSlice::from(bits![static u8, Msb0; 1,1]))
                .unwrap()
                .1,
            6
        );
        decoder = Uper::decode_integer::<i128>(Integer {
            distinguished_values: None,
            constraints: vec![Constraint::SubtypeConstraint(ElementSet {
                set: ElementOrSetOperation::Element(SubtypeElement::ValueRange {
                    min: Some(ASN1Value::Integer(4000)),
                    max: Some(ASN1Value::Integer(4254)),
                    extensible: false,
                }),
                extensible: false,
            })],
        })
        .unwrap();
        assert_eq!(
            decoder(BSlice::from(bits![static u8, Msb0; 0,0,0,0,0,0,1,0]))
                .unwrap()
                .1,
            4002
        );
        assert_eq!(
            decoder(BSlice::from(bits![static u8, Msb0; 0,0,0,0,0,1,1,0]))
                .unwrap()
                .1,
            4006
        );
        decoder = Uper::decode_integer::<i128>(Integer {
            distinguished_values: None,
            constraints: vec![Constraint::SubtypeConstraint(ElementSet {
                set: ElementOrSetOperation::Element(SubtypeElement::ValueRange {
                    min: Some(ASN1Value::Integer(1)),
                    max: Some(ASN1Value::Integer(65538)),
                    extensible: false,
                }),
                extensible: false,
            })],
        })
        .unwrap();
        assert_eq!(
            decoder(BSlice::from(
                bits![static u8, Msb0; 1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1]
            ))
            .unwrap()
            .1,
            65538
        );
    }

    #[test]
    fn decodes_enum() {
        asn1_internal_tests!("TestEnum ::= ENUMERATED { One, Two, Three }");

        let decoder = TestEnum::decoder::<Uper>().unwrap();
        assert_eq!(
            decoder(BSlice::from(bits![static u8, Msb0; 0,0]))
                .unwrap()
                .1,
            TestEnum::One
        );
        assert_eq!(
            decoder(BSlice::from(bits![static u8, Msb0; 0,1]))
                .unwrap()
                .1,
            TestEnum::Two
        );
        assert_eq!(
            decoder(BSlice::from(bits![static u8, Msb0; 1,0]))
                .unwrap()
                .1,
            TestEnum::Three
        );
    }

    #[test]
    fn decodes_extended_enum() {
        asn1_internal_tests!("TestEnumExt ::= ENUMERATED { One, ..., Three }");
        let decoder = TestEnumExt::decoder::<Uper>().unwrap();
        assert_eq!(
            decoder(BSlice::from(bits![static u8, Msb0; 0,0,0]))
                .unwrap()
                .1,
            TestEnumExt::One
        );
        assert_eq!(
            decoder(BSlice::from(bits![static u8, Msb0; 1,0,0,0,0,0,0,0]))
                .unwrap()
                .1,
            TestEnumExt::Three
        );
        assert_eq!(
            decoder(BSlice::from(bits![static u8, Msb0; 1,0,0,0,0,0,1,1]))
                .unwrap()
                .1,
            TestEnumExt::UnknownExtension
        );
    }

    #[test]
    fn decodes_unextended_sequence() {
        asn1_internal_tests!(
            "TestSequence ::= SEQUENCE
        {item-code INTEGER (0..254),
        item-name IA5String (SIZE (3..10))OPTIONAL,
        urgency ENUMERATED
        {normal, high} DEFAULT normal }"
        );

        assert_eq!(
            TestSequence::decode::<Uper>(BSlice::from(bits![static u8, Msb0; 
              1,0,
              0,0,0,1,1,0,1,1,
              0,1,1,
              1,0,1,0,0,1,1,
              1,0,0,1,0,0,0,
              1,0,0,0,1,0,1,
              1,0,1,0,0,1,0,
              1,0,1,0,0,1,0,
              1,0,1,1,0,0,1]))
            .unwrap()
            .1,
            TestSequence {
                item_code: TestSequence_item_code(27),
                item_name: TestSequence_item_name("SHERRY".into()),
                urgency: TestSequence_urgency::normal
            }
        );
    }

    #[test]
    fn decodes_extened_sequence() {
        asn1_internal_tests!(
            "TestSequence ::= SEQUENCE
      {item-code INTEGER (0..254),
      item-name IA5String (SIZE (3..10))OPTIONAL,
      ...,
      urgency ENUMERATED {normal, high} DEFAULT normal }"
        );

        assert_eq!(
            TestSequence::decode::<Uper>(BSlice::from(bits![static u8, Msb0; 
            1,
            1,
            0,0,0,1,1,0,1,1,
            0,1,1,
            1,0,1,0,0,1,1,
            1,0,0,1,0,0,0,
            1,0,0,0,1,0,1,
            1,0,1,0,0,1,0,
            1,0,1,0,0,1,0,
            1,0,1,1,0,0,1,
            0,0,0,0,0,0,1,
            1,
            0,0,0,0,0,0,0,1,
            1,0,0,0,0,0,0,0]))
            .unwrap()
            .1,
            TestSequence {
                item_code: TestSequence_item_code(27),
                item_name: TestSequence_item_name("SHERRY".into()),
                urgency: TestSequence_urgency::high,
                unknown_extension: vec![]
            }
        );
    }

    #[test]
    fn decodes_extended_choice() {
        asn1_internal_tests!(
            "Choice-example ::= CHOICE {normal NULL, high NULL, ..., medium NULL }"
        );
        assert_eq!(
            Choice_example::decode::<Uper>(BSlice::from(bits![static u8, Msb0; 0,0]))
                .unwrap()
                .1,
            Choice_example::normal(Choice_example_normal)
        );
        assert_eq!(
            Choice_example::decode::<Uper>(BSlice::from(
                bits![static u8, Msb0; 1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0]
            ))
            .unwrap()
            .1,
            Choice_example::medium(Choice_example_medium)
        )
    }

    #[test]
    fn decodes_fixed_size_bit_string() {
        asn1_internal_tests!("BitStringExample ::= BIT STRING (8)");
        assert_eq!(
            BitStringExample::decode::<Uper>(BSlice::from(bits![static u8, Msb0; 0,0,1,0,1,1,0,0]))
                .unwrap()
                .1,
            BitStringExample(vec![false, false, true, false, true, true, false, false])
        );
    }

    #[test]
    fn decodes_constrained_bit_string() {
        asn1_internal_tests!("BitStringExample ::= BIT STRING (4..8)");
        assert_eq!(
            BitStringExample::decode::<Uper>(BSlice::from(bits![static u8, Msb0; 0,0,1,0,0,1,0,1]))
                .unwrap()
                .1,
            BitStringExample(vec![false, false, true, false, true])
        );
    }

    #[test]
    fn decodes_semi_constrained_bit_string() {
        asn1_internal_tests!("BitStringExample ::= BIT STRING (4..MAX)");
        assert_eq!(
            BitStringExample::decode::<Uper>(BSlice::from(
                bits![static u8, Msb0; 0,0,0,0,0,1,0,1, 0,0,1,0,1]
            ))
            .unwrap()
            .1,
            BitStringExample(vec![false, false, true, false, true])
        );
    }

    #[test]
    fn decodes_deceptive_min_bit_string() {
        asn1_internal_tests!("BitStringExample ::= BIT STRING (MIN..3)");
        assert_eq!(
            BitStringExample::decode::<Uper>(BSlice::from(bits![static u8, Msb0; 0, 1, 1]))
                .unwrap()
                .1,
            BitStringExample(vec![true])
        );
    }

    #[test]
    fn decodes_unconstrained_bit_string() {
        asn1_internal_tests!("BitStringExample ::= BIT STRING");
        assert_eq!(
            BitStringExample::decode::<Uper>(BSlice::from(
                bits![static u8, Msb0; 0,0,0,0,0,1,0,1, 0,0,1,0,1]
            ))
            .unwrap()
            .1,
            BitStringExample(vec![false, false, true, false, true])
        );
    }

    #[test]
    fn decodes_extended_fixed_size_bit_string() {
        asn1_internal_tests!("BitStringExample ::= BIT STRING (8,...)");
        assert_eq!(
            BitStringExample::decode::<Uper>(BSlice::from(
                bits![static u8, Msb0; 0,0,0,1,0,1,1,0,0]
            ))
            .unwrap()
            .1,
            BitStringExample(vec![false, false, true, false, true, true, false, false])
        );
        assert_eq!(
            BitStringExample::decode::<Uper>(BSlice::from(
                bits![static u8, Msb0; 1, 0,0,0,0,0,0,0,1, 0]
            ))
            .unwrap()
            .1,
            BitStringExample(vec![false])
        );
    }

    #[test]
    fn decodes_extended_constrained_bit_string() {
        asn1_internal_tests!("BitStringExample ::= BIT STRING (4..8,...)");
        assert_eq!(
            BitStringExample::decode::<Uper>(BSlice::from(
                bits![static u8, Msb0; 0,0,0,1,0,0,1,0,1]
            ))
            .unwrap()
            .1,
            BitStringExample(vec![false, false, true, false, true])
        );
        assert_eq!(
            BitStringExample::decode::<Uper>(BSlice::from(
                bits![static u8, Msb0; 1,0,0,0,0,0,0,0,1,0]
            ))
            .unwrap()
            .1,
            BitStringExample(vec![false])
        );
    }

    #[test]
    fn decodes_extended_semi_constrained_bit_string() {
        asn1_internal_tests!("BitStringExample ::= BIT STRING (4..MAX,...)");
        assert_eq!(
            BitStringExample::decode::<Uper>(BSlice::from(
                bits![static u8, Msb0; 0, 0,0,0,0,0,1,0,1, 0,0,1,0,1]
            ))
            .unwrap()
            .1,
            BitStringExample(vec![false, false, true, false, true])
        );
        assert_eq!(
            BitStringExample::decode::<Uper>(BSlice::from(
                bits![static u8, Msb0; 1,0,0,0,0,0,0,0,1,0]
            ))
            .unwrap()
            .1,
            BitStringExample(vec![false])
        );
    }

    #[test]
    fn decodes_range_size_character_string() {
        asn1_internal_tests!("NumericStringExample ::= NumericString (SIZE(1..16))");

        assert_eq!(
            NumericStringExample::decode::<Uper>(BSlice::from(
                bits![static u8, Msb0; 0,0,1,1, 0,0,1,0, 1,0,0,0, 0,0,0,1, 0,0,1,1]
            ))
            .unwrap()
            .1,
            NumericStringExample("1702".into())
        );
    }

    #[test]
    fn decodes_range_size_character_string_with_alphabet_constraint() {
        asn1_internal_tests!(
            r#"NumericStringExample ::= NumericString (SIZE(1..16) INTERSECTION FROM("0123"))"#
        );

        assert_eq!(
            NumericStringExample::decode::<Uper>(BSlice::from(
                bits![static u8, Msb0; 0,0,1,1, 0,0, 1,0, 1,1, 0,1]
            ))
            .unwrap()
            .1,
            NumericStringExample("0231".into())
        );
    }

    #[test]
    fn decodes_fixed_size_character_string_with_alphabet_constraint() {
        asn1_internal_tests!(r#"Digit ::= UTF8String (SIZE(1) INTERSECTION FROM("0123456789"))"#);

        assert_eq!(
            Digit::decode::<Uper>(BSlice::from(bits![static u8, Msb0; 1,0,0,0]))
                .unwrap()
                .1,
            Digit("8".into())
        );
    }

    #[test]
    fn decodes_unconstrained_character_string_with_alphabet_constraint() {
        asn1_internal_tests!(r#"Greeting ::= UTF8String (FROM("HELO"))"#);

        assert_eq!(
            Greeting::decode::<Uper>(BSlice::from(
                bits![static u8, Msb0; 0,0,0,0,0,1,0,1, 0,1, 0,0, 1,0, 1,0, 1,1]
            ))
            .unwrap()
            .1,
            Greeting("HELLO".into())
        );
    }

    #[test]
    fn decodes_unconstrained_variable_size_character_string() {
        asn1_internal_tests!(r#"Greeting ::= GraphicString"#);
        assert_eq!(
            Greeting::decode::<Uper>(BSlice::from(
                bits![static u8, Msb0; 
                0,0,0,0,0,1,0,0, 
                1,1,1,1,0,0,0,0,
                1,0,0,1,1,1,1,1,
                1,0,0,1,0,0,1,0,
                1,0,0,1,0,1,1,0,
                ]
            ))
            .unwrap()
            .1,
            Greeting("💖".into())
        );
    }

    #[test]
    fn decodes_extended_variable_size_character_string() {
        asn1_internal_tests!(r#"Greeting ::= GraphicString (SIZE(1..29876,...))"#);
        assert_eq!(
            Greeting::decode::<Uper>(BSlice::from(
                bits![static u8, Msb0; 
                0,0,0,0,0,1,0,0, 
                1,1,1,1,0,0,0,0,
                1,0,0,1,1,1,1,1,
                1,0,0,1,0,0,1,0,
                1,0,0,1,0,1,1,0,
                ]
            ))
            .unwrap()
            .1,
            Greeting("💖".into())
        );
    }
}

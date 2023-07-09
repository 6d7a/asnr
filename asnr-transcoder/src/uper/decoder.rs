use alloc::{boxed::Box, string::String, vec, vec::Vec};
use bitvec::{prelude::Msb0, vec::BitVec};
use bitvec_nom::BSlice;
use nom::{
    bytes::complete::take,
    combinator::{map, map_res},
    error::Error,
    AsBytes, IResult,
};
use num::{FromPrimitive, Integer};

use crate::{
    error::{DecodingError, DecodingErrorType},
    uper::per_visible::PerVisibleIntegerConstraints,
    Decode, DecodeMember, Decoder, DecoderForIndex,
};

use super::Uper;

type BitIn<'a> = BSlice<'a, u8, Msb0>;
type BitOut = BitVec<u8, Msb0>;

enum LengthDeterminant {
    Content(usize),
    ContentFragment(usize),
}

impl<'a> Decoder<BitIn<'a>> for Uper {
    fn decode_open_type(input: BitIn<'a>) -> IResult<BitIn<'a>, Vec<u8>> {
        todo!()
    }

    fn decode_integer<O>(
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
        enumerated: asnr_grammar::types::Enumerated,
    ) -> Result<Box<dyn FnMut(BitIn) -> IResult<BitIn, O>>, DecodingError> {
        let mut constraints = PerVisibleIntegerConstraints::default();
        for c in enumerated.clone().constraints {
            constraints += c.try_into()?
        }
        constraints.as_enum_constraint(&enumerated);
        if constraints.is_extensible() {
            if let Some(bit_length) = constraints.bit_size() {
                Ok(Box::new(move |input: BitIn| -> IResult<BitIn, O> {
                    let (input, is_extended) = read_bit(input)?;
                    if is_extended {
                        map_res(decode_normally_small_number, |i| {
                            let index = i + enumerated.extensible.unwrap();
                            O::try_from(index as i128).map_err(|_| {
                                nom::Err::Error(Error {
                                    input,
                                    code: nom::error::ErrorKind::OneOf,
                                })
                            })
                        })(input)
                    } else {
                        map_res(read_int::<i128>(bit_length), |i| {
                            O::try_from(i).map_err(|_| {
                                nom::Err::Error(Error {
                                    input,
                                    code: nom::error::ErrorKind::OneOf,
                                })
                            })
                        })(input)
                    }
                }))
            } else {
                unreachable!()
            }
        } else {
            if let Some(bit_length) = constraints.bit_size() {
                Ok(Box::new(move |input: BitIn| -> IResult<BitIn, O> {
                    map_res(read_int::<i128>(bit_length), |i| {
                        O::try_from(i).map_err(|_| {
                            nom::Err::Error(Error {
                                input,
                                code: nom::error::ErrorKind::OneOf,
                            })
                        })
                    })(input)
                }))
            } else {
                unreachable!()
            }
        }
    }

    fn decode_choice<O: DecoderForIndex<BitIn<'a>>>(
        choice: asnr_grammar::types::Choice,
    ) -> fn(BitIn<'a>) -> IResult<BitIn<'a>, O> {
        todo!()
    }

    fn decode_null<N: Default>(input: BitIn<'a>) -> IResult<BitIn<'a>, N> {
        Ok((input, N::default()))
    }

    fn decode_boolean(input: BitIn<'a>) -> IResult<BitIn<'a>, bool> {
        read_bit(input)
    }

    fn decode_bit_string(
        bit_string: asnr_grammar::types::BitString,
    ) -> fn(BitIn<'a>) -> IResult<BitIn<'a>, Vec<bool>> {
        todo!()
    }

    fn decode_character_string(
        char_string: asnr_grammar::types::CharacterString,
    ) -> fn(BitIn<'a>) -> IResult<BitIn<'a>, String> {
        todo!()
    }

    fn decode_sequence<T: DecodeMember<BitIn<'a>> + Default>(
        sequence: asnr_grammar::types::Sequence,
    ) -> Result<Box<dyn FnMut(BitIn<'a>) -> IResult<BitIn, T>>, DecodingError> {
        if let Some(extension_index) = sequence.extensible {
            Ok(Box::new(move |input| {
                let (mut input, is_extended) = read_bit(input)?;
                let mut member_presence = vec![];
                for m in sequence.members.iter() {
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
                        input = instance.decode_member_at_index(index, Uper::new(), input)?;
                    }
                }
                if is_extended {
                    let (mut input, length) = decode_normally_small_number(input)?;
                    let mut extension_presence = vec![];
                    for _ in 0..length {
                        if m.is_optional {
                            let parsed = read_bit(input)?;
                            input = parsed.0;
                            extension_presence.push(parsed.1);
                        } else {
                            extension_presence.push(true)
                        }
                    }
                    input = instance.decode_member_at_index(extension_index, Uper::new(), input)?
                }
                Ok((input, instance))
            }))
        } else {
            Ok(Box::new(move |mut input| {
                let mut member_presence = vec![];
                for m in sequence.members.iter() {
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
                        input = instance.decode_member_at_index(index, Uper::new(), input)?;
                    }
                }
                Ok((input, instance))
            }))
        }
    }

    fn decode_sequence_of<T: Decode<BitIn<'a>>>(
        sequence_of: asnr_grammar::types::SequenceOf,
        member_decoder: impl FnMut(BitIn<'a>) -> IResult<BitIn<'a>, T>,
    ) -> Result<Box<dyn FnMut(BitIn<'a>) -> IResult<BitIn<'a>, Vec<T>>>, DecodingError> {
        todo!()
    }

    fn decode_unknown_extension(input: BitIn<'a>) -> IResult<BitIn<'a>, Vec<u8>> {
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

fn decode_normally_small_number(input: BitIn) -> IResult<BitIn, usize> {
    let (input, over_63) = read_bit(input)?;
    if over_63 {
        let (input, length_det) = decode_length_determinant(input)?;
        if let LengthDeterminant::Content(i) = length_det {
            Ok((input, i))
        } else {
            Err(nom::Err::Error(Error {
                input,
                code: nom::error::ErrorKind::Digit,
            }))
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
    use asnr_grammar::{
        constraints::*,
        types::{Enumeral, Enumerated, Integer},
        *,
    };

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
        #[derive(Debug, PartialEq)]
        enum TestEnum {
            One,
            Two,
            Three,
            UnknownExt,
        }

        impl TryFrom<i128> for TestEnum {
            type Error = DecodingError;

            fn try_from(value: i128) -> Result<Self, Self::Error> {
                match value {
                    x if x == Self::One as i128 => Ok(Self::One),
                    x if x == Self::Two as i128 => Ok(Self::Two),
                    x if x == Self::Three as i128 => Ok(Self::Three),
                    _ => Ok(Self::UnknownExt),
                }
            }
        }

        let mut decoder = Uper::decode_enumerated::<TestEnum>(Enumerated {
            extensible: None,
            members: vec![
                Enumeral {
                    name: "One".into(),
                    description: None,
                    index: 0,
                },
                Enumeral {
                    name: "Two".into(),
                    description: None,
                    index: 1,
                },
                Enumeral {
                    name: "Three".into(),
                    description: None,
                    index: 2,
                },
            ],
            constraints: vec![],
        })
        .unwrap();
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
        decoder = Uper::decode_enumerated::<TestEnum>(Enumerated {
            extensible: Some(2),
            members: vec![
                Enumeral {
                    name: "One".into(),
                    description: None,
                    index: 0,
                },
                Enumeral {
                    name: "Two".into(),
                    description: None,
                    index: 1,
                },
                Enumeral {
                    name: "Three".into(),
                    description: None,
                    index: 2,
                },
            ],
            constraints: vec![],
        })
        .unwrap();
        assert_eq!(
            decoder(BSlice::from(bits![static u8, Msb0; 0,0,0]))
                .unwrap()
                .1,
            TestEnum::One
        );
        assert_eq!(
            decoder(BSlice::from(bits![static u8, Msb0; 1,0,0,0,0,0,0,0]))
                .unwrap()
                .1,
            TestEnum::Three
        );
        assert_eq!(
            decoder(BSlice::from(bits![static u8, Msb0; 1,0,0,0,0,0,1,1]))
                .unwrap()
                .1,
            TestEnum::UnknownExt
        );
    }
}

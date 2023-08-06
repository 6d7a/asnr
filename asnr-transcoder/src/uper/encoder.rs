use alloc::{boxed::Box, format, string::String, vec::Vec};
use asnr_grammar::{constraints::Constraint, types::*};
use bitvec::{bitvec, prelude::Msb0, vec::BitVec, view::BitView};
use core::fmt::Debug;

use crate::{
    error::{DecodingError, EncodingError},
    Encode, Encoder, EncoderForIndex,
};

use super::{
    bit_length,
    per_visible::{PerVisibleAlphabetConstraints, PerVisibleRangeConstraints},
    rustify_name, AsBytesDummy, Uper,
};

type BitOut = BitVec<u8, Msb0>;

impl Encoder<u8, BitOut> for Uper {
    fn encode_integer<I>(
        integer: Integer,
    ) -> Result<Box<dyn Fn(I, BitOut) -> Result<BitOut, EncodingError>>, EncodingError>
    where
        I: num::Integer + num::ToPrimitive + num::FromPrimitive + Copy,
    {
        let constraints = per_visible_range_constraints(true, &integer.constraints)?;
        if constraints.is_extensible() {
            if let Some(bit_length) = constraints.bit_length() {
                Ok(Box::new(
                    move |encodable, mut output| -> Result<BitOut, EncodingError> {
                        let extends_constraints =
                            write_extended_bit(&constraints, encodable, &mut output)?;
                        if extends_constraints {
                            let varlength = encode_varlength_integer(encodable, None)?;
                            assert_byte_alignment(varlength.len())?;
                            wrap_in_length_determinant::<I>(
                                varlength.len() / 8,
                                varlength,
                                None,
                                output,
                            )
                        } else {
                            encode_constrained_integer(
                                encodable - constraints.min().unwrap(),
                                bit_length,
                                output,
                            )
                        }
                    },
                ))
            } else {
                Ok(Box::new(
                    move |encodable, mut output| -> Result<BitOut, EncodingError> {
                        let extends_constraints =
                            write_extended_bit(&constraints, encodable, &mut output)?;

                        let varlength = encode_varlength_integer(
                            encodable,
                            if extends_constraints {
                                None
                            } else {
                                constraints.min()
                            },
                        )?;
                        assert_byte_alignment(varlength.len())?;
                        wrap_in_length_determinant::<I>(
                            varlength.len() / 8,
                            varlength,
                            None,
                            output,
                        )
                    },
                ))
            }
        } else {
            if let Some(bit_length) = constraints.bit_length() {
                Ok(Box::new(
                    move |encodable, output| -> Result<BitOut, EncodingError> {
                        constraints.lies_within(&encodable)?;
                        encode_constrained_integer(
                            encodable - constraints.min().unwrap(),
                            bit_length,
                            output,
                        )
                    },
                ))
            } else {
                Ok(Box::new(
                    move |encodable, output| -> Result<BitOut, EncodingError> {
                        constraints.lies_within(&encodable)?;
                        let varlength = encode_varlength_integer(encodable, constraints.min())?;
                        assert_byte_alignment(varlength.len())?;
                        wrap_in_length_determinant::<I>(
                            varlength.len() / 8,
                            varlength,
                            None,
                            output,
                        )
                    },
                ))
            }
        }
    }

    fn encode_boolean(value: bool, mut output: BitOut) -> Result<BitOut, EncodingError> {
        output.push(value);
        Ok(output)
    }

    fn encode_null(output: BitOut) -> Result<BitOut, EncodingError> {
        Ok(output)
    }

    fn encode_bit_string(
        bit_string: BitString,
    ) -> Result<Box<dyn Fn(Vec<bool>, BitOut) -> Result<BitOut, EncodingError>>, EncodingError>
    {
        let constraints = per_visible_range_constraints(false, &bit_string.constraints)?;
        if constraints.is_extensible() {
            Ok(Box::new(
                move |encodable: Vec<bool>, mut output: BitOut| -> Result<BitOut, EncodingError> {
                    let actual_length = encodable.len();
                    let _ = write_extended_bit(&constraints, actual_length, &mut output)?;
                    let to_wrap = encodable.into_iter().fold(bitvec![u8, Msb0;], |acc, curr| {
                        Self::encode_boolean(curr, acc).unwrap()
                    });
                    with_size_length_determinant(actual_length, &constraints, to_wrap, output)
                },
            ))
        } else {
            Ok(Box::new(
                move |encodable: Vec<bool>, output: BitOut| -> Result<BitOut, EncodingError> {
                    let actual_length = encodable.len();
                    let to_wrap = encodable.into_iter().fold(bitvec![u8, Msb0;], |acc, curr| {
                        Self::encode_boolean(curr, acc).unwrap()
                    });
                    with_size_length_determinant(actual_length, &constraints, to_wrap, output)
                },
            ))
        }
    }

    fn encode_character_string(
        character_string: CharacterString,
    ) -> Result<Box<dyn Fn(&str, BitOut) -> Result<BitOut, EncodingError>>, EncodingError> {
        let mut constraints = PerVisibleRangeConstraints::default_unsigned();
        let mut permitted_alphabet =
            PerVisibleAlphabetConstraints::default_for(character_string.r#type);
        for c in &character_string.constraints {
            constraints +=
                c.try_into()
                    .map_err(|_: DecodingError<AsBytesDummy>| EncodingError {
                        details: format!("Failed to parse character string constraints"),
                    })?;
            PerVisibleAlphabetConstraints::try_new::<AsBytesDummy>(c, character_string.r#type)?
                .map(|mut p| permitted_alphabet += &mut p);
        }
        permitted_alphabet.finalize();
        if constraints.is_extensible() && permitted_alphabet.is_known_multiplier_string() {
            Ok(Box::new(
                move |encodable: &str, mut output: BitOut| -> Result<BitOut, EncodingError> {
                    let actual_length = encodable.len();
                    let _ = write_extended_bit(&constraints, actual_length, &mut output)?;
                    let to_wrap = encode_sized_string(&permitted_alphabet, encodable)?;
                    with_size_length_determinant(actual_length, &constraints, to_wrap, output)
                },
            ))
        } else {
            Ok(Box::new(
                move |encodable: &str, output: BitOut| -> Result<BitOut, EncodingError> {
                    let to_wrap = encode_sized_string(&permitted_alphabet, encodable)?;
                    with_size_length_determinant(encodable.len(), &constraints, to_wrap, output)
                },
            ))
        }
    }

    fn encode_sequence<S: EncoderForIndex<u8, BitOut> + Debug>(
        sequence: SequenceOrSet,
    ) -> Result<Box<dyn Fn(S, BitOut) -> Result<BitOut, EncodingError>>, EncodingError> {
        let member_list: Vec<(usize, String, bool)> = sequence
            .members
            .iter()
            .enumerate()
            .map(|(i, m)| (i, rustify_name(&m.name), m.is_optional))
            .collect();
        let encode_optional_map = |encodable: S,
                                   mut output: BitOut,
                                   member_list: &Vec<(usize, String, bool)>|
         -> (S, BitOut, Vec<bool>) {
            // This is performance-wise pretty ugly and should be handled differently
            // in the future
            let stringified_encodable = format!("{encodable:?}");
            let mut skip_list: Vec<bool> = member_list
                .iter()
                .filter_map(|(_, name, opt)| {
                    opt.then(|| stringified_encodable.contains(&format!("{name}: None")))
                })
                .map(|not_present| {
                    output.push(!not_present);
                    not_present
                })
                .collect();
            // Reverting skip_list so that we can pop presence information from back
            skip_list.reverse();
            (encodable, output, skip_list)
        };
        if let Some(index_of_first_extension) = sequence.extensible {
            Ok(Box::new(move |encodable, mut output| {
                let root_bits = bitvec![u8, Msb0;];
                let mut extension_bits = bitvec![u8, Msb0;];
                let (encodable, mut root_bits, mut skip_list) =
                    encode_optional_map(encodable, root_bits, &member_list);
                let mut extension_presence = Vec::new();
                'encoding_members: for (index, _, optional) in &member_list {
                    if *optional
                        && skip_list.pop().ok_or(EncodingError {
                            details: format!(
                                "Optionals list is too short for Encodable {:?}",
                                encodable
                            ),
                        })?
                    {
                        if index >= &index_of_first_extension {
                            extension_presence.push(false);
                        }
                        continue 'encoding_members;
                    }
                    if index < &index_of_first_extension {
                        root_bits =
                            S::encoder_for_index::<Uper>((*index).try_into().map_err(|_| {
                                EncodingError {
                                    details: format!("Index {index} exceeds usize range!"),
                                }
                            })?)?(&encodable, root_bits)?;
                    } else {
                        extension_presence.push(true);
                        let mut extension =
                            S::encoder_for_index::<Uper>((*index).try_into().map_err(|_| {
                                EncodingError {
                                    details: format!("Index {index} exceeds usize range!"),
                                }
                            })?)?(&encodable, bitvec![u8, Msb0;])?;
                        extension = align_back(extension);
                        extension_bits = wrap_in_length_determinant(
                            extension_bits.len() / 8,
                            extension,
                            Some(0),
                            extension_bits,
                        )?;
                    }
                }
                output.push(!extension_bits.is_empty());
                output.append(&mut root_bits);
                if !extension_bits.is_empty() {
                    output = encode_normally_small_number(extension_presence.len(), output)?;
                    for bit in extension_presence {
                        output.push(bit);
                    }
                    output.append(&mut extension_bits);
                }
                Ok(output)
            }))
        } else {
            Ok(Box::new(move |encodable, output| {
                let (encodable, mut output, mut skip_list) =
                    encode_optional_map(encodable, output, &member_list);
                'encoding_members: for (index, _, optional) in &member_list {
                    if *optional
                        && skip_list.pop().ok_or(EncodingError {
                            details: format!(
                                "Optionals list is too short for Encodable {:?}",
                                encodable
                            ),
                        })?
                    {
                        continue 'encoding_members;
                    }
                    output = S::encoder_for_index::<Uper>((*index).try_into().map_err(|_| {
                        EncodingError {
                            details: format!("Index {index} exceeds usize range!"),
                        }
                    })?)?(&encodable, output)?;
                }
                Ok(output)
            }))
        }
    }

    fn encode_enumerated<E: Encode<u8, BitOut> + Debug>(
        enumerated: Enumerated,
    ) -> Result<Box<dyn Fn(E, BitOut) -> Result<BitOut, EncodingError>>, EncodingError> {
        let mut member_ids = enumerated
            .members
            .iter()
            .map(|m| (rustify_name(&m.name), m.index))
            .collect::<Vec<(String, i128)>>();
        member_ids.sort_by(|(_, a), (_, b)| a.cmp(b));
        let indices_for_member = member_ids
            .into_iter()
            .enumerate()
            .map(|(i, (n, _))| (n, i))
            .collect::<Vec<(String, usize)>>();
        if let Some(index_of_first_extension) = enumerated.extensible {
            Ok(Box::new(move |encodable, mut output| {
                let index = indices_for_member
                    .iter()
                    .find_map(|(name, index)| (&format!("{encodable:?}") == name).then(|| *index))
                    .ok_or(EncodingError {
                        details: format!(
                            "Could not find enumerated option {encodable:?} among {:?}",
                            &indices_for_member
                        ),
                    })?;
                if index >= index_of_first_extension {
                    output.push(true);
                    encode_normally_small_number(index - index_of_first_extension, output)
                } else {
                    output.push(false);
                    encode_constrained_integer(
                        index,
                        bit_length(0, (index_of_first_extension - 1) as i128),
                        output,
                    )
                }
            }))
        } else {
            Ok(Box::new(move |encodable, output| {
                let index = indices_for_member
                    .iter()
                    .find_map(|(name, index)| (&format!("{encodable:?}") == name).then(|| *index))
                    .ok_or(EncodingError {
                        details: format!(
                            "Could not find enumerated option {encodable:?} among {:?}",
                            &indices_for_member
                        ),
                    })?;
                encode_constrained_integer(
                    index,
                    bit_length(0, (indices_for_member.len() - 1) as i128),
                    output,
                )
            }))
        }
    }

    fn encode_choice<C: EncoderForIndex<u8, BitOut> + Debug>(
        choice: Choice,
    ) -> Result<Box<dyn Fn(C, BitOut) -> Result<BitOut, EncodingError>>, EncodingError> {
        let mut indices_for_member = choice
            .options
            .iter()
            .enumerate()
            .map(|(i, m)| {
                (
                    rustify_name(&m.name),
                    m.tag.as_ref().map_or(i, |t| t.id as usize),
                )
            })
            .collect::<Vec<(String, usize)>>();
        indices_for_member.sort_by(|(_, a), (_, b)| a.cmp(b));
        if let Some(index_of_first_extension) = choice.extensible {
            Ok(Box::new(move |encodable, output| {
                let index = indices_for_member
                    .iter()
                    .find_map(|(name, index)| {
                        (format!("{encodable:?}").contains(name)).then(|| *index)
                    })
                    .ok_or(EncodingError {
                        details: format!(
                            "Could not find choice option {encodable:?} among {:?}",
                            &indices_for_member
                        ),
                    })?;
                if index >= index_of_first_extension {
                    let output =
                        encode_normally_small_number(index - index_of_first_extension, output)?;
                    let to_wrap =
                        align_back(C::encoder_for_index::<Uper>(index.try_into().map_err(
                            |_| EncodingError {
                                details: format!("Index {index} exceeds usize range!"),
                            },
                        )?)?(&encodable, bitvec![u8, Msb0;])?);
                    wrap_in_length_determinant(to_wrap.len() / 8, to_wrap, Some(0), output)
                } else {
                    let output = encode_constrained_integer(
                        index,
                        bit_length(0, (indices_for_member.len() - 1) as i128),
                        output,
                    )?;
                    C::encoder_for_index::<Uper>(index.try_into().map_err(|_| EncodingError {
                        details: format!("Index {index} exceeds usize range!"),
                    })?)?(&encodable, output)
                }
            }))
        } else {
            Ok(Box::new(move |encodable, output| {
                let index = indices_for_member
                    .iter()
                    .find_map(|(name, index)| {
                        (format!("{encodable:?}").contains(name)).then(|| *index)
                    })
                    .ok_or(EncodingError {
                        details: format!(
                            "Could not find enumerated option {encodable:?} among {:?}",
                            &indices_for_member
                        ),
                    })?;
                let output = encode_constrained_integer(
                    index,
                    bit_length(0, (indices_for_member.len() - 1) as i128),
                    output,
                )?;
                C::encoder_for_index::<Uper>(index.try_into().map_err(|_| EncodingError {
                    details: format!("Index {index} exceeds usize range!"),
                })?)?(&encodable, output)
            }))
        }
    }
}

fn per_visible_range_constraints(
    signed: bool,
    constraint_list: &Vec<Constraint>,
) -> Result<PerVisibleRangeConstraints, EncodingError> {
    let mut constraints = if signed {
        PerVisibleRangeConstraints::default()
    } else {
        PerVisibleRangeConstraints::default_unsigned()
    };
    for c in constraint_list {
        constraints += c
            .try_into()
            .map_err(|_: DecodingError<AsBytesDummy>| EncodingError {
                details: format!("Failed to parse bit string constraints"),
            })?
    }
    Ok(constraints)
}

fn encode_sized_string(
    permitted_alphabet: &PerVisibleAlphabetConstraints,
    string: &str,
) -> Result<BitOut, EncodingError> {
    let bit_length = permitted_alphabet.bit_length();
    if bit_length == 0 {
        return encode_sized_string(&permitted_alphabet.fall_back_to_standard_charset(), string);
    }
    if permitted_alphabet.is_known_multiplier_string() {
        let mut output = BitVec::new();
        for c in string.chars() {
            let index =
                permitted_alphabet
                    .index_by_character_map()?
                    .get(&c)
                    .ok_or(EncodingError {
                        details: format!("Character {c} is not part of permitted character set"),
                    })?;
            output = encode_constrained_integer(*index, bit_length, output)?;
        }
        Ok(output)
    } else {
        Ok(string.as_bytes().view_bits::<Msb0>().to_bitvec())
    }
}

fn assert_byte_alignment(length: usize) -> Result<(), EncodingError> {
    if length % 8 != 0 {
        return Err(EncodingError {
            details: "Variable-length integer's encoding violates byte-alignment!".into(),
        });
    }
    Ok(())
}

fn write_extended_bit<I>(
    constraints: &PerVisibleRangeConstraints,
    encodable: I,
    output: &mut BitOut,
) -> Result<bool, EncodingError>
where
    I: num::Integer + num::ToPrimitive + num::FromPrimitive + Copy,
{
    let within_constraints = constraints.lies_within(&encodable)?;
    if within_constraints {
        output.push(false);
    } else {
        output.push(true);
    }
    Ok(within_constraints)
}

/// Wraps the provided buffer in a length determinant for size constraints
/// ### Params
/// * `actual_size` - number of counted items (i.e. size) of the encoded value. An _item_ can be an octet, a character, a member of a collection, depending on the ASN1 type that is encoded.
/// * `constraints` - specification of the encoded type's constraints
/// * `to_wrap` - BitVec containing the encoded value that should receive a size length determinant prefix
/// * `output` - the output buffer that the sized value should be appended to
/// ### Reference in ASN1 Complete (Larmouth 302)
/// >* _With no PER-visible size constraint, or a constraint that allows counts
/// in excess of 64K, we encode a general length determinant._
/// >* _For abstract values outside the root, a general length determinant is again used._
/// >* _With a size constraint that gives a fixed value for the count, there
/// is no length determinant encoding._
/// >* _Otherwise, we encode the count exactly like an integer with the equivalent constraint_
fn with_size_length_determinant(
    actual_size: usize,
    constraints: &PerVisibleRangeConstraints,
    mut to_wrap: BitOut,
    output: BitOut,
) -> Result<BitOut, EncodingError> {
    if let (Some(bit_length), Some(Some(_)), true) = (
        constraints.bit_length(),
        constraints
            .range_width::<AsBytesDummy>()?
            .map(|w| (w <= 65536).then(|| w)),
        constraints.lies_within(&actual_size)?,
    ) {
        let mut output = encode_constrained_integer(
            actual_size - constraints.min().unwrap_or(0),
            bit_length,
            output,
        )?;
        output.append(&mut to_wrap);
        Ok(output)
    } else {
        wrap_in_length_determinant(actual_size, to_wrap, Some(0), output)
    }
}

fn wrap_in_length_determinant<I>(
    length_offset: usize,
    mut to_wrap: BitOut,
    min: Option<I>,
    mut output: BitOut,
) -> Result<BitOut, EncodingError> {
    match length_offset {
        x if x < 128 => {
            let mut length_det = encode_constrained_integer(x, 8, output)?;
            length_det.append(&mut to_wrap);
            Ok(length_det)
        }
        x if x < 16384 => {
            output.append(&mut bitvec![u8, Msb0; 1, 0]);
            let mut length_det = encode_constrained_integer(x, 14, output)?;
            length_det.append(&mut to_wrap);
            Ok(length_det)
        }
        x => {
            let (mut fragment, fragment_size) = match x {
                s if s < 32768 => (bitvec![u8, Msb0; 1,1,0,0,0,0,1,0], 32768),
                s if s < 49152 => (bitvec![u8, Msb0; 1,1,0,0,0,0,1,1], 49152),
                _ => (bitvec![u8, Msb0; 1,1,0,0,0,1,0,0], 65536),
            };
            fragment.extend(to_wrap[..fragment_size].iter());
            wrap_in_length_determinant(length_offset - fragment_size, fragment, min, output)
        }
    }
}

fn encode_varlength_integer<I>(integer: I, min: Option<I>) -> Result<BitOut, EncodingError>
where
    I: num::Integer + num::ToPrimitive + Copy,
{
    let int_as_i128 = integer.to_i128().ok_or(EncodingError {
        details: "Failed to convert integer to u128!".into(),
    })?;
    let min_as_i128 = min.map(|i| {
        i.to_i128().ok_or(EncodingError {
            details: "Failed to convert integer to u128!".into(),
        })
    });
    match min_as_i128 {
        Some(Err(e)) => Err(e),
        Some(Ok(m)) => {
            let offset = int_as_i128 - m;
            let output =
                encode_constrained_integer(offset, bit_length(0, offset), bitvec![u8, Msb0;])?;
            Ok(align(output))
        }
        None if int_as_i128 >= 0 => {
            let output = encode_constrained_integer(
                int_as_i128,
                bit_length(0, int_as_i128),
                bitvec![u8, Msb0;],
            )?;
            Ok(align(pad(1, output)))
        }
        None => {
            let bit_length_signed = bit_length(0, int_as_i128.abs() - 1) + 1;
            Ok(align(
                int_as_i128.to_be_bytes().view_bits::<Msb0>()[(128 - bit_length_signed)..]
                    .to_bitvec(),
            ))
        }
    }
}

fn encode_normally_small_number<I>(number: I, mut output: BitOut) -> Result<BitOut, EncodingError>
where
    I: num::Integer + num::ToPrimitive + Copy,
{
    if number.to_u32().unwrap_or(64) > 63 {
        Err(EncodingError {
            details: "Encoding normally-small numbers larger than 63 is not supported yet!".into(),
        })
    } else {
        output.push(false);
        encode_constrained_integer(number, 6, output)
    }
}

fn encode_constrained_integer<I>(
    integer: I,
    bit_length: usize,
    mut output: BitOut,
) -> Result<BitOut, EncodingError>
where
    I: num::Integer + num::ToPrimitive + Copy,
{
    let as_u128 = integer.to_u128().ok_or(EncodingError {
        details: "Failed to convert integer to u128!".into(),
    })?;
    output.extend((0..bit_length).rev().map(|n| (as_u128 >> n) & 1 != 0));
    Ok(output)
}

fn pad(bytes: usize, mut output: BitOut) -> BitOut {
    let mut padding = bitvec![u8, Msb0; 0; bytes];
    padding.append(&mut output);
    padding
}

fn align(output: BitOut) -> BitOut {
    let missing_bits = 8 - output.len() % 8;
    if missing_bits == 8 {
        return output;
    }
    pad(missing_bits, output)
}

fn align_back(mut output: BitOut) -> BitOut {
    let missing_bits = 8 - output.len() % 8;
    if missing_bits == 8 {
        return output;
    }
    for _ in 0..missing_bits {
        output.push(false);
    }
    return output;
}

#[cfg(test)]
mod tests {
    use crate::uper::{
        encoder::{align, encode_constrained_integer, pad},
        Uper,
    };
    use asnr_compiler_derive::asn1_internal_tests;
    use bitvec::{bitvec, prelude::Msb0};

    #[test]
    fn pads_bits() {
        let input = bitvec![u8, Msb0; 1, 1];
        assert_eq!(pad(1, input), bitvec![u8, Msb0; 0, 1, 1])
    }

    #[test]
    fn aligns_bits() {
        let input = bitvec![u8, Msb0; 1, 1];
        assert_eq!(align(input), bitvec![u8, Msb0; 0,0,0,0,0,0,1,1])
    }

    #[test]
    fn encodes_constrained_int() {
        assert_eq!(
            encode_constrained_integer(0, 6, bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,0,0,0,0,0]
        );
        assert_eq!(
            encode_constrained_integer(0, 2, bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,0]
        );
        assert_eq!(
            encode_constrained_integer(1, 2, bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,1]
        );
        assert_eq!(
            encode_constrained_integer(3, 2, bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 1,1]
        );
        assert_eq!(
            encode_constrained_integer(2, 8, bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,0,0,0,0,0,1,0]
        );
        assert_eq!(
            encode_constrained_integer(65537, 17, bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1]
        )
    }

    #[test]
    fn encodes_simple_constrained_integer() {
        asn1_internal_tests!("TestInteger ::= INTEGER(3..6)");
        assert_eq!(
            TestInteger::encode::<Uper>(TestInteger(3), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,0]
        );
        assert_eq!(
            TestInteger::encode::<Uper>(TestInteger(5), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 1,0]
        );
        assert!(TestInteger::encode::<Uper>(TestInteger(7), bitvec![u8, Msb0;]).is_err())
    }

    #[test]
    fn encodes_semi_constrained_integer() {
        asn1_internal_tests!("TestInteger ::= INTEGER(-1..MAX)");
        assert_eq!(
            TestInteger::encode::<Uper>(TestInteger(3), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0]
        );
        assert_eq!(
            TestInteger::encode::<Uper>(TestInteger(127), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,0,0,0,0,0,0,1, 1,0,0,0,0,0,0,0]
        );
        assert_eq!(
            TestInteger::encode::<Uper>(TestInteger(255), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,0,0,0,0,0,1,0, 0,0,0,0,0,0,0,1, 0,0,0,0,0,0,0,0]
        );
        assert!(TestInteger::encode::<Uper>(TestInteger(-2), bitvec![u8, Msb0;]).is_err())
    }

    #[test]
    fn encodes_unconstrained_integer() {
        asn1_internal_tests!("TestInteger ::= INTEGER");
        assert_eq!(
            TestInteger::encode::<Uper>(TestInteger(4096), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,0,0,0,0,0,1,0, 0,0,0,1,0,0,0,0, 0,0,0,0,0,0,0,0]
        );
    }

    #[test]
    fn encodes_downwards_unconstrained_integer() {
        asn1_internal_tests!("TestInteger ::= INTEGER(MIN..65535)");
        assert_eq!(
            TestInteger::encode::<Uper>(TestInteger(127), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,0,0,0,0,0,0,1, 0,1,1,1,1,1,1,1]
        );
        assert_eq!(
            TestInteger::encode::<Uper>(TestInteger(-128), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,0,0,0,0,0,0,1, 1,0,0,0,0,0,0,0]
        );
        assert_eq!(
            TestInteger::encode::<Uper>(TestInteger(128), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,0,0,0,0,0,1,0, 0,0,0,0,0,0,0,0, 1,0,0,0,0,0,0,0]
        );
    }

    #[test]
    fn encodes_boolean() {
        asn1_internal_tests!("TestBool ::= BOOLEAN");
        assert_eq!(
            TestBool::encode::<Uper>(TestBool(true), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 1]
        );
        assert_eq!(
            TestBool::encode::<Uper>(TestBool(false), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0]
        );
    }

    #[test]
    fn encodes_null() {
        asn1_internal_tests!("TestNull ::= NULL");
        assert_eq!(
            TestNull::encode::<Uper>(TestNull, bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0;]
        );
    }

    #[test]
    fn encodes_fixed_size_bit_string() {
        asn1_internal_tests!("TestBitString ::= BIT STRING (3)");
        assert_eq!(
            TestBitString::encode::<Uper>(
                TestBitString(vec![true, false, true]),
                bitvec![u8, Msb0;]
            )
            .unwrap(),
            bitvec![u8, Msb0; 1,0,1]
        );
    }

    #[test]
    fn encodes_constrained_bit_string() {
        asn1_internal_tests!("TestBitString ::= BIT STRING (3..4)");
        assert_eq!(
            TestBitString::encode::<Uper>(
                TestBitString(vec![false, false, true]),
                bitvec![u8, Msb0;]
            )
            .unwrap(),
            bitvec![u8, Msb0; 0,0,0,1]
        );
        assert_eq!(
            TestBitString::encode::<Uper>(
                TestBitString(vec![false, false, true, true]),
                bitvec![u8, Msb0;]
            )
            .unwrap(),
            bitvec![u8, Msb0; 1,0,0,1,1]
        );
        assert!(TestBitString::encode::<Uper>(
            TestBitString(vec![false, false, true, true, true]),
            bitvec![u8, Msb0;]
        )
        .is_err())
    }

    #[test]
    fn encodes_semi_constrained_bit_string() {
        asn1_internal_tests!("TestBitString ::= BIT STRING (3..MAX)");
        assert_eq!(
            TestBitString::encode::<Uper>(
                TestBitString(vec![false, false, true]),
                bitvec![u8, Msb0;]
            )
            .unwrap(),
            bitvec![u8, Msb0; 0,0,0,0,0,0,1,1, 0,0,1]
        );
        assert!(TestBitString::encode::<Uper>(
            TestBitString(vec![false, false]),
            bitvec![u8, Msb0;]
        )
        .is_err())
    }

    #[test]
    fn encodes_unconstrained_bit_string() {
        asn1_internal_tests!("TestBitString ::= BIT STRING");
        assert_eq!(
            TestBitString::encode::<Uper>(TestBitString(vec![]), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,0,0,0,0,0,0,0]
        );
    }

    #[test]
    fn encodes_extended_fixed_size_bit_string() {
        asn1_internal_tests!("TestBitString ::= BIT STRING (3,...)");
        assert_eq!(
            TestBitString::encode::<Uper>(TestBitString(vec![]), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 1,0,0,0,0,0,0,0,0]
        );
    }

    #[test]
    fn encodes_extended_constrained_bit_string() {
        asn1_internal_tests!("TestBitString ::= BIT STRING (3..4,...)");
        assert_eq!(
            TestBitString::encode::<Uper>(
                TestBitString(vec![false, false, true, true, true]),
                bitvec![u8, Msb0;]
            )
            .unwrap(),
            bitvec![u8, Msb0; 1, 0,0,0,0,0,1,0,1, 0,0,1,1,1]
        );
        assert_eq!(
            TestBitString::encode::<Uper>(
                TestBitString(vec![false, false, true, true]),
                bitvec![u8, Msb0;]
            )
            .unwrap(),
            bitvec![u8, Msb0; 0,1,0,0,1,1]
        );
    }

    #[test]
    fn encodes_extended_semi_constrained_bit_string() {
        asn1_internal_tests!("TestBitString ::= BIT STRING (3..MAX,...)");
        assert_eq!(
            TestBitString::encode::<Uper>(
                TestBitString(vec![false, false, true]),
                bitvec![u8, Msb0;]
            )
            .unwrap(),
            bitvec![u8, Msb0; 0, 0,0,0,0,0,0,1,1, 0,0,1]
        );
        assert_eq!(
            TestBitString::encode::<Uper>(TestBitString(vec![false, false]), bitvec![u8, Msb0;])
                .unwrap(),
            bitvec![u8, Msb0; 1, 0,0,0,0,0,0,1,0, 0,0]
        )
    }

    #[test]
    fn encodes_constrained_character_string_with_permitted_alphabet() {
        asn1_internal_tests!(
            r#"TestString ::= BMPString (SIZE(1..4) INTERSECTION FROM("te" | "s"))"#
        );
        assert_eq!(
            TestString::encode::<Uper>(TestString("test".into()), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 1,1, 1,0, 0,0, 0,1, 1,0]
        );
    }

    #[test]
    fn encodes_unconstrained_variable_size_character_string() {
        asn1_internal_tests!(r#"TestString ::= GraphicString"#);
        assert_eq!(
            TestString::encode::<Uper>(TestString("🦀".into()), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0;
            0,0,0,0,0,1,0,0,
            1,1,1,1,0,0,0,0,1,0,0,1,1,1,1,1,1,0,1,0,0,1,1,0,1,0,0,0,0,0,0,0
            ]
        );
    }

    #[test]
    fn encodes_constrained_extensible_character_string_with_permitted_alphabet() {
        asn1_internal_tests!(r#"TestString ::= NumericString (SIZE(1..4,...))"#);
        assert_eq!(
            TestString::encode::<Uper>(TestString("040234".into()), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0;
              1,
              0,0,0,0,0,1,1,0,
              0,0,0,1,
              0,1,0,1,
              0,0,0,1,
              0,0,1,1,
              0,1,0,0,
              0,1,0,1
            ]
        );
        assert_eq!(
            TestString::encode::<Uper>(TestString("040".into()), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0;
              0,
              1,0,
              0,0,0,1,
              0,1,0,1,
              0,0,0,1
            ]
        );
    }

    #[test]
    fn encodes_simple_enumerated() {
        asn1_internal_tests!(r#"TestEnum ::= ENUMERATED {m1, m2, m3}"#);
        assert_eq!(
            TestEnum::encode::<Uper>(TestEnum::m1, bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,0]
        );
        assert_eq!(
            TestEnum::encode::<Uper>(TestEnum::m2, bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,1]
        );
        assert_eq!(
            TestEnum::encode::<Uper>(TestEnum::m3, bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 1,0]
        );
    }

    #[test]
    fn encodes_indexed_enumerated() {
        asn1_internal_tests!(r#"TestEnum ::= ENUMERATED {m1( -8), m2(0), m3(-20)}"#);
        assert_eq!(
            TestEnum::encode::<Uper>(TestEnum::m1, bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,1]
        );
        assert_eq!(
            TestEnum::encode::<Uper>(TestEnum::m2, bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 1,0]
        );
        assert_eq!(
            TestEnum::encode::<Uper>(TestEnum::m3, bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,0]
        );
    }

    #[test]
    fn encodes_extended_enumerated() {
        asn1_internal_tests!(
            r#"HashAlgorithm ::= ENUMERATED { 
                sha256,
                ...,
                sha384
              }"#
        );
        assert_eq!(
            HashAlgorithm::encode::<Uper>(HashAlgorithm::sha256, bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0]
        );
        assert_eq!(
            HashAlgorithm::encode::<Uper>(HashAlgorithm::sha384, bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 1,0,0,0,0,0,0,0]
        );
    }

    #[test]
    fn encodes_empty_extended_enumerated() {
        asn1_internal_tests!(
            r#"InitiallyEmpty ::= ENUMERATED { 
                ...,
                now
              }"#
        );
        assert_eq!(
            InitiallyEmpty::encode::<Uper>(InitiallyEmpty::now, bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 1,0,0,0,0,0,0,0]
        );
    }

    #[test]
    fn encodes_simple_choice() {
        asn1_internal_tests!(
            r#"VarLengthNumber ::= CHOICE {
              content INTEGER(0..127),
              extension BOOLEAN
              }"#
        );
        assert_eq!(
            VarLengthNumber::encode::<Uper>(
                VarLengthNumber::content(VarLengthNumber_content(42)),
                bitvec![u8, Msb0;]
            )
            .unwrap(),
            bitvec![u8, Msb0; 0, 0,1,0,1,0,1,0]
        );
        assert_eq!(
            VarLengthNumber::encode::<Uper>(
                VarLengthNumber::extension(VarLengthNumber_extension(true)),
                bitvec![u8, Msb0;]
            )
            .unwrap(),
            bitvec![u8, Msb0; 1, 1]
        );
    }

    #[test]
    fn encodes_extended_choice() {
        asn1_internal_tests!(
            r#"SymmetricEncryptionKey ::= CHOICE {
              aes128Ccm OCTET STRING(SIZE(1)),
              ...
              none NULL
             }"#
        );
        assert_eq!(
            SymmetricEncryptionKey::encode::<Uper>(
                SymmetricEncryptionKey::aes128Ccm(SymmetricEncryptionKey_aes128Ccm("A2".into())),
                bitvec![u8, Msb0;]
            )
            .unwrap(),
            bitvec![u8, Msb0; 0]
        );
        assert_eq!(
            SymmetricEncryptionKey::encode::<Uper>(
                SymmetricEncryptionKey::none(SymmetricEncryptionKey_none),
                bitvec![u8, Msb0;]
            )
            .unwrap(),
            bitvec![u8, Msb0; 1,0,0,0,0,0,0,0]
        );
    }
}

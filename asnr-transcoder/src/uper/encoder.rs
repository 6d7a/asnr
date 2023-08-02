use alloc::{boxed::Box, format, vec::Vec};
use asnr_grammar::types::*;
use bitvec::{bitvec, prelude::Msb0, vec::BitVec, view::BitView};

use crate::{
    error::{DecodingError, EncodingError},
    Encoder,
};

use super::{
    bit_length,
    per_visible::{PerVisibleAlphabetConstraints, PerVisibleRangeConstraints},
    AsBytesDummy, Uper,
};

type BitOut = BitVec<u8, Msb0>;

impl Encoder<u8, BitOut> for Uper {
    fn encode_integer<I>(
        integer: Integer,
    ) -> Result<Box<dyn Fn(I, BitOut) -> Result<BitOut, EncodingError>>, EncodingError>
    where
        I: num::Integer + num::ToPrimitive + num::FromPrimitive + Copy,
    {
        let mut constraints = PerVisibleRangeConstraints::default();
        for c in &integer.constraints {
            constraints += c.try_into().map_err(|e| EncodingError {
                details: format!("Failed to parse integer constraints"),
            })?
        }
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
        let mut constraints = PerVisibleRangeConstraints::default_unsigned();
        for c in &bit_string.constraints {
            constraints += c
                .try_into()
                .map_err(|_: DecodingError<AsBytesDummy>| EncodingError {
                    details: format!("Failed to parse bit string constraints"),
                })?
        }
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
                        details: format!("Failed to parse bit string constraints"),
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
                    let to_wrap = encode_sized_string(& permitted_alphabet, encodable)?;
                    with_size_length_determinant(actual_length, &constraints, to_wrap, output)
                },
            ))
        } else {
            Ok(Box::new(
                move |encodable: &str, output: BitOut| -> Result<BitOut, EncodingError> {
                    let to_wrap = encode_sized_string(& permitted_alphabet, encodable)?;
                    with_size_length_determinant(encodable.len(), &constraints, to_wrap, output)
                },
            ))
        }
    }

    fn encode_sequence<S>(
        sequence: Sequence,
    ) -> Result<Box<dyn Fn(S, BitOut) -> Result<BitOut, EncodingError>>, EncodingError> {
      todo!()
    }
}

fn encode_sized_string(
    permitted_alphabet: &PerVisibleAlphabetConstraints,
    string: &str,
) -> Result<BitOut, EncodingError> {
    let bit_length = permitted_alphabet.bit_length();
    if bit_length == 0 {
        return encode_sized_string(
            &permitted_alphabet.fall_back_to_standard_charset(),
            string,
        );
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
}

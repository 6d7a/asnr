use alloc::{boxed::Box, format};
use asnr_grammar::types::Integer;
use bitvec::{bitvec, prelude::Msb0, vec::BitVec, view::BitView};

use crate::{error::EncodingError, Encoder};

use super::{bit_length, per_visible::PerVisibleRangeConstraints, Uper};

type BitOut = BitVec<u8, Msb0>;

impl Encoder<u8, BitOut> for Uper {
    fn encode_integer<I>(
        integer: Integer,
    ) -> Result<Box<dyn FnMut(I, BitOut) -> Result<BitOut, EncodingError>>, EncodingError>
    where
        I: num::Integer + num::ToPrimitive + num::FromPrimitive + Copy,
    {
        let mut constraints = PerVisibleRangeConstraints::default();
        for c in integer.constraints {
            constraints += c.try_into().map_err(|e| EncodingError {
                details: format!("Failed to parse integer constraints"),
            })?
        }
        if constraints.is_extensible() {
            if let Some(bit_length) = constraints.bit_size() {
                Ok(Box::new(
                    move |encodable, mut output| -> Result<BitOut, EncodingError> {
                        let extends_constraints = constraints.lies_within(&encodable)?;
                        if extends_constraints {
                            output.push(false);
                        } else {
                            output.push(true);
                        }
                        if extends_constraints {
                            let varlength = encode_varlength_integer(encodable, None)?;
                            if varlength.len() % 8 != 0 {
                                return Err(EncodingError { details: "Variable-length integer's encoding violates byte-alignment!".into() });
                            }
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
                        let extends_constraints = constraints.lies_within(&encodable)?;
                        if extends_constraints {
                            output.push(false);
                        } else {
                            output.push(true);
                        }
                        let varlength = if extends_constraints {
                            encode_varlength_integer(encodable, None)?
                        } else {
                            encode_varlength_integer(encodable, constraints.min())?
                        };
                        if varlength.len() % 8 != 0 {
                            return Err(EncodingError {
                                details:
                                    "Variable-length integer's encoding violates byte-alignment!"
                                        .into(),
                            });
                        }
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
            if let Some(bit_length) = constraints.bit_size() {
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
                        if varlength.len() % 8 != 0 {
                            return Err(EncodingError {
                                details:
                                    "Variable-length integer's encoding violates byte-alignment!"
                                        .into(),
                            });
                        }
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
      bit_string: asnr_grammar::types::BitString,
      ) -> Result<Box<dyn FnMut(Vec<bool>, BitOut) -> Result<BitOut, EncodingError>>, EncodingError> {
        let mut constraints = PerVisibleRangeConstraints::default_unsigned();
        for c in bit_string.constraints {
            constraints += c.try_into().map_err(|e| EncodingError {
                details: format!("Failed to parse bit string constraints"),
            })?
        }
        todo!()
    }
}

fn wrap_in_length_determinant<I>(
    length: usize,
    mut to_wrap: BitOut,
    min: Option<I>,
    output: BitOut,
) -> Result<BitOut, EncodingError> {
    match length {
        x if x < 128 => {
            let mut length_det = encode_constrained_integer(x, 8, bitvec![u8, Msb0;])?;
            length_det.append(&mut to_wrap);
            Ok(length_det)
        }
        x if x < 16384 => {
            let mut length_det = encode_constrained_integer(x, 14, bitvec![u8, Msb0; 1, 0])?;
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
            wrap_in_length_determinant(length - fragment_size, fragment, min, output)
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
    use std::println;

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
            TestInteger::encode::<Uper>(&TestInteger(3), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,0]
        );
        assert_eq!(
            TestInteger::encode::<Uper>(&TestInteger(5), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 1,0]
        );
        assert!(TestInteger::encode::<Uper>(&TestInteger(7), bitvec![u8, Msb0;]).is_err())
    }

    #[test]
    fn encodes_semi_constrained_integer() {
        asn1_internal_tests!("TestInteger ::= INTEGER(-1..MAX)");
        assert_eq!(
            TestInteger::encode::<Uper>(&TestInteger(3), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0]
        );
        assert_eq!(
            TestInteger::encode::<Uper>(&TestInteger(127), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,0,0,0,0,0,0,1, 1,0,0,0,0,0,0,0]
        );
        assert_eq!(
            TestInteger::encode::<Uper>(&TestInteger(255), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,0,0,0,0,0,1,0, 0,0,0,0,0,0,0,1, 0,0,0,0,0,0,0,0]
        );
        assert!(TestInteger::encode::<Uper>(&TestInteger(-2), bitvec![u8, Msb0;]).is_err())
    }

    #[test]
    fn encodes_unconstrained_integer() {
        asn1_internal_tests!("TestInteger ::= INTEGER");
        assert_eq!(
            TestInteger::encode::<Uper>(&TestInteger(4096), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,0,0,0,0,0,1,0, 0,0,0,1,0,0,0,0, 0,0,0,0,0,0,0,0]
        );
    }

    #[test]
    fn encodes_downwards_unconstrained_integer() {
        asn1_internal_tests!("TestInteger ::= INTEGER(MIN..65535)");
        assert_eq!(
            TestInteger::encode::<Uper>(&TestInteger(127), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,0,0,0,0,0,0,1, 0,1,1,1,1,1,1,1]
        );
        assert_eq!(
            TestInteger::encode::<Uper>(&TestInteger(-128), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,0,0,0,0,0,0,1, 1,0,0,0,0,0,0,0]
        );
        assert_eq!(
            TestInteger::encode::<Uper>(&TestInteger(128), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 0,0,0,0,0,0,1,0, 0,0,0,0,0,0,0,0, 1,0,0,0,0,0,0,0]
        );
    }

    #[test]
    fn encodes_boolean() {
        asn1_internal_tests!("TestBool ::= BOOLEAN");
        assert_eq!(
            TestBool::encode::<Uper>(&TestBool(true), bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0; 1]
        );
        assert_eq!(
          TestBool::encode::<Uper>(&TestBool(false), bitvec![u8, Msb0;]).unwrap(),
          bitvec![u8, Msb0; 0]
      );
    }

    #[test]
    fn encodes_null() {
        asn1_internal_tests!("TestNull ::= NULL");
        assert_eq!(
            TestNull::encode::<Uper>(&TestNull, bitvec![u8, Msb0;]).unwrap(),
            bitvec![u8, Msb0;]
        );
    }
}

use bitvec::{vec::BitVec, prelude::Msb0};

use crate::Encoder;

use super::Uper;

type BitOut = BitVec<u8, Msb0>;

impl Encoder<u8, BitOut> for Uper {
    fn encode_integer<O>(
        _integer: asnr_grammar::types::Integer,
    ) -> Result<alloc::boxed::Box<dyn FnMut(BitOut,O) -> Result<BitOut, crate::error::EncodingError>>, crate::error::EncodingError>
    where
        O: num::Integer + num::FromPrimitive + Copy {
        todo!()
    }
}
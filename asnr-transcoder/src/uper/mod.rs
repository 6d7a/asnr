use bitvec::prelude::Msb0;
use bitvec_nom::BSlice;

mod decoder;
mod encoder;
mod per_visible;

pub struct Uper;

pub(crate) type BitIn<'a> = BSlice<'a, u8, Msb0>;

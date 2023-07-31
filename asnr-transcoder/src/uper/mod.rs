use bitvec::prelude::Msb0;
use bitvec_nom::BSlice;

mod decoder;
mod encoder;
mod per_visible;

pub struct Uper;

pub(crate) type BitIn<'a> = BSlice<'a, u8, Msb0>;
pub(crate) type AsBytesDummy = [u8; 0];

pub(crate) fn bit_length(min: i128, max: i128) -> usize {
    let number_of_values = max - min + 1;
    let mut power = 0;
    while number_of_values > 2_i128.pow(power) {
        power += 1;
    }
    power as usize
}

#[cfg(test)]
mod tests {
    use crate::uper::bit_length;

    #[test]
    fn computes_bit_size() {
        assert_eq!(bit_length(1, 1), 0);
        assert_eq!(bit_length(-1, 0), 1);
        assert_eq!(bit_length(3, 6), 2);
        assert_eq!(bit_length(4000, 4255), 8);
        assert_eq!(bit_length(4000, 4256), 9);
        assert_eq!(bit_length(0, 32000), 15);
        assert_eq!(bit_length(0, 65538), 17);
        assert_eq!(bit_length(-1, 127), 8);
    }
}
use bitvec::{bitvec, prelude::Msb0, vec::BitVec, view::BitView};
use bitvec_nom::BSlice;

use alloc::{string::String, vec::Vec};

use crate::{
    error::{DecodingError, EncodingError},
    Decode, Encode,
};

mod decoder;
mod encoder;
mod per_visible;

pub struct Uper;

impl Uper {
    pub fn decode<'a, T: Decode<'a, BitIn<'a>>>(
        input: &'a [u8],
    ) -> Result<T, DecodingError<BitIn>> {
        T::decode::<Uper>(BitIn::from(input.view_bits::<Msb0>())).map(|(_, res)| res)
    }

    pub fn encode<'a, T: Encode<u8, BitOut>>(input: T) -> Result<Vec<u8>, EncodingError> {
        T::encode::<Uper>(input, bitvec![u8, Msb0;]).map(|mut bitvec| {
            bitvec.set_uninitialized(false);
            bitvec.into_vec()
        })
    }
}

pub type BitIn<'a> = BSlice<'a, u8, Msb0>;
pub type BitOut = BitVec<u8, Msb0>;
pub(crate) type AsBytesDummy = [u8; 0];

const RUST_KEYWORDS: [&'static str; 38] = [
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while",
];

pub(crate) fn bit_length(min: i128, max: i128) -> usize {
    let number_of_values = max - min + 1;
    let mut power = 0;
    while number_of_values > 2_i128.pow(power) {
        power += 1;
    }
    power as usize
}

pub fn rustify_name(name: &String) -> String {
    let name = name.replace("-", "_");
    if RUST_KEYWORDS.contains(&name.as_str()) {
        String::from("r_") + &name
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use asnr_compiler_derive::asn1_internal_tests;

    use crate::uper::{bit_length, Uper};

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
        assert_eq!(bit_length(-900000000, 900000001), 31);
    }

    #[test]
    fn encodes_as_decodes_integer() {
        asn1_internal_tests!(r#"Int-1 ::= INTEGER
            Int-2 ::= INTEGER(42)
            Int-3 ::= INTEGER(-1..65355)
            Int-4 ::= INTEGER(23..MAX)
            Int-5 ::= INTEGER(20,...)
            Int-6 ::= INTEGER(1..24,...)"#);

        assert_eq!(42, Uper::decode::<Int_1>(&Uper::encode(Int_1(42)).unwrap()).unwrap().0);
        assert_eq!(42, Uper::decode::<Int_2>(&Uper::encode(Int_2(42)).unwrap()).unwrap().0);
        assert_eq!(42, Uper::decode::<Int_3>(&Uper::encode(Int_3(42)).unwrap()).unwrap().0);
        assert_eq!(42, Uper::decode::<Int_4>(&Uper::encode(Int_4(42)).unwrap()).unwrap().0);
        assert_eq!(87000, Uper::decode::<Int_5>(&Uper::encode(Int_5(87000)).unwrap()).unwrap().0);
        assert_eq!(42, Uper::decode::<Int_6>(&Uper::encode(Int_6(42)).unwrap()).unwrap().0);
    }
}

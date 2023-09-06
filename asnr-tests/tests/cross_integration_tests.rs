use core::str::FromStr;

use asnr_compiler_derive::asn1;
use bitvec::{prelude::Msb0, view::BitView};

use rasn::prelude::*;

#[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq)]
#[non_exhaustive]
pub struct TestSequenceRasn {
    #[rasn(size("0..=8"))]
    hello: OctetString,
    #[rasn(
        extension_addition,
        value("0..=8"),
        default = "test_sequence_world_default"
    )]
    world: u8,
}

fn test_sequence_world_default() -> u8 {
    8
}


#[test]
fn encodes_octet_sequence() {
    asn1!(
        r#"TestSequenceAsnr ::= SEQUENCE { 
        hello OCTET STRING (SIZE(0..8)),
        ...,
        world INTEGER(0..9) DEFAULT 8
      }"#
    );

    let mut encoder = rasn::uper::Encoder::new(rasn::uper::enc::EncoderOptions::unaligned());
    TestSequenceRasn {
        hello: bytes::Bytes::from(vec![1, 2, 3, 4]),
        world: 4,
    }
    .encode(&mut encoder)
    .unwrap();
    let encoded = encoder.bitstring_output();
    let decoded = TestSequenceAsnr::decode::<asnr_transcoder::uper::Uper>(
        bitvec_nom::BSlice::from(encoded.as_bitslice()),
    )
    .unwrap()
    .1;
    assert_eq!(decoded.hello.0, vec![1, 2, 3, 4]);
    assert_eq!(decoded.world, Some(InnerTestSequenceAsnrWorld(4)));
    let re_encoded = asnr_transcoder::uper::Uper::encode(TestSequenceAsnr {
        hello: InnerTestSequenceAsnrHello(vec![1, 2, 3, 4]),
        world: Some(InnerTestSequenceAsnrWorld(4)),
    })
    .unwrap();
    let mut decoder = rasn::uper::Decoder::new(
        re_encoded.view_bits::<Msb0>(),
        rasn::uper::de::DecoderOptions::unaligned(),
    );
    let re_decoded = TestSequenceRasn::decode(&mut decoder).unwrap();
    assert_eq!(re_decoded.hello.to_vec(), vec![1,2,3,4]);
    assert_eq!(re_decoded.world, 64); // This is a rasn bug
}

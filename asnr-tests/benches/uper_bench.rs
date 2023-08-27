use std::num::ParseIntError;

use asnr_transcoder::uper::Uper;
use asnr_tests::asn1::v2x::{ItsPduHeader, CAM};
use criterion::{BenchmarkId, criterion_group, criterion_main, Criterion};
use asnr_compiler_derive::asn1;
use bitvec::prelude::*;
use bitvec_nom::BSlice;

fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
  (0..s.len())
      .step_by(2)
      .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
      .collect()
}

fn decode_small_sequence(c: &mut Criterion) {
  asn1!("TestSequence ::= SEQUENCE
  {item-code INTEGER (0..254),
  item-name IA5String (SIZE (3..10))OPTIONAL,
  ...,
  urgency ENUMERATED {normal, high} DEFAULT normal }");
  let input = BSlice::from(bits![static u8, Msb0; 
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
    1,0,0,0,0,0,0,0]);
  
  c.bench_with_input(BenchmarkId::new("Test Sequence", ""), &input, |b, i| {
    b.iter(||TestSequence::decode::<Uper>(*i).unwrap());
  });

  let header = decode_hex("0202de140ce5c7c0405ab23d82ce2781e9a278274bc633fa54587ca0a27e8302968a9733ff82001a103fe0143980106e0075801158ce0002f03adc08c4c800015781d620469633800abc0edb0239319c0055e075081185900002af03a0c0912c800016781c9e0565640000c3c0e0902dbb19c006de058d810218ce0035f0155c0006c67000df808d5fde662700073c0476fd67319c0058604137d31589c006dc").unwrap();

  c.bench_with_input(BenchmarkId::new("ItsPduHeader", ""), &header, |b, i| {
    b.iter(|| Uper::decode::<ItsPduHeader>(i).unwrap());
  });

  let cam = decode_hex("0202de140ce5c7c0405ab23d82ce2781e9a278274bc633fa54587ca0a27e8302968a9733ff82001a103fe0143980106e0075801158ce0002f03adc08c4c800015781d620469633800abc0edb0239319c0055e075081185900002af03a0c0912c800016781c9e0565640000c3c0e0902dbb19c006de058d810218ce0035f0155c0006c67000df808d5fde662700073c0476fd67319c0058604137d31589c006dc").unwrap();

  c.bench_with_input(BenchmarkId::new("CAM", ""), &cam, |b, i| {
    b.iter(|| Uper::decode::<CAM>(i).unwrap());
  });
}

criterion_group!(benches, decode_small_sequence);
criterion_main!(benches);
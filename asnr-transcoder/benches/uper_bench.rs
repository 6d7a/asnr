use asnr_transcoder::uper::Uper;
use criterion::{BenchmarkId, criterion_group, criterion_main, Criterion};
use asnr_compiler_derive::asn1;
use bitvec::prelude::*;
use bitvec_nom::BSlice;

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
  
  c.bench_with_input(BenchmarkId::new("Test Sequence", "asas"), &input, |b, i| {
    b.iter(||TestSequence::decode::<Uper>(*i).unwrap());
  });
}

criterion_group!(benches, decode_small_sequence);
criterion_main!(benches);
#[macro_use]
extern crate asnr_compiler_derive;

use asnr_compiler_derive::asn1;

#[test]
fn generates_inline_asn1() {
    asn1!("My-int ::= INTEGER (0..24)");
    assert_eq!(My_int::default(), My_int(0))
}

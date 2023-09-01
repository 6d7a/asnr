# ASNR
ASNR - an ASN1 compiler for Rust that makes your skin tingle.

### Motivation

I've been using the excellent [asn1c](https://github.com/vlm/asn1c) with [a thin Rust wrapper](https://sjames.github.io/articles/2020-04-26-rust-ffi-asn1-codec/) parser in various situations. Unfortunately, on WebAssembly targets, this set-up becomes close to impossible to handle. Since there is not much out there in terms of ASN1 compilers for Rust and because I always enjoy a good read of [John Larmouth's beloved classic](https://www.oss.com/asn1/resources/books-whitepapers-pubs/larmouth-asn1-book.pdf), I started this project.

# ASNR Compiler
The ASNR compiler is a parser combinator that parses ASN1 specifications and outputs encoding-rule-agnotic rust representations of the ASN1 data elements. ASNR heavily relies on the great library [nom](https://docs.rs/nom/latest/nom/) for its basic parsers. It is designed to be 
encoding-rule-agnostic, so that its output can be used regardless whether the actual encoding follows
BER, DER, CER, PER, XER, or whatever exotic *ERs still out there.

## Example
In order to compile ASN1 in your build process, invoke the ASNR compiler in your [`build.rs` build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html).
```rust
// build.rs build script
use std::path::PathBuf;
use asnr_compiler::Asnr;

fn main() {
  // Initialize the compiler
  match Asnr::new()
    // add a single ASN1 source file
    .add_asn_by_path(PathBuf::from("spec_1.asn"))
    // add several ASN1 source files
    .add_asn_sources_by_path(vec![
        PathBuf::from("spec_2.asn"),
        PathBuf::from("spec_3.asn"),
    ].iter())
    // set an output path for the generated rust code
    .set_output_path(PathBuf::from("./asn/generated.rs"))
    // you may also compile literal ASN1 snippets
    .add_asn_literal("My-test-integer ::= INTEGER (1..128)")
    // optionally choose to support `no_std`
    .no_std(true)
    .compile() {
    Ok(warnings /* Vec<Box<dyn Error>> */) => { /* handle compilation warnings */ }
    Err(error /* Box<dyn Error> */) => { /* handle unrecoverable compilation error */ }
  }
}
```

See also the `asnr-compiler-derive` crate, that provides shorthand macros for inline ASN1 support.
```rust
use asnr_compiler_derive::asn1;

asn1! { 
  r#"
    HashAlgorithm ::= ENUMERATED { 
      sha256,
      ...,
      sha384
    }
  "#
}

// or

asn1_no_std! {
  r#"
    ServiceSpecificPermissions ::= CHOICE {
      opaque    OCTET STRING (SIZE(0..MAX)),
      ...,
      extension BOOLEAN DEFAULT TRUE
    }
  "#
}
```

# ASNR Transcoder
The transcoder crate handles the actual encoding and decoding of data at runtime.
It aims to be suitable for `no_std` environments and `wasm-unknown` targets.
For a start, the asnr transcoder will provide support for UPER encoding rules, 
but transcoding can be easily customized by implementing the crate's `Encoder` and `Decoder` traits.

The ASNR transcoder de- and encodes messages by composing functions that handle the
de-/encoding of generic ASN1 types like SEQUENCEs or INTEGERs. In the current implementation,
that choice has led to a lot of boxing and unboxing, but I hope to find a more efficient solution
in the future. The advantage of this design is that authors of custom encoders and decoders have
pretty much all of the information concerning the data element as it's specified in an 
ASN1 specification, including constraints, even comments up to a certain degree. 

## Usage
Let's consider the following ASN1 Sequence:
```asn1
ExampleSequence ::= SEQUENCE {
  member-1 IA5String (SIZE (1..24)),
  member-2 INTEGER (0..15),
  ...,
  extension BOOLEAN OPTIONAL
}
```

```rust
use asnr_transcoder::uper::Uper;
/// import your generated ASN1 representations
use my_asn_spec::*;

fn decode_example_sequence(binary: &[u8]) -> ExampleSequence {
  Uper::decode(binary).unwrap()
}

fn encode_example_sequence() -> Vec<u8> {
  let example_sequence = ExampleSequence {
    // ASN1-built-in types are represented as new types within SEQUENCEs
    member_1: InnerExampleSequenceMember1("Hello, World!".into()),
    member_2: InnerExampleSequenceMember2(8),
    extension: None
  };
  Uper::encode(example_sequence).unwrap()
}
```
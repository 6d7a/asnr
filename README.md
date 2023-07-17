# ASNR
_Disclaimer: This is a pet project and heavy WIP. I advise looking into [`rasn`](https://github.com/XAMPPRocky/rasn) for something more mature._

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
  match Asnr::compiler()                                    // Initialize the compiler
    .add_asn_source(PathBuf::from("spec_1.asn"))            // add a single ASN1 source file
    .add_asn_sources(vec![                                  // add several ASN1 source files
        PathBuf::from("spec_2.asn"),
        PathBuf::from("spec_3.asn"),
    ])
    .set_output_path(PathBuf::from("./asn/generated.rs"))   // Set an output path for the generated rust code
    .compile() {
    Ok(warnings: Vec<Box<dyn Error>>) -> { /* handle compilation warnings */ }
    Err(error: Box<dyn Error>) -> { /* handle unrecoverable compilation error */ }
  }
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
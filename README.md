# ASNR
_Disclaimer: This is a pet project and heavy WIP. I advise against using this until a stable cargo crate has been released._

ASNR - an ASN1 compiler for Rust that makes your skin tingle.

### Motivation

I've been using the excellent [asn1c](https://github.com/vlm/asn1c) with [a thin Rust wrapper](https://sjames.github.io/articles/2020-04-26-rust-ffi-asn1-codec/) parser in various situations. Unfortunately, on WebAssembly targets, this set-up becomes close to impossible to handle. Since there is not much out there in terms of ASN1 compilers for Rust and because I always enjoy a good read of [John Larmouth's beloved classic](https://www.oss.com/asn1/resources/books-whitepapers-pubs/larmouth-asn1-book.pdf), I started this project.

# ASNR Compiler
The ASNR compiler is a parser combinator that parses ASN1 specifications and outputs encoding-rule-agnotic rust representations of the ASN1 data elements. ASNR heavily relies on the great library [nom](https://docs.rs/nom/latest/nom/) for its basic parsers. It is designed to be 
encoding-rule-agnostic, so that its output can be used regardless whether the actual encoding follows
BER, DER, CER, PER, XER, or whatever exotic *ERs still out there.

# ASNR Transcoder
The transcoder crate handles the actual encoding and decoding of data at runtime.
It aims to be suitable for `no_std` environments and `wasm-unknown` targets.
For a start, the asnr transcoder will provide support for UPER encoding rules, 
but transcoding can be easily customized by implementing the crate's `Encoder` and `Decoder` traits.
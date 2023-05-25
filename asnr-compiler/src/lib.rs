//! The `asnr-compiler` library is a parser combinator that parses ASN1 specifications and outputs
//! encoding-rule-agnotic rust representations of the ASN1 data elements. ASNR heavily relies on the great
//! library [nom](https://docs.rs/nom/latest/nom/) for its basic parsers. It is designed to be
//! encoding-rule-agnostic, so that its output can be used regardless whether the actual encoding follows
//! BER, DER, CER, PER, XER, or whatever exotic *ERs still out there.
//!
//! ## Example
//!
//! In order to compile ASN1 in your build process, invoke the ASNR compiler in your [`build.rs` build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html).
//!
//! ```rust
//! // build.rs build script
//! use std::path::PathBuf;
//! use asnr_compiler::Asnr;
//! 
//! fn main() {
//!   match Asnr::compiler()                                    // Initialize the compiler
//!     .add_asn_source(PathBuf::from("spec_1.asn"))            // add a single ASN1 source file
//!     .add_asn_sources(vec![                                  // add several ASN1 source files
//!         PathBuf::from("spec_2.asn"),
//!         PathBuf::from("spec_3.asn"),
//!     ])
//!     .set_output_path(PathBuf::from("./asn/generated.rs"))   // Set an output path for the generated rust code
//!     .compile() {
//!     Ok(warnings: Vec<Box<dyn Error>>) -> { /* handle compilation warnings */ }
//!     Err(error: Box<dyn Error>) -> { /* handle unrecoverable compilation error */ }
//!   }
//! }
//! ```
mod generator;
mod parser;
mod validator;

use std::{
    env::var,
    error::Error,
    fs::{self, read_to_string},
    path::PathBuf,
};

use asnr_grammar::ToplevelDeclaration;
use generator::{error::GeneratorError, generate, GENERATED_RUST_IMPORTS};
use parser::asn_string;
use validator::{error::ValidatorError, Validate};

/// The ASNR compiler
#[derive(Debug, PartialEq)]
pub struct Asnr {
    sources: Vec<PathBuf>,
}

impl Asnr {
    /// Provides a Builder for building ASNR compiler commands
    pub fn compiler() -> AsnrCompiler {
        AsnrCompiler::default()
    }
}

#[derive(Default)]
pub struct AsnrCompiler {
    sources: Vec<PathBuf>,
    output_path: PathBuf,
}

impl From<Vec<PathBuf>> for AsnrCompiler {
    fn from(value: Vec<PathBuf>) -> Self {
        AsnrCompiler {
            sources: value,
            output_path: default_output_dir(),
        }
    }
}

impl From<PathBuf> for AsnrCompiler {
    fn from(value: PathBuf) -> Self {
        AsnrCompiler {
            sources: vec![value],
            output_path: default_output_dir(),
        }
    }
}

impl AsnrCompiler {
    /// Add an ASN1 source to the compile command
    /// * `path_to_source` - path to ASN1 file to include
    pub fn add_asn_source(mut self, path_to_source: PathBuf) -> AsnrCompiler {
        self.sources.push(path_to_source);
        self
    }

    /// Add several ASN1 sources to the compile command
    /// * `path_to_source` - vector of paths to the ASN1 files to be included
    pub fn add_asn_sources(mut self, paths_to_sources: Vec<PathBuf>) -> AsnrCompiler {
        self.sources.extend(paths_to_sources.into_iter());
        self
    }

    /// Set the output path for the generated rust representation.
    /// The ASNR's output directory defaults in sequence to env(`OUT_DIR`),
    /// std::env::current_dir(), and "."
    /// * `output_path` - path to an output file or directory, if path indicates
    ///                   a directory, the output file is named `asnr_generated.rs`
    pub fn set_output_path(mut self, mut output_path: PathBuf) -> AsnrCompiler {
        self.output_path = if output_path.is_dir() {
            output_path.set_file_name("asnr_generated.rs");
            output_path
        } else {
            output_path
        };
        self
    }

    /// Runs the ASNR compiler command.
    /// Returns a Result wrapping a compilation result:
    /// * _Ok_  - Vector of warnings raised during the compilation
    /// * _Err_ - Unrecoverable error, no rust representations were generated
    pub fn compile(self) -> Result<Vec<Box<dyn Error>>, Box<dyn Error>> {
        let mut result = String::from(GENERATED_RUST_IMPORTS);
        for src in self.sources {
            let toplevel_declarations = asn_string(&read_to_string(src)?)?;
            let (valid_tlds, validator_errors) = toplevel_declarations.into_iter().fold(
                (
                    Vec::<ToplevelDeclaration>::new(),
                    Vec::<ValidatorError>::new(),
                ),
                |(mut tlds, mut errors), tld| {
                    match tld.validate() {
                        Ok(_) => tlds.push(tld),
                        Err(e) => errors.push(e),
                    }
                    (tlds, errors)
                },
            );
            let (generated, generator_errors) = valid_tlds.into_iter().fold(
                (String::new(), Vec::<GeneratorError>::new()),
                |(mut rust, mut errors), tld| {
                    match generate(tld, None) {
                        Ok(r) => {
                            rust = rust + &r + "\n";
                        }
                        Err(e) => errors.push(e),
                    }
                    (rust, errors)
                },
            );
            result += &generated;
        }
        fs::write("test.rs", result)?;
        Ok(())
    }
}

fn default_output_dir() -> PathBuf {
    if let Ok(p) = var("OUT_DIR") {
        PathBuf::from(p + "/asnr_generated.rs")
    } else if let Ok(mut d) = std::env::current_dir() {
        d.set_file_name("asnr_generated.rs");
        d
    } else {
        PathBuf::from("./asnr_generated.rs")
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::Asnr;

    #[test]
    fn compiles_a_simple_spec() {
        Asnr::compiler()
            .add_asn_source(PathBuf::from("spec_1.asn"))
            .add_asn_sources(vec![
                PathBuf::from("spec_2.asn"),
                PathBuf::from("spec_3.asn"),
            ])
            .set_output_path(PathBuf::from("./asn/generated.rs"))
            .compile();
    }
}

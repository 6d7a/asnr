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
//!     .add_asn_by_path(PathBuf::from("spec_1.asn"))            // add a single ASN1 source file
//!     .add_asn_sources_by_path(vec![                                  // add several ASN1 source files
//!         PathBuf::from("spec_2.asn"),
//!         PathBuf::from("spec_3.asn"),
//!     ])
//!     .set_output_path(PathBuf::from("./asn/generated.rs"))   // Set an output path for the generated rust code
//!     .compile() {
//!     Ok(warnings /* Vec<Box<dyn Error>> */) => { /* handle compilation warnings */ }
//!     Err(error /* Box<dyn Error> */) => { /* handle unrecoverable compilation error */ }
//!   }
//! }
//! ```
mod generator;
mod parser;
mod validator;

use std::{
    env::{self, var},
    error::Error,
    fs::{self, read_to_string},
    io::{self, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

use asnr_grammar::ToplevelDeclaration;
use generator::{generate, templates::asnr::template::imports_and_generic_types, Generator};
use parser::asn_spec;
use validator::Validator;

/// The ASNR compiler
#[derive(Debug, PartialEq)]
pub struct Asnr {
    sources: Vec<PathBuf>,
}

#[derive(Debug, PartialEq)]
enum AsnSource {
    Path(PathBuf),
    Literal(String),
}

impl Asnr {
    /// Provides a Builder for building ASNR compiler commands
    pub fn compiler() -> AsnrCompiler {
        AsnrCompiler::default()
    }
}

#[derive(Debug, PartialEq, Default)]
pub enum Framework {
  #[default] Asnr,
  Rasn
}

#[derive(Default)]
pub struct AsnrCompiler {
    sources: Vec<AsnSource>,
    output_path: PathBuf,
    no_std: bool,
    framework: Framework
}

impl From<Vec<PathBuf>> for AsnrCompiler {
    fn from(value: Vec<PathBuf>) -> Self {
        AsnrCompiler {
            sources: value.into_iter().map(|p| AsnSource::Path(p)).collect(),
            output_path: default_output_dir(),
            ..Default::default()
        }
    }
}

impl From<PathBuf> for AsnrCompiler {
    fn from(value: PathBuf) -> Self {
        AsnrCompiler {
            sources: vec![AsnSource::Path(value)],
            output_path: default_output_dir(),
            ..Default::default()
        }
    }
}

impl From<&str> for AsnrCompiler {
    fn from(value: &str) -> Self {
        AsnrCompiler {
            sources: vec![AsnSource::Literal(value.into())],
            output_path: default_output_dir(),
            ..Default::default()
        }
    }
}

impl AsnrCompiler {
    /// Add an ASN1 source to the compile command by path
    /// * `path_to_source` - path to ASN1 file to include
    pub fn add_asn_by_path(mut self, path_to_source: PathBuf) -> AsnrCompiler {
        self.sources.push(AsnSource::Path(path_to_source));
        self
    }

    /// Add several ASN1 sources by path to the compile command
    /// * `path_to_source` - vector of paths to the ASN1 files to be included
    pub fn add_asn_sources_by_path(mut self, paths_to_sources: Vec<PathBuf>) -> AsnrCompiler {
        self.sources
            .extend(paths_to_sources.into_iter().map(|p| AsnSource::Path(p)));
        self
    }

    /// Define, which framework should be used for representing ASN1 in Rust.
    /// * Framework::Asnr uses the `asnr-grammar` and `asnr-trancoder` crates
    /// * Framework::Rasn uses [rasn](https://github.com/XAMPPRocky/rasn)
    pub fn framework(mut self, framework: Framework) -> AsnrCompiler {
      self.framework = framework;
      self
    }

    /// Add a literal ASN1 source to the compile command
    /// * `literal` - literal ASN1 statement to include
    /// ```rust
    /// # use asnr_compiler::Asnr;
    /// Asnr::compiler().add_asn_literal("My-test-integer ::= INTEGER (1..128)").compile_to_string();
    /// ```
    pub fn add_asn_literal(mut self, literal: &str) -> AsnrCompiler {
        self.sources.push(AsnSource::Literal(literal.into()));
        self
    }

    /// Generate Rust representations compatible with an environment without the standard library
    /// * `is_supporting` - whether the generated Rust should comply with no_std
    pub fn no_std(mut self, is_supporting: bool) -> AsnrCompiler {
        self.no_std = is_supporting;
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

    /// Runs the ASNR compiler command and returns stringified Rust.
    /// Returns a Result wrapping a compilation result:
    /// * _Ok_  - tuple containing the stringified Rust representation of the ASN1 spec as well as a vector of warnings raised during the compilation
    /// * _Err_ - Unrecoverable error, no rust representations were generated
    pub fn compile_to_string(self) -> Result<(String, Vec<Box<dyn Error>>), Box<dyn Error>> {
        self.internal_compile(false)
    }

    fn internal_compile(
        &self,
        include_clippy_allows: bool,
    ) -> Result<(String, Vec<Box<dyn Error>>), Box<dyn Error>> {
        let mut result = imports_and_generic_types(None, self.no_std, include_clippy_allows);
        let mut warnings = Vec::<Box<dyn Error>>::new();
        let mut modules: Vec<ToplevelDeclaration> = vec![];
        for src in &self.sources {
            let stringified_src = match src {
                AsnSource::Path(p) => read_to_string(p)?,
                AsnSource::Literal(l) => l.clone(),
            };
            modules.append(
                &mut asn_spec(&stringified_src)?
                    .into_iter()
                    .flat_map(|(_, tld)| tld)
                    .collect(),
            );
        }
        let (valid_tlds, mut validator_errors) = Validator::new(modules).validate()?;
        let (generated, mut generator_errors) = valid_tlds.into_iter().fold(
            (String::new(), Vec::<Box<dyn Error>>::new()),
            |(mut rust, mut errors), tld| {
                match generate(&self.framework, tld, None) {
                    Ok(r) => {
                        rust = rust + &r + "\n";
                    }
                    Err(e) => errors.push(Box::new(e)),
                }
                (rust, errors)
            },
        );
        result += &generated;
        warnings.append(&mut validator_errors);
        warnings.append(&mut generator_errors);

        result = format_bindings(&result).unwrap_or(result);

        Ok((result, warnings))
    }

    /// Runs the ASNR compiler command.
    /// Returns a Result wrapping a compilation result:
    /// * _Ok_  - Vector of warnings raised during the compilation
    /// * _Err_ - Unrecoverable error, no rust representations were generated
    pub fn compile(self) -> Result<Vec<Box<dyn Error>>, Box<dyn Error>> {
        let (result, warnings) = self.internal_compile(true)?;

        fs::write(self.output_path, result)?;

        Ok(warnings)
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

fn format_bindings(bindings: &String) -> Result<String, Box<dyn Error>> {
    let mut rustfmt = PathBuf::from(env::var("CARGO_HOME")?);
    rustfmt.push("bin/rustfmt");
    let mut cmd = Command::new(&*rustfmt);

    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());

    let mut child = cmd.spawn()?;
    let mut child_stdin = child.stdin.take().unwrap();
    let mut child_stdout = child.stdout.take().unwrap();

    // Write to stdin in a new thread, so that we can read from stdout on this
    // thread. This keeps the child from blocking on writing to its stdout which
    // might block us from writing to its stdin.
    let bindings = bindings.to_owned();
    let stdin_handle = ::std::thread::spawn(move || {
        let _ = child_stdin.write_all(bindings.as_bytes());
        bindings
    });

    let mut output = vec![];
    io::copy(&mut child_stdout, &mut output)?;

    let status = child.wait()?;
    let bindings = stdin_handle.join().expect(
        "The thread writing to rustfmt's stdin doesn't do \
             anything that could panic",
    );

    match String::from_utf8(output) {
        Ok(bindings) => match status.code() {
            Some(0) => Ok(bindings),
            Some(2) => Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Rustfmt parsing errors.".to_string(),
            ))),
            Some(3) => Ok(bindings),
            _ => Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Internal rustfmt error".to_string(),
            ))),
        },
        _ => Ok(bindings.into()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::Asnr;

    #[test]
    fn compiles_a_simple_spec() {
        println!(
            "{:#?}",
            Asnr::compiler()
                .no_std(true)
                // .add_asn_by_path(PathBuf::from("test_asn1/AddGrpC.asn"))
                // .add_asn_by_path(PathBuf::from("test_asn1/ETSI-ITS-CDD.asn"))
                .add_asn_by_path(PathBuf::from("test_asn1/REGION.asn"))
                //.add_asn_by_path(PathBuf::from("test_asn1/v2x.asn"))
                //     .add_asn_by_path(PathBuf::from(
                //         "test_asn1/CPM-OriginatingStationContainers.asn"
                //     ))
                //     .add_asn_by_path(PathBuf::from("test_asn1/CPM-PerceivedObjectContainer.asn"))
                //     .add_asn_by_path(PathBuf::from("test_asn1/CPM-PerceptionRegionContainer.asn"))
                //     .add_asn_by_path(PathBuf::from(
                //         "test_asn1/CPM-SensorInformationContainer.asn"
                //     ))
                //    .add_asn_source(PathBuf::from("test_asn1/CPM-PDU-Descriptions.asn"))
                .set_output_path(PathBuf::from("../asnr-transcoder/src/generated.rs"))
                .compile()
                .unwrap()
        )
    }
}

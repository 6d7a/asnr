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
//!   // Initialize the compiler
//!   match Asnr::new()
//!     // add a single ASN1 source file
//!     .add_asn_by_path(PathBuf::from("spec_1.asn"))
//!     // add several ASN1 source files
//!     .add_asn_sources_by_path(vec![
//!         PathBuf::from("spec_2.asn"),
//!         PathBuf::from("spec_3.asn"),
//!     ].iter())
//!     // set an output path for the generated rust code
//!     .set_output_path(PathBuf::from("./asn/generated.rs"))
//!     // you may also compile literal ASN1 snippets
//!     .add_asn_literal("My-test-integer ::= INTEGER (1..128)")
//!     // optionally choose to support `no_std`
//!     .no_std(true)
//!     .compile() {
//!     Ok(warnings /* Vec<Box<dyn Error>> */) => { /* handle compilation warnings */ }
//!     Err(error /* Box<dyn Error> */) => { /* handle unrecoverable compilation error */ }
//!   }
//! }
//! ```
mod generator;
mod parser;
mod validator;
pub(crate) mod utils;

use std::{
    env::{self},
    error::Error,
    fs::{self, read_to_string},
    io::{self, Write},
    path::PathBuf,
    process::{Command, Stdio},
    vec,
};

use asnr_grammar::ToplevelDeclaration;
use generator::{generate, template::imports_and_generic_types};
use parser::asn_spec;
use validator::Validator;

/// The ASNR compiler
#[derive(Debug, PartialEq)]
pub struct Asnr<S: AsnrState> {
    state: S,
}

/// Typestate representing compiler with missing parameters
pub struct AsnrMissingParams {
    no_std: bool,
}

impl Default for AsnrMissingParams {
    fn default() -> Self {
        Self { no_std: false }
    }
}

/// Typestate representing compiler that is ready to compile
pub struct AsnrCompileReady {
    sources: Vec<AsnSource>,
    output_path: PathBuf,
    no_std: bool,
}

/// Typestate representing compiler that has the output path set, but is missing ASN1 sources
pub struct AsnrOutputSet {
    output_path: PathBuf,
    no_std: bool,
}

/// Typestate representing compiler that knows about ASN1 sources, but doesn't have an output path set
pub struct AsnrSourcesSet {
    sources: Vec<AsnSource>,
    no_std: bool,
}

/// State of the Asnr compiler
pub trait AsnrState {}
impl AsnrState for AsnrCompileReady {}
impl AsnrState for AsnrOutputSet {}
impl AsnrState for AsnrSourcesSet {}
impl AsnrState for AsnrMissingParams {}

#[derive(Debug, PartialEq)]
enum AsnSource {
    Path(PathBuf),
    Literal(String),
}

impl Asnr<AsnrMissingParams> {
    /// Provides a Builder for building ASNR compiler commands
    pub fn new() -> Asnr<AsnrMissingParams> {
        Asnr {
            state: AsnrMissingParams::default(),
        }
    }

    /// Add an ASN1 source to the compile command by path
    /// * `path_to_source` - path to ASN1 file to include
    pub fn add_asn_by_path(self, path_to_source: impl Into<PathBuf>) -> Asnr<AsnrSourcesSet> {
        Asnr {
            state: AsnrSourcesSet {
                sources: vec![AsnSource::Path(path_to_source.into())],
                no_std: self.state.no_std,
            },
        }
    }

    /// Generate Rust representations compatible with an environment without the standard library
    /// * `is_supporting` - whether the generated Rust should comply with no_std
    pub fn no_std(self, is_supporting: bool) -> Self {
        Self {
            state: AsnrMissingParams {
                no_std: is_supporting,
            },
        }
    }

    /// Add several ASN1 sources by path to the compile command
    /// * `path_to_source` - iterator of paths to the ASN1 files to be included
    pub fn add_asn_sources_by_path(
        self,
        paths_to_sources: impl Iterator<Item = impl Into<PathBuf>>,
    ) -> Asnr<AsnrSourcesSet> {
        Asnr {
            state: AsnrSourcesSet {
                sources: paths_to_sources
                    .map(|p| AsnSource::Path(p.into()))
                    .collect(),
                no_std: self.state.no_std,
            },
        }
    }

    /// Add a literal ASN1 source to the compile command
    /// * `literal` - literal ASN1 statement to include
    /// ```rust
    /// # use asnr_compiler::Asnr;
    /// Asnr::new().add_asn_literal("My-test-integer ::= INTEGER (1..128)").compile_to_string();
    /// ```
    pub fn add_asn_literal(self, literal: impl Into<String>) -> Asnr<AsnrSourcesSet> {
        Asnr {
            state: AsnrSourcesSet {
                sources: vec![AsnSource::Literal(literal.into())],
                no_std: self.state.no_std,
            },
        }
    }

    /// Set the output path for the generated rust representation.
    /// * `output_path` - path to an output file or directory, if path indicates
    ///                   a directory, the output file is named `asnr_generated.rs`
    pub fn set_output_path(self, output_path: impl Into<PathBuf>) -> Asnr<AsnrOutputSet> {
        let mut path: PathBuf = output_path.into();
        if path.is_dir() {
            path.set_file_name("asnr_generated.rs");
        }
        Asnr {
            state: AsnrOutputSet {
                output_path: path,
                no_std: self.state.no_std,
            },
        }
    }
}

impl Asnr<AsnrOutputSet> {
    /// Add an ASN1 source to the compile command by path
    /// * `path_to_source` - path to ASN1 file to include
    pub fn add_asn_by_path(self, path_to_source: impl Into<PathBuf>) -> Asnr<AsnrCompileReady> {
        Asnr {
            state: AsnrCompileReady {
                sources: vec![AsnSource::Path(path_to_source.into())],
                no_std: self.state.no_std,
                output_path: self.state.output_path,
            },
        }
    }

    /// Generate Rust representations compatible with an environment without the standard library
    /// * `is_supporting` - whether the generated Rust should comply with no_std
    pub fn no_std(self, is_supporting: bool) -> Self {
        Self {
            state: AsnrOutputSet {
                output_path: self.state.output_path,
                no_std: is_supporting,
            },
        }
    }

    /// Add several ASN1 sources by path to the compile command
    /// * `path_to_source` - iterator of paths to the ASN1 files to be included
    pub fn add_asn_sources_by_path(
        self,
        paths_to_sources: impl Iterator<Item = impl Into<PathBuf>>,
    ) -> Asnr<AsnrCompileReady> {
        Asnr {
            state: AsnrCompileReady {
                sources: paths_to_sources
                    .map(|p| AsnSource::Path(p.into()))
                    .collect(),
                no_std: self.state.no_std,
                output_path: self.state.output_path,
            },
        }
    }

    /// Add a literal ASN1 source to the compile command
    /// * `literal` - literal ASN1 statement to include
    /// ```rust
    /// # use asnr_compiler::Asnr;
    /// Asnr::new().add_asn_literal("My-test-integer ::= INTEGER (1..128)").compile_to_string();
    /// ```
    pub fn add_asn_literal(self, literal: impl Into<String>) -> Asnr<AsnrCompileReady> {
        Asnr {
            state: AsnrCompileReady {
                sources: vec![AsnSource::Literal(literal.into())],
                no_std: self.state.no_std,
                output_path: self.state.output_path,
            },
        }
    }
}

impl Asnr<AsnrSourcesSet> {
    /// Add an ASN1 source to the compile command by path
    /// * `path_to_source` - path to ASN1 file to include
    pub fn add_asn_by_path(self, path_to_source: impl Into<PathBuf>) -> Asnr<AsnrSourcesSet> {
        let mut sources: Vec<AsnSource> = self.state.sources;
        sources.push(AsnSource::Path(path_to_source.into()));
        Asnr {
            state: AsnrSourcesSet {
                sources,
                no_std: self.state.no_std,
            },
        }
    }

    /// Generate Rust representations compatible with an environment without the standard library
    /// * `is_supporting` - whether the generated Rust should comply with no_std
    pub fn no_std(self, is_supporting: bool) -> Asnr<AsnrSourcesSet> {
        Self {
            state: AsnrSourcesSet {
                sources: self.state.sources,
                no_std: is_supporting,
            },
        }
    }

    /// Add several ASN1 sources by path to the compile command
    /// * `path_to_source` - iterator of paths to the ASN1 files to be included
    pub fn add_asn_sources_by_path(
        self,
        paths_to_sources: impl Iterator<Item = impl Into<PathBuf>>,
    ) -> Asnr<AsnrSourcesSet> {
        let mut sources: Vec<AsnSource> = self.state.sources;
        sources.extend(paths_to_sources.map(|p| AsnSource::Path(p.into())));
        Asnr {
            state: AsnrSourcesSet {
                sources,
                no_std: self.state.no_std,
            },
        }
    }

    /// Add a literal ASN1 source to the compile command
    /// * `literal` - literal ASN1 statement to include
    /// ```rust
    /// # use asnr_compiler::Asnr;
    /// Asnr::new().add_asn_literal("My-test-integer ::= INTEGER (1..128)").compile_to_string();
    /// ```
    pub fn add_asn_literal(self, literal: impl Into<String>) -> Asnr<AsnrSourcesSet> {
        let mut sources: Vec<AsnSource> = self.state.sources;
        sources.push(AsnSource::Literal(literal.into()));
        Asnr {
            state: AsnrSourcesSet {
                sources,
                no_std: self.state.no_std,
            },
        }
    }

    /// Set the output path for the generated rust representation.
    /// * `output_path` - path to an output file or directory, if path indicates
    ///                   a directory, the output file is named `asnr_generated.rs`
    pub fn set_output_path(self, output_path: impl Into<PathBuf>) -> Asnr<AsnrCompileReady> {
        let mut path: PathBuf = output_path.into();
        if path.is_dir() {
            path.set_file_name("asnr_generated.rs");
        }
        Asnr {
            state: AsnrCompileReady {
                sources: self.state.sources,
                output_path: path,
                no_std: self.state.no_std,
            },
        }
    }

    /// Runs the ASNR compiler command and returns stringified Rust.
    /// Returns a Result wrapping a compilation result:
    /// * _Ok_  - tuple containing the stringified Rust representation of the ASN1 spec as well as a vector of warnings raised during the compilation
    /// * _Err_ - Unrecoverable error, no rust representations were generated
    pub fn compile_to_string(self) -> Result<(String, Vec<Box<dyn Error>>), Box<dyn Error>> {
        internal_compile(&self, false)
    }
}

impl Asnr<AsnrCompileReady> {
    /// Add an ASN1 source to the compile command by path
    /// * `path_to_source` - path to ASN1 file to include
    pub fn add_asn_by_path(self, path_to_source: impl Into<PathBuf>) -> Asnr<AsnrCompileReady> {
        let mut sources: Vec<AsnSource> = self.state.sources;
        sources.push(AsnSource::Path(path_to_source.into()));
        Asnr {
            state: AsnrCompileReady {
                output_path: self.state.output_path,
                sources,
                no_std: self.state.no_std,
            },
        }
    }

    /// Generate Rust representations compatible with an environment without the standard library
    /// * `is_supporting` - whether the generated Rust should comply with no_std
    pub fn no_std(self, is_supporting: bool) -> Asnr<AsnrCompileReady> {
        Self {
            state: AsnrCompileReady {
                output_path: self.state.output_path,
                sources: self.state.sources,
                no_std: is_supporting,
            },
        }
    }

    /// Add several ASN1 sources by path to the compile command
    /// * `path_to_source` - iterator of paths to the ASN1 files to be included
    pub fn add_asn_sources_by_path(
        self,
        paths_to_sources: impl Iterator<Item = impl Into<PathBuf>>,
    ) -> Asnr<AsnrCompileReady> {
        let mut sources: Vec<AsnSource> = self.state.sources;
        sources.extend(paths_to_sources.map(|p| AsnSource::Path(p.into())));
        Asnr {
            state: AsnrCompileReady {
                sources,
                output_path: self.state.output_path,
                no_std: self.state.no_std,
            },
        }
    }

    /// Add a literal ASN1 source to the compile command
    /// * `literal` - literal ASN1 statement to include
    /// ```rust
    /// # use asnr_compiler::Asnr;
    /// Asnr::new().add_asn_literal("My-test-integer ::= INTEGER (1..128)").compile_to_string();
    /// ```
    pub fn add_asn_literal(self, literal: impl Into<String>) -> Asnr<AsnrCompileReady> {
        let mut sources: Vec<AsnSource> = self.state.sources;
        sources.push(AsnSource::Literal(literal.into()));
        Asnr {
            state: AsnrCompileReady {
                output_path: self.state.output_path,
                sources,
                no_std: self.state.no_std,
            },
        }
    }

    /// Runs the ASNR compiler command and returns stringified Rust.
    /// Returns a Result wrapping a compilation result:
    /// * _Ok_  - tuple containing the stringified Rust representation of the ASN1 spec as well as a vector of warnings raised during the compilation
    /// * _Err_ - Unrecoverable error, no rust representations were generated
    pub fn compile_to_string(self) -> Result<(String, Vec<Box<dyn Error>>), Box<dyn Error>> {
        internal_compile(&Asnr {
            state: AsnrSourcesSet {
                sources: self.state.sources,
                no_std: self.state.no_std,
            },
        }, false)
    }

    /// Runs the ASNR compiler command.
    /// Returns a Result wrapping a compilation result:
    /// * _Ok_  - Vector of warnings raised during the compilation
    /// * _Err_ - Unrecoverable error, no rust representations were generated
    pub fn compile(self) -> Result<Vec<Box<dyn Error>>, Box<dyn Error>> {
        let (result, warnings) = internal_compile(
            &Asnr {
                state: AsnrSourcesSet {
                    sources: self.state.sources,
                    no_std: self.state.no_std,
                },
            },
            true,
        )?;

        fs::write(self.state.output_path, result)?;

        Ok(warnings)
    }
}

fn internal_compile(
    asnr: &Asnr<AsnrSourcesSet>,
    include_clippy_allows: bool,
) -> Result<(String, Vec<Box<dyn Error>>), Box<dyn Error>> {
    let mut result = imports_and_generic_types(None, asnr.state.no_std, include_clippy_allows);
    let mut warnings = Vec::<Box<dyn Error>>::new();
    let mut modules: Vec<ToplevelDeclaration> = vec![];
    for src in &asnr.state.sources {
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
            match generate(tld, None) {
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
            Asnr::new()
                .no_std(true)
                // .add_asn_by_path(PathBuf::from("test_asn1/AddGrpC.asn"))
                // .add_asn_by_path(PathBuf::from("test_asn1/ETSI-ITS-CDD.asn"))
                .add_asn_by_path(PathBuf::from("test_asn1/v2x.asn"))
                // .add_asn_by_path(PathBuf::from("test_asn1/denm_2_0.asn"))
                // .add_asn_by_path(PathBuf::from(
                //     "test_asn1/CPM-OriginatingStationContainers.asn"
                // ))
                // .add_asn_by_path(PathBuf::from("test_asn1/CPM-PerceivedObjectContainer.asn"))
                // .add_asn_by_path(PathBuf::from("test_asn1/CPM-PerceptionRegionContainer.asn"))
                // .add_asn_by_path(PathBuf::from(
                //     "test_asn1/CPM-SensorInformationContainer.asn"
                // ))
                // .add_asn_by_path(PathBuf::from("test_asn1/CPM-PDU-Descriptions.asn"))
                .set_output_path(PathBuf::from("../asnr-transcoder/src/generated.rs"))
                .compile()
                .unwrap()
        )
    }
}

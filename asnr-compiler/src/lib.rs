mod generator;
mod parser;
mod test;
mod validator;

use std::{error::Error, fs::{read_to_string, self}, path::PathBuf};

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
}

impl From<Vec<PathBuf>> for AsnrCompiler {
    fn from(value: Vec<PathBuf>) -> Self {
        AsnrCompiler { sources: value }
    }
}

impl From<PathBuf> for AsnrCompiler {
    fn from(value: PathBuf) -> Self {
        AsnrCompiler {
            sources: vec![value],
        }
    }
}

impl AsnrCompiler {
    pub fn new() -> AsnrCompiler {
        AsnrCompiler { sources: vec![] }
    }

    /// Add an ASN1 source to the compile command.
    /// * `path_to_source` - path to ASN1 file to include
    pub fn add_asn_source(mut self, path_to_source: PathBuf) -> AsnrCompiler {
        self.sources.push(path_to_source);
        self
    }

    /// Add several ASN1 sources to the compile command.
    /// * `path_to_source` - vector of paths to the ASN1 files to be included
    pub fn add_asn_sources(mut self, paths_to_sources: Vec<PathBuf>) -> AsnrCompiler {
        self.sources.extend(paths_to_sources.into_iter());
        self
    }

    // Runs the ASNR compiler command.
    pub fn compile(self) -> Result<(), Box<dyn Error>> {
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


#[cfg(test)]
mod tests {
    use crate::Asnr;

  #[test]
  fn compiles_a_simple_spec() {
    Asnr::compiler().add_asn_source("simple.asn".into()).compile();
  }
}
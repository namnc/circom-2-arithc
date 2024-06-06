//! # Circom To Arithmetic Circuit
//!
//! This library provides the functionality to convert a Circom program into an arithmetic circuit.

pub mod circom;
pub mod circuit;
pub mod cli;
pub mod process;
pub mod program;
pub mod runtime;

use std::path::{Path, PathBuf};

pub use cli::Args;

pub fn build_output(output_path: &Path, filename: &str, ext: &str) -> PathBuf {
    let mut file = output_path.to_path_buf();
    file.push(format!("{}.{}", filename, ext));
    file
}

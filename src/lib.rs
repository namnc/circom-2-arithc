//! # Circom To Arithmetic Circuit
//!
//! This library provides the functionality to convert a Circom program into an arithmetic circuit.

pub mod arithmetic_circuit;
pub mod circom;
pub mod cli;
pub mod compiler;
pub mod process;
pub mod program;
pub mod runtime;

pub use cli::{build_output, Args};
mod topological_sort;

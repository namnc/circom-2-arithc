//! # Circom To Arithmetic Circuit
//!
//! This library provides the functionality to convert a Circom program into an arithmetic circuit.

pub mod a_gate_type;
pub mod circom;
pub mod cli;
pub mod compiler;
pub mod process;
pub mod program;
pub mod runtime;

mod topological_sort;

//! # Program Module
//!
//! This module processes the circom input program to build the arithmetic circuit.

use crate::{
    circom::{input::Input, parser::parse_project, type_analysis::analyse_project},
    circuit::ArithmeticCircuit,
    process::process_statements,
    runtime::{Runtime, RuntimeError},
};
use circom_program_structure::ast::Expression;
use std::io;
use thiserror::Error;

/// Parses a given Circom program and constructs an arithmetic circuit from it.
pub fn build_circuit(input: &Input) -> Result<ArithmeticCircuit, ProgramError> {
    let mut circuit = ArithmeticCircuit::new();
    let mut runtime = Runtime::new();
    let mut program_archive = parse_project(input).map_err(|_| ProgramError::ParsingError)?;

    analyse_project(&mut program_archive).map_err(|_| ProgramError::AnalysisError)?;

    if let Expression::Call { id, .. } = program_archive.get_main_expression() {
        let statements = program_archive.get_template_data(id).get_body_as_vec();

        process_statements(
            &mut circuit,
            &mut runtime,
            statements,
            &program_archive,
            true,
        )?;
    }

    Ok(circuit)
}

/// Program errors
#[derive(Error, Debug)]
pub enum ProgramError {
    #[error("Analysis error")]
    AnalysisError,
    #[error("Call error")]
    CallError,
    #[error("Empty data item")]
    EmptyDataItem,
    #[error("Input initialization error")]
    InputInitializationError,
    #[error("Invalid data type")]
    InvalidDataType,
    #[error("IO error: {0}")]
    IOError(#[from] io::Error),
    #[error("JSON serialization error: {0}")]
    JsonSerializationError(#[from] serde_json::Error),
    #[error("Parsing error")]
    ParsingError,
    #[error("Runtime error: {0}")]
    RuntimeError(RuntimeError),
    #[error("Output directory creation error")]
    OutputDirectoryCreationError,
}

impl From<RuntimeError> for ProgramError {
    fn from(e: RuntimeError) -> Self {
        ProgramError::RuntimeError(e)
    }
}

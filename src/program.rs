//! # Program Module
//!
//! This module handles the parsing and processing of Circom circuits, enabling the construction and analysis of arithmetic circuits from Circom files.

use crate::circuit::ArithmeticCircuit;
use crate::compiler::{analyse_project, parse_project, Input};
use crate::runtime::{Runtime, RuntimeError};
use crate::traverse::traverse_sequence_of_statements;
use circom_program_structure::ast::Expression;
use circom_program_structure::program_archive::ProgramArchive;
use thiserror::Error;

/// Parses a Circom file, processes its content, and sets up the necessary structures for circuit analysis.
pub fn parse_circom() -> Result<(), ProgramError> {
    let user_input = Input::default();
    let mut program_archive = parse_project(&user_input).map_err(|_| ProgramError::ParsingError)?;
    analyse_project(&mut program_archive).map_err(|_| ProgramError::AnalysisError)?;

    let mut circuit = traverse_program(&program_archive)?;

    circuit.print_ac();
    circuit.truncate_zero_add_gate();
    circuit.print_ac();
    circuit.serde();

    Ok(())
}

/// Traverses the program structure of a parsed Circom file and constructs an arithmetic circuit.
pub fn traverse_program(
    program_archive: &ProgramArchive,
) -> Result<ArithmeticCircuit, ProgramError> {
    let mut circuit = ArithmeticCircuit::new();
    let mut runtime = Runtime::new()?;

    if let Expression::Call { id, .. } = program_archive.get_main_expression() {
        let template_body = program_archive.get_template_data(id).get_body_as_vec();

        traverse_sequence_of_statements(
            &mut circuit,
            &mut runtime,
            template_body,
            program_archive,
            true,
        )?;
    };

    Ok(circuit)
}

/// Program errors
#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum ProgramError {
    #[error("Analysis error")]
    AnalysisError,
    #[error("Call error")]
    CallError,
    #[error("Parsing error")]
    ParsingError,
    #[error("Context error: {0}")]
    RuntimeError(RuntimeError),
}

impl From<RuntimeError> for ProgramError {
    fn from(e: RuntimeError) -> Self {
        ProgramError::RuntimeError(e)
    }
}

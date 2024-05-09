//! # Program Module
//!
//! This module processes the circom input program to build the arithmetic circuit.

use crate::{
    circom::{input::Input, parser::parse_project, type_analysis::analyse_project},
    circuit::{AGateType, ArithmeticCircuit, CircuitError},
    process::{process_expression, process_statements},
    runtime::{DataAccess, DataType, Runtime, RuntimeError},
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

    match program_archive.get_main_expression() {
        Expression::Call { id, args, .. } => {
            let template_data = program_archive.get_template_data(id);

            // Get values
            let mut values: Vec<Option<u32>> = Vec::new();
            for expression in args {
                let access =
                    process_expression(&mut circuit, &mut runtime, &program_archive, expression)?;
                let value = runtime.current_context()?.get_variable_value(&access)?;
                values.push(value);
            }

            // Get and declare arguments
            let names = template_data.get_name_of_params();
            for (name, &value) in names.iter().zip(values.iter()) {
                let signal_gen = runtime.get_signal_gen();
                runtime.current_context()?.declare_item(
                    DataType::Variable,
                    name,
                    &[],
                    signal_gen,
                )?;
                runtime
                    .current_context()?
                    .set_variable(&DataAccess::new(name, Vec::new()), value)?;
            }

            // Process the main component
            let statements = template_data.get_body_as_vec();
            process_statements(&mut circuit, &mut runtime, &program_archive, statements)?;
        }
        _ => return Err(ProgramError::MainExpressionNotACall),
    }

    Ok(circuit)
}

pub fn build_circuit_pure(
    main: &str,
    _read_file: impl Fn(&str) -> String
) -> Result<ArithmeticCircuit, ProgramError> {
    if main == "(make_error)" {
        return Err(ProgramError::OperationError("testing error".into()));
    }

    let mut circuit = ArithmeticCircuit::new();

    circuit.add_signal(0, "a".into(), None)?;
    circuit.add_signal(1, "b".into(), None)?;

    circuit.add_signal(2, "c".into(), None)?;

    circuit.add_gate(AGateType::AAdd, 0, 1, 2)?;

    Ok(circuit)
}

/// Program errors
#[derive(Error, Debug)]
pub enum ProgramError {
    #[error("Analysis error")]
    AnalysisError,
    #[error("Call error")]
    CallError,
    #[error("Circuit error: {0}")]
    CircuitError(CircuitError),
    #[error("Empty data item")]
    EmptyDataItem,
    #[error("Expression not implemented")]
    ExpressionNotImplemented,
    #[error("Input initialization error")]
    InputInitializationError,
    #[error("Invalid data type")]
    InvalidDataType,
    #[error("IO error: {0}")]
    IOError(#[from] io::Error),
    #[error("JSON serialization error: {0}")]
    JsonSerializationError(#[from] serde_json::Error),
    #[error("Main expression not a call")]
    MainExpressionNotACall,
    #[error("Operation error: {0}")]
    OperationError(String),
    #[error("Operation not supported")]
    OperationNotSupported,
    #[error("Output directory creation error")]
    OutputDirectoryCreationError,
    #[error("Parsing error")]
    ParsingError,
    #[error("Runtime error: {0}")]
    RuntimeError(RuntimeError),
    #[error("Statement not implemented")]
    StatementNotImplemented,
    #[error("Undefined function or template")]
    UndefinedFunctionOrTemplate,
}

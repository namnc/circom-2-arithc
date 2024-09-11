//! # Program Module
//!
//! This module processes the circom input program to build the arithmetic circuit.

use crate::{
    circom::{parser::parse_project, type_analysis::analyse_project},
    cli::Args,
    compiler::{CircuitError, Compiler},
    process::{process_expression, process_statements},
    runtime::{DataAccess, DataType, Runtime, RuntimeError},
};
use bristol_circuit::BristolCircuitError;
use circom_program_structure::ast::Expression;
use std::io;
use thiserror::Error;

/// Parses a given Circom program and constructs an arithmetic circuit from it.
pub fn compile(args: &Args) -> Result<Compiler, ProgramError> {
    let mut compiler = Compiler::new();
    let mut runtime = Runtime::new();
    let mut program_archive = parse_project(args)?;

    analyse_project(&mut program_archive)?;

    match program_archive.get_main_expression() {
        Expression::Call { id, args, .. } => {
            let template_data = program_archive.get_template_data(id);

            // Get values
            let mut values: Vec<Option<u32>> = Vec::new();
            for expression in args {
                let access =
                    process_expression(&mut compiler, &mut runtime, &program_archive, expression)?;
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
            process_statements(&mut compiler, &mut runtime, &program_archive, statements)?;

            for (ikey, (_ivs, _ivh)) in template_data.get_inputs().iter() {
                let filter = format!("0.{}", ikey);
                compiler.add_inputs(compiler.get_signals(filter));
            }

            for (okey, (_ovs, _ovh)) in template_data.get_outputs().iter() {
                let filter = format!("0.{}", okey);
                let signals = compiler.get_signals(filter);
                compiler.add_outputs(signals);
            }
        }
        _ => return Err(ProgramError::MainExpressionNotACall),
    }

    compiler.update_type(args.value_type)?;

    Ok(compiler)
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
    #[error("Signal substitution not implemented")]
    SignalSubstitutionNotImplemented,
    #[error("Undefined function or template")]
    UndefinedFunctionOrTemplate,
    #[error(transparent)]
    BristolCircuitError(#[from] BristolCircuitError),
}

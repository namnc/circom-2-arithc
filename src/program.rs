//! # Program Module
//!
//! This module handles the parsing and processing of Circom circuits, enabling the construction and analysis of arithmetic circuits from Circom files.

use crate::circuit::ArithmeticCircuit;
use crate::compiler::{analyse_project, parse_project, Input};
use crate::runtime::Runtime;
use crate::traverse::traverse_sequence_of_statements;
use circom_program_structure::ast::Expression;
use circom_program_structure::program_archive::ProgramArchive;

/// Parses a Circom file, processes its content, and sets up the necessary structures for circuit analysis.
pub fn parse_circom() -> Result<(), ()> {
    let user_input = Input::default()?;
    let mut program_archive = parse_project(&user_input)?;
    analyse_project(&mut program_archive)?;

    let mut circuit = traverse_program(&program_archive);

    circuit.print_ac();
    circuit.truncate_zero_add_gate();
    circuit.print_ac();
    circuit.serde();

    Ok(())
}

/// Traverses the program structure of a parsed Circom file and constructs an arithmetic circuit.
pub fn traverse_program(program_archive: &ProgramArchive) -> ArithmeticCircuit {
    let mut ac = ArithmeticCircuit::new();
    let mut runtime = Runtime::new().unwrap();

    if let Expression::Call { id, .. } = program_archive.get_main_expression() {
        let template_body = program_archive.get_template_data(id).get_body_as_vec();

        traverse_sequence_of_statements(
            &mut ac,
            &mut runtime,
            template_body,
            program_archive,
            true,
        );
    };

    ac
}

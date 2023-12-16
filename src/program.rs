//! # Program Module
//!
//! This module handles the parsing and processing of Circom circuits, enabling the construction and analysis of arithmetic circuits from Circom files.

use crate::circuit::ArithmeticCircuit;
use crate::compiler::{analyse_project, parse_project, Input};
use crate::runtime::Runtime;
use crate::traverse::traverse_sequence_of_statements;
use circom_program_structure::ast::Expression;
use circom_program_structure::program_archive::ProgramArchive;
use mpz_circuits::types::ValueType;

/// Parses a Circom file, processes its content, and sets up the necessary structures for circuit analysis.
pub fn parse_circom(filename: &str, inputs: &[ValueType], outputs: &[ValueType]) -> Result<(), ()> {
    let user_input = Input::default()?;
    let mut program_archive = parse_project(&user_input)?;
    analyse_project(&mut program_archive)?;

    // let config = ExecutionConfig {
    //     no_rounds: user_input.no_rounds(),
    //     flag_p: user_input.parallel_simplification_flag(),
    //     flag_s: user_input.reduced_simplification_flag(),
    //     flag_f: user_input.unsimplified_flag(),
    //     flag_old_heuristics: user_input.flag_old_heuristics(),
    //     flag_verbose: user_input.flag_verbose(),
    //     inspect_constraints_flag: user_input.inspect_constraints_flag(),
    //     r1cs_flag: user_input.r1cs_flag(),
    //     json_constraint_flag: user_input.json_constraints_flag(),
    //     json_substitution_flag: user_input.json_substitutions_flag(),
    //     sym_flag: user_input.sym_flag(),
    //     sym: user_input.sym_file().to_string(),
    //     r1cs: user_input.r1cs_file().to_string(),
    //     json_constraints: user_input.json_constraints_file().to_string(),
    //     prime: user_input.prime(),
    // };

    traverse_program(&program_archive);

    // let circuit = execute_project(program_archive, config)?;
    // let compilation_config = CompilerConfig {
    //     vcp: circuit,
    //     debug_output: user_input.print_ir_flag(),
    //     c_flag: user_input.c_flag(),
    //     wasm_flag: user_input.wasm_flag(),
    //     wat_flag: user_input.wat_flag(),
    //     js_folder: user_input.js_folder().to_string(),
    //     wasm_name: user_input.wasm_name().to_string(),
    //     c_folder: user_input.c_folder().to_string(),
    //     c_run_name: user_input.c_run_name().to_string(),
    //     c_file: user_input.c_file().to_string(),
    //     dat_file: user_input.dat_file().to_string(),
    //     wat_file: user_input.wat_file().to_string(),
    //     wasm_file: user_input.wasm_file().to_string(),
    //     produce_input_log: user_input.main_inputs_flag(),
    // };
    // compile(compilation_config)?;

    // Sample code for binary circuit

    // let builder = CircuitBuilder::new();

    // let mut feed_ids: Vec<usize> = Vec::new();
    // let mut feed_map: HashMap<usize, Node<Feed>> = HashMap::default();

    // let mut input_len = 0;
    // for input in inputs {
    //     let input = builder.add_input_by_type(input.clone());
    //     for (node, old_id) in input.iter().zip(input_len..input_len + input.len()) {
    //         feed_map.insert(old_id, *node);
    //     }
    //     input_len += input.len();
    // }

    // let mut state = builder.state().borrow_mut();
    // let pattern = Regex::new(GATE_PATTERN).unwrap();
    // for cap in pattern.captures_iter(&file) {
    //     let UncheckedGate {
    //         xref,
    //         yref,
    //         zref,
    //         gate_type,
    //     } = UncheckedGate::parse(cap)?;
    //     feed_ids.push(zref);

    //     match gate_type {
    //         GateType::Xor => {
    //             let new_x = feed_map
    //                 .get(&xref)
    //                 .ok_or(ParseError::UninitializedFeed(xref))?;
    //             let new_y = feed_map
    //                 .get(&yref.unwrap())
    //                 .ok_or(ParseError::UninitializedFeed(yref.unwrap()))?;
    //             let new_z = state.add_xor_gate(*new_x, *new_y);
    //             feed_map.insert(zref, new_z);
    //         }
    //         GateType::And => {
    //             let new_x = feed_map
    //                 .get(&xref)
    //                 .ok_or(ParseError::UninitializedFeed(xref))?;
    //             let new_y = feed_map
    //                 .get(&yref.unwrap())
    //                 .ok_or(ParseError::UninitializedFeed(yref.unwrap()))?;
    //             let new_z = state.add_and_gate(*new_x, *new_y);
    //             feed_map.insert(zref, new_z);
    //         }
    //         GateType::Inv => {
    //             let new_x = feed_map
    //                 .get(&xref)
    //                 .ok_or(ParseError::UninitializedFeed(xref))?;
    //             let new_z = state.add_inv_gate(*new_x);
    //             feed_map.insert(zref, new_z);
    //         }
    //     }
    // }
    // drop(state);
    // feed_ids.sort();

    // for output in outputs.iter().rev() {
    //     let feeds = feed_ids
    //         .drain(feed_ids.len() - output.len()..)
    //         .map(|id| {
    //             *feed_map
    //                 .get(&id)
    //                 .expect("Old feed should be mapped to new feed")
    //         })
    //         .collect::<Vec<Node<Feed>>>();

    //     let output = output.to_bin_repr(&feeds).unwrap();
    //     builder.add_output(output);
    // }

    // Ok(builder.build()?)

    Ok(())
}

/// Traverses the program structure of a parsed Circom file and constructs an arithmetic circuit.
pub fn traverse_program(program_archive: &ProgramArchive) -> ArithmeticCircuit {
    let mut ac = ArithmeticCircuit::new();

    let mut runtime = Runtime::new().unwrap();

    let main_file_id = program_archive.get_file_id_main();

    // let mut runtime_information = RuntimeInformation::new(*main_file_id, program_archive.id_max, prime);

    use Expression::Call;

    // runtime_information.public_inputs = program_archive.get_public_inputs_main_component().clone();

    // Main expresssion
    // program_archive.get_main_expression()

    // Public inputs
    // program_archive.get_public_inputs_main_component().clone();

    if let Call { id, args, .. } = program_archive.get_main_expression() {
        let template_body = program_archive.get_template_data(id).get_body_as_vec();

        traverse_sequence_of_statements(
            &mut ac,
            &mut runtime,
            template_body,
            program_archive,
            true,
        );

        ac.print_ac();
        ac.truncate_zero_add_gate();
        ac.print_ac();
        ac.serde();

        // let folded_value_result =
        //     if let Call { id, args, .. } = &program_archive.get_main_expression() {
        //         let mut arg_values = Vec::new();
        //         for arg_expression in args.iter() {
        //             let f_arg = execute_expression(arg_expression, program_archive, &mut runtime_information, flags);
        //             arg_values.push(safe_unwrap_to_arithmetic_slice(f_arg.unwrap(), line!()));
        //             // improve
        //         }
        //         execute_template_call_complete(
        //             id,
        //             arg_values,
        //             BTreeMap::new(),
        //             program_archive,
        //             &mut runtime_information,
        //             flags,
        //         )
        //     } else {
        //         unreachable!("The main expression should be a call.");
        //     };

        // match folded_value_result {
        //     Result::Err(_) => Result::Err(runtime_information.runtime_errors),
        //     Result::Ok(folded_value) => {
        //         debug_assert!(FoldedValue::valid_node_pointer(&folded_value));
        //         Result::Ok((runtime_information.exec_program, runtime_information.runtime_errors))
        //     }
        // }
    };

    ac
}

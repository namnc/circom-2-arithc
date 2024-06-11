use std::collections::HashMap;

use circom_2_arithc::{program::compile, Args};
use sim_circuit::{simulate, NumberU32};

pub fn simulation_test<
    Input: IntoIterator<Item = (&'static str, u32)>,
    Output: IntoIterator<Item = (&'static str, u32)>,
>(
    test_file_path: &str,
    input: Input,
    expected_output: Output,
) {
    let compiler_input = Args::new(test_file_path.into(), "./".into());
    let circuit = compile(&compiler_input).unwrap().build_circuit().unwrap();

    let input = input
        .into_iter()
        .map(|(name, value)| (name.to_string(), NumberU32(value)))
        .collect::<HashMap<String, NumberU32>>();

    let expected_output = expected_output
        .into_iter()
        .map(|(name, value)| (name.to_string(), NumberU32(value)))
        .collect::<HashMap<String, NumberU32>>();

    let output = simulate(&circuit.to_sim(), &input).unwrap();

    assert_eq!(output, expected_output);
}

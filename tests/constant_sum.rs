use circom_2_arithc::{circom::input::Input, program::build_circuit};

const TEST_FILE_PATH: &str = "./tests/circuits/constantSum.circom";

#[test]
fn test_constant_sum() {
    let input = Input::new(TEST_FILE_PATH.into(), "./".into()).unwrap();
    let circuit = build_circuit(&input).unwrap();
    let sim_circuit = circuit.build_sim_circuit().unwrap();

    let circuit_input = vec![];
    let res = sim_circuit.execute(&circuit_input).unwrap();

    assert_eq!(res, vec![8]);
}

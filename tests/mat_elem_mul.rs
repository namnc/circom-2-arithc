use circom_2_arithc::{circom::input::Input, program::build_circuit};

const TEST_FILE_PATH: &str = "./tests/circuits/matElemMul.circom";

#[test]
fn test_matrix_element_multiplication() {
    let input = Input::new(TEST_FILE_PATH.into(), "./".into()).unwrap();
    let circuit = build_circuit(&input).unwrap();
    let sim_circuit = circuit.build_sim_circuit().unwrap();

    let circuit_input = vec![2, 2, 2, 2, 2, 2, 2, 2];
    let res = sim_circuit.execute(&circuit_input).unwrap();
    assert_eq!(res, vec![4, 4, 4, 4]);
}

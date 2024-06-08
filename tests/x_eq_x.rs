use circom_2_arithc::{program::build_circuit, Args};

const TEST_FILE_PATH: &str = "./tests/circuits/xEqX.circom";

#[test]
fn test_x_eq_x() {
    let input = Args::new(TEST_FILE_PATH.into(), "./".into());
    let circuit = build_circuit(&input).unwrap();
    let sim_circuit = circuit.build_sim_circuit().unwrap();

    let circuit_input = vec![1];
    let res = sim_circuit.execute(&circuit_input).unwrap();
    assert_eq!(res, vec![1]);
}

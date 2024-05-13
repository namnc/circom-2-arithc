use circom_2_arithc::{circom::input::Input, program::build_circuit};
use circom_virtual_fs::RealFs;

const TEST_FILE_PATH: &str = "./tests/circuits/sum.circom";

#[test]
fn test_sum() {
    let input = Input::new(TEST_FILE_PATH, ".", None);
    let circuit = build_circuit(&mut RealFs::new(), &input).unwrap();
    let sim_circuit = circuit.build_sim_circuit().unwrap();

    let circuit_input = vec![1, 2];
    let res = sim_circuit.execute(&circuit_input).unwrap();
    assert_eq!(res, vec![3]);
}

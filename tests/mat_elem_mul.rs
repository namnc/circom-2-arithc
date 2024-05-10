use circom_2_arithc::{circom::input::Input, program::build_circuit};
use circom_vfs_utils::normalize_physical_path;

const TEST_FILE_PATH: &str = "./tests/circuits/matElemMul.circom";

#[test]
fn test_matrix_element_multiplication() {
    let input = Input::new(
        &normalize_physical_path(TEST_FILE_PATH),
        &normalize_physical_path("."),
        None,
    );
    let circuit = build_circuit(&input).unwrap();
    let mut sim_circuit = circuit.build_sim_circuit().unwrap();

    let circuit_input = vec![2, 2, 2, 2, 2, 2, 2, 2];
    let res = sim_circuit.execute(&circuit_input).unwrap();
    assert_eq!(res, vec![4, 4, 4, 4]);
}

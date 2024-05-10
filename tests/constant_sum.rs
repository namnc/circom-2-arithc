use circom_2_arithc::{circom::input::Input, program::build_circuit};
use circom_vfs_utils::normalize_physical_path;

const TEST_FILE_PATH: &str = "./tests/circuits/constantSum.circom";

#[test]
fn test_constant_sum() {
    let input = Input::new_pure(
        &normalize_physical_path(TEST_FILE_PATH),
        &normalize_physical_path("."),
        None,
    );
    let circuit = build_circuit(&input).unwrap();
    let sim_circuit = circuit.build_sim_circuit().unwrap();

    let circuit_input = vec![];
    let res = sim_circuit.execute(&circuit_input).unwrap();

    assert_eq!(
        res,
        Vec::<u32>::new(), // FIXME: Should be vec![8]
    );
}

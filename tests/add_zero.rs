use circom_2_arithc::{circom::input::Input, program::build_circuit};
use circom_vfs_utils::normalize_physical_path;

const TEST_FILE_PATH: &str = "./tests/circuits/addZero.circom";

#[test]
fn test_add_zero() {
    let input = Input::new_pure(
        &normalize_physical_path(TEST_FILE_PATH),
        &normalize_physical_path("."),
        None,
    );
    let circuit = build_circuit(&input).unwrap();
    let sim_circuit = circuit.build_sim_circuit().unwrap();

    let circuit_input = vec![
        42, // actual input
        0,  // constant - FIXME: should not need to provide this
    ];

    let res = sim_circuit.execute(&circuit_input).unwrap();

    assert_eq!(res, vec![42]);
}

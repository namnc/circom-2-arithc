use circom_2_arithc::{circom::input::Input, program::build_circuit};
use circom_vfs_utils::normalize_physical_path;

const TEST_FILE_PATH: &str = "./tests/circuits/prefixOps.circom";

#[test]
fn test_prefix_ops() {
    let input = Input::new(
        &normalize_physical_path(TEST_FILE_PATH),
        &normalize_physical_path("."),
        None,
    );
    let circuit = build_circuit(&input).unwrap();
    let sim_circuit = circuit.build_sim_circuit().unwrap();

    let circuit_input = vec![
        0, 1, 2,    // actual inputs
        0, u32::MAX // constants - FIXME: should not need to provide these
    ];

    let res = sim_circuit.execute(&circuit_input).unwrap();

    assert_eq!(res, vec![
        0, // -0

        1, // !0
        0, // !1
        0, // !2

        0b_11111111_11111111_11111111_11111111, // ~0
        0b_11111111_11111111_11111111_11111110, // ~1
        0b_11111111_11111111_11111111_11111101, // ~2
    ]);
}

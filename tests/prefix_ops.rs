use circom_2_arithc::{circom::input::Input, program::build_circuit};
use circom_virtual_fs::RealFs;

const TEST_FILE_PATH: &str = "./tests/circuits/prefixOps.circom";

#[test]
fn test_prefix_ops() {
    let input = Input::new(TEST_FILE_PATH, ".", None);
    let circuit = build_circuit(&mut RealFs::new(), &input).unwrap();
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

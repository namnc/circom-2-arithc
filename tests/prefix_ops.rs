use circom_2_arithc::{circom::input::Input, program::build_circuit};

const TEST_FILE_PATH: &str = "./tests/circuits/prefixOps.circom";

#[test]
fn test_prefix_ops() {
    let input = Input::new(TEST_FILE_PATH.into(), "./".into()).unwrap();
    let circuit = build_circuit(&input).unwrap();
    let sim_circuit = circuit.build_sim_circuit().unwrap();

    let circuit_input = vec![0, 1, 2];
    let res = sim_circuit.execute(&circuit_input).unwrap();

    assert_eq!(res, vec![
        0, // -0

        1, // !0
        0, // !1
        0, // !2

        u32::MAX,     // ~0
        u32::MAX - 1, // ~1
        u32::MAX - 2, // ~2
    ]);
}

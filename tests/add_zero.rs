use circom_2_arithc::{circom::input::Input, program::build_circuit};
use circom_virtual_fs::RealFs;

const TEST_FILE_PATH: &str = "./tests/circuits/addZero.circom";

#[test]
fn test_add_zero() {
    let input = Input::new(TEST_FILE_PATH, ".", None);
    let circuit = build_circuit(&mut RealFs::new(), &input).unwrap();
    let sim_circuit = circuit.build_sim_circuit().unwrap();

    let circuit_input = vec![
        42, // actual input
        0,  // constant - FIXME: should not need to provide this
    ];

    let res = sim_circuit.execute(&circuit_input).unwrap();

    assert_eq!(res, vec![42]);
}

use circom_2_arithc::{circom::input::Input, program::build_circuit};

const TEST_FILE_PATH: &str = "./tests/circuits/underConstrained.circom";

#[test]
fn test_under_constrained() {
    let input = Input::new(TEST_FILE_PATH.into(), "./".into()).unwrap();

    // TODO: Should this be an error because the circuit is under-constrained?
    let circuit = build_circuit(&input).unwrap();

    let sim_circuit = circuit.build_sim_circuit().unwrap();

    let res = sim_circuit.execute(&vec![]).unwrap();

    assert_eq!(res, Vec::<u32>::new());
}

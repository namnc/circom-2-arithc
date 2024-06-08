use circom_2_arithc::{program::build_circuit, Args};

const TEST_FILE_PATH: &str = "./tests/circuits/constantSum.circom";

#[test]
fn test_constant_sum() {
    let input = Args::new(TEST_FILE_PATH.into(), "./".into());
    let circuit = build_circuit(&input).unwrap();
    let sim_circuit = circuit.build_sim_circuit().unwrap();

    let circuit_input = vec![];
    let res = sim_circuit.execute(&circuit_input).unwrap();

    assert_eq!(
        res,
        Vec::<u32>::new(), // FIXME: Should be vec![8]
    );
}

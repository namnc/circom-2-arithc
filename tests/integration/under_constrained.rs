use circom_2_arithc::{program::compile, Args};

const TEST_FILE_PATH: &str = "./tests/circuits/underConstrained.circom";

#[test]
fn test_under_constrained() {
    let input = Args::new(TEST_FILE_PATH.into(), "./".into());

    // TODO: Should this be an error because the circuit is under-constrained?
    let compiler = compile(&input).unwrap();

    let sim_circuit = compiler.build_sim_circuit().unwrap();

    let res = sim_circuit.execute(&[]).unwrap();

    assert_eq!(res, Vec::<u32>::new());
}

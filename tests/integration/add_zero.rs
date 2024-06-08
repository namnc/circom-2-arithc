use circom_2_arithc::{program::compile, Args};

const TEST_FILE_PATH: &str = "./tests/circuits/addZero.circom";

#[test]
fn test_add_zero() {
    let input = Args::new(TEST_FILE_PATH.into(), "./".into());
    let circuit = compile(&input).unwrap().build_circuit().unwrap();

    let outputs = circuit.eval([("0.in", 42)]).unwrap();

    assert_eq!(outputs.get("0.out"), Some(&42));
}

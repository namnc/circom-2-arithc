use circom_2_arithc::program::build_circuit_pure;

#[test]
fn test_build_circuit_pure() {
    let circuit = build_circuit_pure("", |_| "".into()).unwrap();
    let sim_circuit = circuit.build_sim_circuit().unwrap();

    let circuit_input = vec![1, 2];
    let res = sim_circuit.execute(&circuit_input).unwrap();
    assert_eq!(res, vec![3]);
}

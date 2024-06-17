use crate::helpers::simulation_test;

#[test]
#[should_panic]
fn should_fail() {
    simulation_test("tests/circuits/badCircuit.circom", [], []);
}

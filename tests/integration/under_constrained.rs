use crate::helpers::simulation_test;

#[test]
fn test_under_constrained() {
    // FIXME: There should be an error instead (zero comes from default initialization, not from
    //        running the circuit)
    simulation_test("tests/circuits/underConstrained.circom", [], [("0.x", 0)]);
}

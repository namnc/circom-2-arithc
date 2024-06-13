use crate::helpers::simulation_test;

#[test]
fn test_x_eq_x() {
    simulation_test("tests/circuits/xEqX.circom", [("0.x", 37)], [("0.out", 1)]);
}

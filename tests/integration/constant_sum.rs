mod helpers;
use helpers::simulation_test;

#[test]
fn test_constant_sum() {
    simulation_test("tests/circuits/constantSum.circom", [], [("0.out", 8)]);
}

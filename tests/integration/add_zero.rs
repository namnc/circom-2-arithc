use crate::helpers::simulation_test;

#[test]
fn test_add_zero() {
    simulation_test(
        "tests/circuits/addZero.circom",
        [("0.in", 42)],
        [("0.out", 42)],
    );
}

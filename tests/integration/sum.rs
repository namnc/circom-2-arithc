mod helpers;
use helpers::simulation_test;

#[test]
fn test_sum() {
    simulation_test(
        "tests/circuits/sum.circom",
        [("0.a", 3), ("0.b", 5)],
        [("0.out", 8)],
    );
}

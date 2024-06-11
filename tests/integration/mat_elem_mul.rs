mod helpers;
use helpers::simulation_test;

#[test]
fn test_matrix_element_multiplication() {
    simulation_test(
        "tests/circuits/matElemMul.circom",
        [
            ("0.a[0][0]", 2),
            ("0.a[0][1]", 2),
            ("0.a[1][0]", 2),
            ("0.a[1][1]", 2),
            ("0.b[0][0]", 2),
            ("0.b[0][1]", 2),
            ("0.b[1][0]", 2),
            ("0.b[1][1]", 2),
        ],
        [
            ("0.out[0][0]", 4),
            ("0.out[0][1]", 4),
            ("0.out[1][0]", 4),
            ("0.out[1][1]", 4),
        ],
    );
}

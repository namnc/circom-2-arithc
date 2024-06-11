mod helpers;
use helpers::simulation_test;

#[test]
#[should_panic] // FIXME: Should NOT panic (see comment below)
fn test_prefix_ops() {
    // FIXME: The compiler sees several of the outputs as inputs, leading to the error below
    //        CircuitError(Inconsistency {
    //            message: "The nodes used for input and output are not unique"
    //        })
    simulation_test(
        "tests/circuits/prefixOps.circom",
        [("0.a", 0), ("0.b", 1), ("0.c", 2)],
        [
            ("0.negateA", 0),                                          // -0
            ("0.notA", 1),                                             // !0
            ("0.notB", 0),                                             // !1
            ("0.notC", 0),                                             // !2
            ("0.complementA", 0b_11111111_11111111_11111111_11111111), // ~0
            ("0.complementB", 0b_11111111_11111111_11111111_11111110), // ~1
            ("0.complementC", 0b_11111111_11111111_11111111_11111101), // ~2
        ],
    );
}

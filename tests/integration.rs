use circom_2_arithc::{
    cli::{Args, ValueType},
    program::compile,
};
use sim_circuit::{simulate, NumberU32};
use std::collections::HashMap;

pub fn simulation_test<
    Input: IntoIterator<Item = (&'static str, u32)>,
    Output: IntoIterator<Item = (&'static str, u32)>,
>(
    test_file_path: &str,
    input: Input,
    expected_output: Output,
) {
    let compiler_input = Args::new(test_file_path.into(), "./".into(), ValueType::Sint);
    let circuit = compile(&compiler_input).unwrap().build_circuit().unwrap();

    let input = input
        .into_iter()
        .map(|(name, value)| (name.to_string(), NumberU32(value)))
        .collect::<HashMap<String, NumberU32>>();

    let expected_output = expected_output
        .into_iter()
        .map(|(name, value)| (name.to_string(), NumberU32(value)))
        .collect::<HashMap<String, NumberU32>>();

    let output = simulate(&circuit.to_sim(), &input).unwrap();

    assert_eq!(output, expected_output);
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use circom_2_arithc::{cli::Args, program::compile};

    #[test]
    fn test_add_zero() {
        simulation_test(
            "tests/circuits/integration/addZero.circom",
            [("0.in", 42)],
            [("0.out", 42)],
        );
    }

    #[test]
    fn test_constant_sum() {
        simulation_test(
            "tests/circuits/integration/constantSum.circom",
            [],
            [("0.out", 8)],
        );
    }

    #[test]
    fn test_infix_ops() {
        simulation_test(
            "tests/circuits/integration/infixOps.circom",
            [
                ("0.x0", 0),
                ("0.x1", 1),
                ("0.x2", 2),
                ("0.x3", 3),
                ("0.x4", 4),
                ("0.x5", 5),
            ],
            [
                ("0.mul_2_3", 6),
                // ("0.div_4_3", 1), // unsupported for NumberU32
                ("0.idiv_4_3", 1),
                ("0.add_3_4", 7),
                ("0.sub_4_1", 3),
                ("0.pow_2_4", 16),
                ("0.mod_5_3", 2),
                ("0.shl_5_1", 10),
                ("0.shr_5_1", 2),
                ("0.leq_2_3", 1),
                ("0.leq_3_3", 1),
                ("0.leq_4_3", 0),
                ("0.geq_2_3", 0),
                ("0.geq_3_3", 1),
                ("0.geq_4_3", 1),
                ("0.lt_2_3", 1),
                ("0.lt_3_3", 0),
                ("0.lt_4_3", 0),
                ("0.gt_2_3", 0),
                ("0.gt_3_3", 0),
                ("0.gt_4_3", 1),
                ("0.eq_2_3", 0),
                ("0.eq_3_3", 1),
                ("0.neq_2_3", 1),
                ("0.neq_3_3", 0),
                ("0.or_0_1", 1),
                ("0.and_0_1", 0),
                ("0.bit_or_1_3", 3),
                ("0.bit_and_1_3", 1),
                ("0.bit_xor_1_3", 2),
            ],
        );
    }

    #[test]
    fn test_matrix_element_multiplication() {
        simulation_test(
            "tests/circuits/integration/matElemMul.circom",
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

    #[test]
    #[should_panic] // FIXME: Should NOT panic (see comment below)
    fn test_prefix_ops() {
        // FIXME: The compiler sees several of the outputs as inputs, leading to the error below
        //        CircuitError(Inconsistency {
        //            message: "Node 10 used for both input 0.complementC and output 0.complementC"
        //        })
        simulation_test(
            "tests/circuits/integration/prefixOps.circom",
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

    #[test]
    fn test_sum() {
        simulation_test(
            "tests/circuits/integration/sum.circom",
            [("0.a", 3), ("0.b", 5)],
            [("0.out", 8)],
        );
    }

    #[test]
    fn test_under_constrained() {
        // FIXME: There should be an error instead (zero comes from default initialization, not from
        //        running the circuit)
        simulation_test(
            "tests/circuits/integration/underConstrained.circom",
            [],
            [("0.x", 0)],
        );
    }

    #[test]
    fn test_x_eq_x() {
        simulation_test(
            "tests/circuits/integration/xEqX.circom",
            [("0.x", 37)],
            [("0.out", 1)],
        );
    }

    #[test]
    fn test_direct_output() {
        simulation_test(
            "tests/circuits/integration/directOutput.circom",
            [],
            [("0.out", 42)],
        );
    }

    #[test]
    fn test_out_of_bounds() {
        let compiler_input = Args::new(
            "tests/circuits/integration/indexOutOfBounds.circom".into(),
            "./".into(),
            ValueType::Sint,
        );
        let circuit = compile(&compiler_input);

        assert!(circuit.is_err());
        assert_eq!(
            circuit.unwrap_err().to_string(),
            "Runtime error: Index out of bounds"
        );
    }
}

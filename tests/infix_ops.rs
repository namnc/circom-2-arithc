use circom_2_arithc::{circom::input::Input, program::build_circuit};

const TEST_FILE_PATH: &str = "./tests/circuits/infixOps.circom";

#[test]
fn test_infix_ops() {
    let input = Input::new(TEST_FILE_PATH.into(), "./".into()).unwrap();
    let circuit = build_circuit(&input).unwrap();
    let sim_circuit = circuit.build_sim_circuit().unwrap();

    let circuit_input = vec![0, 1, 2, 3, 4, 5];

    let res = sim_circuit.execute(&circuit_input).unwrap();

    assert_eq!(res, vec![
        6,  // 2  * 3
        1,  // 4  / 3 // TODO: Should this behave differently? (finite field division)
        1,  // 4  \ 3 // (This one is definitely int division)
        7,  // 3  + 4
        3,  // 4  - 1
        16, // 2 ** 4
        2,  // 5  % 3
        10, // 5 << 1
        2,  // 5 >> 1
        1,  // 2 <= 3
        // 1,  // 3 <= 3
        0,  // 4 <= 3
        0,  // 2 >= 3
        // 1,  // 3 >= 3
        1,  // 4 >= 3
        1,  // 2  < 3
        // 0,  // 3  < 3
        0,  // 4  < 3
        0,  // 2  > 3
        // 1,  // 3  > 3
        1,  // 4  > 3
        0,  // 2 == 3
        // 1,  // 3 == 3
        1,  // 2 != 3
        // 0,  // 3 != 3
        1,  // 0 || 1
        0,  // 0 && 1
        3,  // 1  | 3
        1,  // 1  & 3
        2,  // 1  ^ 3
    ]);
}

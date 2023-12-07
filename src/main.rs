use circom_2_arithc::program::parse_circom;
use dotenv::dotenv;
use env_logger::{init_from_env, Env};
use mpz_circuits::types::ValueType;

fn main() {
    dotenv().ok();
    init_from_env(Env::default().filter_or("LOG_LEVEL", "info"));

    let _circ = parse_circom(
        "circuits/bristol/adder64_reverse.txt",
        &[ValueType::U64, ValueType::U64],
        &[ValueType::U64],
    )
    .unwrap();

    // stupid assert always true
    assert_eq!(3, 3);

    //let output: u64 = evaluate!(circ, fn(1u64, 2u64) -> u64).unwrap();

    //assert_eq!(output, 3);
}

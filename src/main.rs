use circom_2_arithc::program::parse_circom;
use dotenv::dotenv;
use env_logger::{init_from_env, Env};
use mpz_circuits::types::ValueType;

fn main() {
    dotenv().ok();
    init_from_env(Env::default().filter_or("LOG_LEVEL", "info"));

    parse_circom(
        "circuits/bristol/adder64_reverse.txt",
        &[ValueType::U64, ValueType::U64],
        &[ValueType::U64],
    )
    .unwrap()
}

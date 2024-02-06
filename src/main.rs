use circom_2_arithc::{
    compiler::Input,
    program::{build_circuit, ProgramError},
};
use dotenv::dotenv;
use env_logger::init_from_env;
use serde_json::to_string;
use std::{fs::File, io::Write};

fn main() -> Result<(), ProgramError> {
    dotenv().expect("Failed to initialize dotenv");
    init_from_env("LOG_LEVEL=info");

    let input = Input::new().map_err(|_| ProgramError::InputInitializationError)?;
    let circuit = build_circuit(&input)?;

    let circuit_json = to_string(&circuit)?;
    File::create(input.out_mpc)?.write_all(circuit_json.as_bytes())?;

    Ok(())
}

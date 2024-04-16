use circom_2_arithc::{
    circom::input::{input_processing::view, Input},
    program::{build_circuit, ProgramError},
};
use dotenv::dotenv;
use env_logger::init_from_env;
use serde_json::to_string;
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

fn main() -> Result<(), ProgramError> {
    optional_dotenv();
    init_from_env("LOG_LEVEL=info");

    let output_path = PathBuf::from(view().value_of("output").unwrap());

    fs::create_dir_all(output_path).map_err(|_| ProgramError::OutputDirectoryCreationError)?;

    let input = Input::new().map_err(|_| ProgramError::InputInitializationError)?;
    let output_dir = input
        .out_r1cs
        .parent()
        .ok_or(ProgramError::OutputDirectoryCreationError)?
        .to_path_buf();

    let circuit_json = to_string(&build_circuit(&input)?)?;
    let output_file_path = Input::build_output(&output_dir, &input.out_wasm_name, "json");
    File::create(output_file_path)?.write_all(circuit_json.as_bytes())?;

    Ok(())
}

fn optional_dotenv() -> () {
    let env_error = match dotenv() {
        Ok(_) => None,
        Err(e) => match &e {
            dotenv::Error::Io(io_error) => match io_error.kind() {
                // Allow missing .env file
                std::io::ErrorKind::NotFound => Some(e),

                _ => Some(e),
            },
            _ => Some(e),
        }
    };

    if let Some(env_error) = env_error {
        panic!("Failed to initialize environment: {}", env_error);
    }
}

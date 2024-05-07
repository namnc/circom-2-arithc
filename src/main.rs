use circom_2_arithc::{
    circom::input::{input_processing::view, Input},
    program::{build_circuit, ProgramError},
};
use dotenv::dotenv;
use env_logger::{init_from_env, Env};
use serde_json::to_string;
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

fn main() -> Result<(), ProgramError> {
    dotenv().ok();
    init_from_env(Env::default().filter_or("LOG_LEVEL", "info"));

    let output_path = PathBuf::from(view().value_of("output").unwrap());

    fs::create_dir_all(output_path.clone())
        .map_err(|_| ProgramError::OutputDirectoryCreationError)?;

    let input = Input::new(PathBuf::from("./src/assets/circuit.circom"), output_path)
        .map_err(|_| ProgramError::InputInitializationError)?;
    let output_dir = input
        .out_r1cs
        .parent()
        .ok_or(ProgramError::OutputDirectoryCreationError)?
        .to_path_buf();

    let circuit = build_circuit(&input)?;
    let report = circuit.generate_circuit_report()?;

    let output_file_path = Input::build_output(&output_dir, &input.out_wasm_name, "json");
    File::create(output_file_path)?.write_all(to_string(&circuit)?.as_bytes())?;

    let report_file_path = PathBuf::from("./output/report.json");
    File::create(report_file_path)?.write_all(to_string(&report)?.as_bytes())?;

    Ok(())
}

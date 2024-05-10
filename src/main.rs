use circom_2_arithc::{
    circom::input::{input_processing::view, Input},
    program::{build_circuit, ProgramError},
};
use circom_vfs_utils::normalize_physical_path;
use dotenv::dotenv;
use env_logger::{init_from_env, Env};
use serde_json::to_string;
use std::{
    fs::{self, File},
    io::Write,
};

fn main() -> Result<(), ProgramError> {
    dotenv().ok();
    init_from_env(Env::default().filter_or("LOG_LEVEL", "info"));

    let output_path = normalize_physical_path(view().value_of("output").unwrap());

    fs::create_dir_all(&output_path)
        .map_err(|_| ProgramError::OutputDirectoryCreationError)?;

    let input = Input::new(
        normalize_physical_path("src/assets/circuit.circom").into(),
        output_path.into(),
    )
        .map_err(|_| ProgramError::InputInitializationError)?;
    let output_dir = input
        .out_r1cs
        .parent()
        .ok_or(ProgramError::OutputDirectoryCreationError)?;

    let circuit = build_circuit(&input)?;
    let report = circuit.generate_circuit_report()?;

    let output_file_path = Input::build_output(&output_dir, &input.out_wasm_name, "json");
    File::create(output_file_path.to_string())?.write_all(to_string(&circuit)?.as_bytes())?;

    File::create("output/report.json")?.write_all(to_string(&report)?.as_bytes())?;

    Ok(())
}

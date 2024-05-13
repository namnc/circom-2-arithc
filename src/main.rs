use circom_2_arithc::{
    circom::input::{input_processing::{self, view}, Input},
    program::{build_circuit, ProgramError},
};
use circom_virtual_fs::{FileSystem, RealFs};
use dotenv::dotenv;
use env_logger::{init_from_env, Env};
use serde_json::to_string;
use std::{
    fs::File,
    io::Write,
};

fn main() -> Result<(), ProgramError> {
    dotenv().ok();
    init_from_env(Env::default().filter_or("LOG_LEVEL", "info"));

    let mut fs = RealFs::new();

    let output_path = fs.normalize(&view().value_of("output").unwrap().into()).unwrap();

    fs.create_dir_all(&output_path)
        .map_err(|_| ProgramError::OutputDirectoryCreationError)?;

    let input = input_processing::generate_input(
        &fs.normalize(&"src/assets/circuit.circom".into())?.to_string(),
        &output_path.to_string(),
    )
        .map_err(|_| ProgramError::InputInitializationError)?;

    let output_dir = input
        .out_r1cs
        .parent()
        .ok_or(ProgramError::OutputDirectoryCreationError)?;

    let circuit = build_circuit(&mut fs, &input)?;
    let report = circuit.generate_circuit_report()?;

    let output_file_path = Input::build_output(&output_dir, &input.out_wasm_name, "json");
    File::create(output_file_path.to_string())?.write_all(to_string(&circuit)?.as_bytes())?;

    File::create("output/report.json")?.write_all(to_string(&report)?.as_bytes())?;

    Ok(())
}

use circom_2_arithc::{
    circom::input::{input_processing::view, Input},
    program::{build_circuit, ProgramError},
};
use circom_vfs_utils::{canonicalize_physical_path, SimplePath};
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

    let mut output_path = SimplePath::new(&canonicalize_physical_path("."));
    output_path.push(view().value_of("output").unwrap());

    fs::create_dir_all(output_path.to_string())
        .map_err(|_| ProgramError::OutputDirectoryCreationError)?;

    let mut main = SimplePath::new(&canonicalize_physical_path("."));
    main.push("src/assets/circuit.circom");

    let input = Input::new(main, output_path)
        .map_err(|_| ProgramError::InputInitializationError)?;
    let output_dir = input
        .out_r1cs
        .parent()
        .ok_or(ProgramError::OutputDirectoryCreationError)?;

    let circuit = build_circuit(&input)?;
    let report = circuit.generate_circuit_report()?;

    let output_file_path = Input::build_output(&output_dir, &input.out_wasm_name, "json");
    File::create(output_file_path.to_string())?.write_all(to_string(&circuit)?.as_bytes())?;

    let mut report_file_path = SimplePath::new(&canonicalize_physical_path("."));
    report_file_path.push("output/report.json");
    File::create(report_file_path.to_string())?.write_all(to_string(&report)?.as_bytes())?;

    Ok(())
}

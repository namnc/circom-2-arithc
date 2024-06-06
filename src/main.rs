use std::{
    fs::{self, File},
    io::Write,
};

use circom_2_arithc::{
    build_output,
    cli::Args,
    program::{build_circuit, ProgramError},
};

use clap::Parser;
use dotenv::dotenv;
use env_logger::{init_from_env, Env};
use serde_json::to_string;

fn main() -> Result<(), ProgramError> {
    dotenv().ok();
    init_from_env(Env::default().filter_or("LOG_LEVEL", "info"));

    let args = Args::parse();

    let circuit = build_circuit(&args)?;
    let report = circuit.generate_circuit_report()?;

    let output_dir = args.output.clone();
    fs::create_dir_all(output_dir.clone())
        .map_err(|_| ProgramError::OutputDirectoryCreationError)?;

    let output_file_path = build_output(
        &output_dir,
        args.input.file_stem().unwrap().to_str().unwrap(),
        "json",
    );
    File::create(output_file_path)?.write_all(to_string(&circuit)?.as_bytes())?;

    let report_file_path = build_output(&output_dir, "report", "json");
    File::create(report_file_path)?.write_all(to_string(&report)?.as_bytes())?;

    Ok(())
}

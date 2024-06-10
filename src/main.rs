use std::{
    fs::{self, File},
    io::Write,
};

use circom_2_arithc::{
    build_output,
    cli::Args,
    program::{compile, ProgramError},
};

use clap::Parser;
use dotenv::dotenv;
use env_logger::{init_from_env, Env};
use serde_json::{to_string, to_string_pretty};

fn main() -> Result<(), ProgramError> {
    dotenv().ok();
    init_from_env(Env::default().filter_or("LOG_LEVEL", "info"));

    let args = Args::parse();

    let compiler = compile(&args)?;
    let report = compiler.generate_circuit_report()?;

    let output_dir = args.output.clone();
    fs::create_dir_all(output_dir.clone())
        .map_err(|_| ProgramError::OutputDirectoryCreationError)?;

    let circuit = compiler.build_circuit()?;

    let output_file_path = build_output(&output_dir, "circuit", "txt");
    circuit.write_bristol(&mut File::create(output_file_path)?)?;

    let output_file_path = build_output(&output_dir, "circuit_info", "json");
    File::create(output_file_path)?.write_all(to_string_pretty(&circuit.info)?.as_bytes())?;

    let report_file_path = build_output(&output_dir, "report", "json");
    File::create(report_file_path)?.write_all(to_string(&report)?.as_bytes())?;

    Ok(())
}

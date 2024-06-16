use clap::Parser;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[clap(name = "Arithmetic Circuits Compiler")]
#[command(disable_help_subcommand = true)]
pub struct Args {
    /// Input file to process
    #[arg(
        short,
        long,
        help = "Path to the input file",
        default_value = "./input/circuit.circom"
    )]
    pub input: PathBuf,

    /// Output file to write the result
    #[arg(
        short,
        long,
        help = "Path to the directory where the output will be written",
        default_value = "./output/"
    )]
    pub output: PathBuf,
}

impl Args {
    pub fn new(input: PathBuf, output: PathBuf) -> Self {
        Self { input, output }
    }
}

/// Function that returns output file path
pub fn build_output(output_path: &Path, filename: &str, ext: &str) -> PathBuf {
    let mut file = output_path.to_path_buf();
    file.push(format!("{}.{}", filename, ext));
    file
}

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[clap(name = "Arithmetic Circuits Compiler")]
#[command(disable_help_subcommand = true)]
pub struct Args {
    /// Input file to process
    #[arg(
        short,
        long,
        help = "Path to the input file",
        default_value = "./src/assets/circuit.circom"
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

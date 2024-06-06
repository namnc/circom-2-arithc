use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[clap(name = "Arithmetic Circuits Compiler")]
#[command(disable_help_subcommand = true)]
pub struct Args {
    /// Input file to process
    #[arg(short, long, help = "Path to the input file")]
    pub input: PathBuf,

    /// Output file to write the result
    #[arg(short, long, help = "Path to the output file")]
    pub output: PathBuf,
}

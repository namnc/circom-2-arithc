use clap::{Parser, ValueEnum};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ValueType {
    Sint,
    Sfloat,
}

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

    /// Default type that's used for values in a MPC backend
    #[arg(
        short,
        long,
        value_enum,
        help = "Type that'll be used for values in a MPC backend",
        default_value_t = ValueType::Sint,
    )]
    pub value_type: ValueType,
}

impl Args {
    pub fn new(input: PathBuf, output: PathBuf, value_type: ValueType) -> Self {
        Self {
            input,
            output,
            value_type,
        }
    }
}

/// Function that returns output file path
pub fn build_output(output_path: &Path, filename: &str, ext: &str) -> PathBuf {
    let mut file = output_path.to_path_buf();
    file.push(format!("{}.{}", filename, ext));
    file
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_output() {
        let output_path = Path::new("./output");
        let filename = "result";
        let ext = "txt";

        let expected = PathBuf::from("./output/result.txt");
        let result = build_output(output_path, filename, ext);

        assert_eq!(result, expected);
    }
}

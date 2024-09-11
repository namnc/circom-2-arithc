use std::path::{Path, PathBuf};

use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ValueType {
    #[serde(rename = "sint")]
    #[default]
    Sint,
    #[serde(rename = "sfloat")]
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

    #[arg(
        short,
        long,
        value_enum,
        help = "Type that'll be used for values in MPC backend",
        default_value_t = ValueType::Sint,
    )]
    pub value_type: ValueType,

    #[arg(
        long,
        help = "Optional: Convert to a boolean circuit by using integers with this number of bits",
        default_value = None,
    )]
    pub boolify_width: Option<usize>,
}

impl Args {
    pub fn new(
        input: PathBuf,
        output: PathBuf,
        value_type: ValueType,
        boolify_width: Option<usize>,
    ) -> Self {
        Self {
            input,
            output,
            value_type,
            boolify_width,
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

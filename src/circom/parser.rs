use crate::{circom::VERSION, cli::Args, program::ProgramError};
use circom_parser::run_parser;
use circom_program_structure::{error_definition::Report, program_archive::ProgramArchive};

pub fn parse_project(args: &Args) -> Result<ProgramArchive, ProgramError> {
    let initial_file = args.input.to_str().unwrap().to_string();
    match run_parser(initial_file, VERSION, vec![]) {
        Result::Err((file_library, report_collection)) => {
            Report::print_reports(&report_collection, &file_library);
            Result::Err(ProgramError::ParsingError)
        }
        Result::Ok((program_archive, warnings)) => {
            Report::print_reports(&warnings, &program_archive.file_library);
            Result::Ok(program_archive)
        }
    }
}

use crate::program::ProgramError;
use circom_program_structure::{error_definition::Report, program_archive::ProgramArchive};
use circom_type_analysis::check_types::check_types;

pub fn analyse_project(program_archive: &mut ProgramArchive) -> Result<(), ProgramError> {
    match check_types(program_archive) {
        Err(errs) => {
            Report::print_reports(&errs, program_archive.get_file_library());
            Err(ProgramError::AnalysisError)
        }
        Ok(warns) => {
            Report::print_reports(&warns, program_archive.get_file_library());
            Ok(())
        }
    }
}

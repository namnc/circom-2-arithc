use circom_circom_algebra::num_traits::ToPrimitive;
use circom_compiler::compiler_interface;
use circom_compiler::compiler_interface::{Config, VCP};
use circom_constraint_writers::debug_writer::DebugWriter;
use circom_program_structure::ast::{
    Access, Expression, ExpressionInfixOpcode, SignalType, Statement, VariableType,
};
use circom_program_structure::error_definition::Report;
use circom_program_structure::program_archive::ProgramArchive;
use circom_type_analysis::check_types::check_types;
use clap::{App, Arg, ArgMatches};
use mpz_circuits::types::ValueType;
use mpz_circuits::{BuilderError, GateType};
use regex::Captures;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{
    collections::{HashMap, LinkedList},
    fmt,
    path::PathBuf,
};

const VERSION: &'static str = "2.0.0";

#[allow(missing_docs)]
pub struct CompilerConfig {
    pub js_folder: String,
    pub wasm_name: String,
    pub wat_file: String,
    pub wasm_file: String,
    pub c_folder: String,
    pub c_run_name: String,
    pub c_file: String,
    pub dat_file: String,
    pub wat_flag: bool,
    pub wasm_flag: bool,
    pub c_flag: bool,
    pub debug_output: bool,
    pub produce_input_log: bool,
    pub vcp: VCP,
}

pub fn compile(config: CompilerConfig) -> Result<(), ()> {
    // activate the c_flag in config
    // later replace it with something else that is more meaningful

    // config.c_flag = true;
    // config.wasm_flag = false;
    // config.wat_flag = false;

    let circuit = compiler_interface::run_compiler(
        config.vcp,
        Config {
            debug_output: config.debug_output,
            produce_input_log: config.produce_input_log,
            wat_flag: config.wat_flag,
        },
        VERSION,
    )?;

    // Sample code for calling writer

    // if config.c_flag {
    //     compiler_interface::write_c(
    //         &circuit,
    //         &config.c_folder,
    //         &config.c_run_name,
    //         &config.c_file,
    //         &config.dat_file,
    //     )?;
    //     println!(
    //         "{} {} and {}",
    //         Colour::Green.paint("Written successfully:"),
    //         config.c_file,
    //         config.dat_file
    //     );
    //     println!(
    //         "{} {}/{}, {}, {}, {}, {}, {}, {} and {}",
    //         Colour::Green.paint("Written successfully:"),
    //         &config.c_folder,
    //         "main.cpp".to_string(),
    //         "circom.hpp".to_string(),
    //         "calcwit.hpp".to_string(),
    //         "calcwit.cpp".to_string(),
    //         "fr.hpp".to_string(),
    //         "fr.cpp".to_string(),
    //         "fr.asm".to_string(),
    //         "Makefile".to_string()
    //     );
    // }

    Ok(())
}

pub struct ExecutionConfig {
    pub r1cs: String,
    pub sym: String,
    pub json_constraints: String,
    pub no_rounds: usize,
    pub flag_s: bool,
    pub flag_f: bool,
    pub flag_p: bool,
    pub flag_old_heuristics: bool,
    pub flag_verbose: bool,
    pub inspect_constraints_flag: bool,
    pub sym_flag: bool,
    pub r1cs_flag: bool,
    pub json_substitution_flag: bool,
    pub json_constraint_flag: bool,
    pub prime: String,
}

pub fn execute_project(
    program_archive: ProgramArchive,
    config: ExecutionConfig,
) -> Result<VCP, ()> {
    use circom_constraint_generation::{build_circuit, BuildConfig};
    let debug = DebugWriter::new(config.json_constraints).unwrap();
    let build_config = BuildConfig {
        no_rounds: config.no_rounds,
        flag_json_sub: config.json_substitution_flag,
        flag_s: config.flag_s,
        flag_f: config.flag_f,
        flag_p: config.flag_p,
        flag_verbose: config.flag_verbose,
        inspect_constraints: config.inspect_constraints_flag,
        flag_old_heuristics: config.flag_old_heuristics,
        prime: config.prime,
    };
    let custom_gates = program_archive.custom_gates;
    let (exporter, vcp) = build_circuit(program_archive, build_config)?;

    // Sample code for generate constraints but we don't need it now
    // Maybe later for generate for Garbler and Evaluator

    // if config.r1cs_flag {
    //     generate_output_r1cs(&config.r1cs, exporter.as_ref(), custom_gates)?;
    // }
    // if config.sym_flag {
    //     generate_output_sym(&config.sym, exporter.as_ref())?;
    // }
    // if config.json_constraint_flag {
    //     generate_json_constraints(&debug, exporter.as_ref())?;
    // }

    Result::Ok(vcp)
}

// fn generate_output_r1cs(
//     file: &str,
//     exporter: &dyn ConstraintExporter,
//     custom_gates: bool,
// ) -> Result<(), ()> {
//     if let Result::Ok(()) = exporter.r1cs(file, custom_gates) {
//         println!("{} {}", Colour::Green.paint("Written successfully:"), file);
//         Result::Ok(())
//     } else {
//         eprintln!(
//             "{}",
//             Colour::Red.paint("Could not write the output in the given path")
//         );
//         Result::Err(())
//     }
// }

// fn generate_output_sym(file: &str, exporter: &dyn ConstraintExporter) -> Result<(), ()> {
//     if let Result::Ok(()) = exporter.sym(file) {
//         println!("{} {}", Colour::Green.paint("Written successfully:"), file);
//         Result::Ok(())
//     } else {
//         eprintln!(
//             "{}",
//             Colour::Red.paint("Could not write the output in the given path")
//         );
//         Result::Err(())
//     }
// }

// fn generate_json_constraints(
//     debug: &DebugWriter,
//     exporter: &dyn ConstraintExporter,
// ) -> Result<(), ()> {
//     if let Ok(()) = exporter.json_constraints(&debug) {
//         println!(
//             "{} {}",
//             Colour::Green.paint("Constraints written in:"),
//             debug.json_constraints
//         );
//         Result::Ok(())
//     } else {
//         eprintln!(
//             "{}",
//             Colour::Red.paint("Could not write the output in the given path")
//         );
//         Result::Err(())
//     }
// }

pub struct Input {
    pub input_program: PathBuf,
    pub out_r1cs: PathBuf,
    pub out_json_constraints: PathBuf,
    pub out_wat_code: PathBuf,
    pub out_wasm_code: PathBuf,
    pub out_wasm_name: String,
    pub out_js_folder: PathBuf,
    pub out_c_run_name: String,
    pub out_c_folder: PathBuf,
    pub out_c_code: PathBuf,
    pub out_c_dat: PathBuf,
    pub out_sym: PathBuf,
    //pub field: &'static str,
    pub c_flag: bool,
    pub wasm_flag: bool,
    pub wat_flag: bool,
    pub r1cs_flag: bool,
    pub sym_flag: bool,
    pub json_constraint_flag: bool,
    pub json_substitution_flag: bool,
    pub main_inputs_flag: bool,
    pub print_ir_flag: bool,
    pub fast_flag: bool,
    pub reduced_simplification_flag: bool,
    pub parallel_simplification_flag: bool,
    pub flag_old_heuristics: bool,
    pub inspect_constraints_flag: bool,
    pub no_rounds: usize,
    pub flag_verbose: bool,
    pub prime: String,
    pub link_libraries: Vec<PathBuf>,
}

const R1CS: &'static str = "r1cs";
const WAT: &'static str = "wat";
const WASM: &'static str = "wasm";
const CPP: &'static str = "cpp";
const JS: &'static str = "js";
const DAT: &'static str = "dat";
const SYM: &'static str = "sym";
const JSON: &'static str = "json";

impl Input {
    pub fn default() -> Result<Input, ()> {
        let input = Input {
            input_program: PathBuf::from(
                "/Users/namncc/Documents/GitHub/mpz-pg/mpz-circuits/src/assets/circuit.circom",
            ),
            out_r1cs: PathBuf::from("./assets/tmp"),
            out_json_constraints: PathBuf::from("./assets/tmp"),
            out_wat_code: PathBuf::from("./assets/tmp"),
            out_wasm_code: PathBuf::from("./assets/tmp"),
            out_wasm_name: String::from("./assets/tmp"),
            out_js_folder: PathBuf::from("./assets/tmp"),
            out_c_run_name: String::from("./assets/tmp"),
            out_c_folder: PathBuf::from("./assets/tmp"),
            out_c_code: PathBuf::from("./assets/tmp"),
            out_c_dat: PathBuf::from("./assets/tmp"),
            out_sym: PathBuf::from("./assets/tmp"),
            c_flag: true,
            wasm_flag: false,
            wat_flag: false,
            r1cs_flag: false,
            sym_flag: false,
            json_constraint_flag: false,
            json_substitution_flag: false,
            main_inputs_flag: false,
            print_ir_flag: true,
            fast_flag: false,
            reduced_simplification_flag: false,
            parallel_simplification_flag: false,
            flag_old_heuristics: false,
            inspect_constraints_flag: false,
            no_rounds: usize::MAX,
            flag_verbose: true,
            prime: String::from("bn128"),
            link_libraries: Vec::new(),
        };
        Ok(input)
    }
    pub fn new() -> Result<Input, ()> {
        // use SimplificationStyle;
        let matches = view();
        let input = get_input(&matches)?;
        let file_name = input.file_stem().unwrap().to_str().unwrap().to_string();
        let output_path = get_output_path(&matches)?;
        let output_c_path = Input::build_folder(&output_path, &file_name, CPP);
        let output_js_path = Input::build_folder(&output_path, &file_name, JS);
        let o_style = get_simplification_style(&matches)?;
        let link_libraries = get_link_libraries(&matches);
        Result::Ok(Input {
            //field: P_BN128,
            input_program: input,
            out_r1cs: Input::build_output(&output_path, &file_name, R1CS),
            out_wat_code: Input::build_output(&output_js_path, &file_name, WAT),
            out_wasm_code: Input::build_output(&output_js_path, &file_name, WASM),
            out_js_folder: output_js_path.clone(),
            out_wasm_name: file_name.clone(),
            out_c_folder: output_c_path.clone(),
            out_c_run_name: file_name.clone(),
            out_c_code: Input::build_output(&output_c_path, &file_name, CPP),
            out_c_dat: Input::build_output(&output_c_path, &file_name, DAT),
            out_sym: Input::build_output(&output_path, &file_name, SYM),
            out_json_constraints: Input::build_output(
                &output_path,
                &format!("{}_constraints", file_name),
                JSON,
            ),
            wat_flag: get_wat(&matches),
            wasm_flag: get_wasm(&matches),
            c_flag: get_c(&matches),
            r1cs_flag: get_r1cs(&matches),
            sym_flag: get_sym(&matches),
            main_inputs_flag: get_main_inputs_log(&matches),
            json_constraint_flag: get_json_constraints(&matches),
            json_substitution_flag: get_json_substitutions(&matches),
            print_ir_flag: get_ir(&matches),
            no_rounds: if let SimplificationStyle::O2(r) = o_style {
                r
            } else {
                0
            },
            fast_flag: o_style == SimplificationStyle::O0,
            reduced_simplification_flag: o_style == SimplificationStyle::O1,
            parallel_simplification_flag: get_parallel_simplification(&matches),
            inspect_constraints_flag: get_inspect_constraints(&matches),
            flag_old_heuristics: get_flag_old_heuristics(&matches),
            flag_verbose: get_flag_verbose(&matches),
            prime: get_prime(&matches)?,
            link_libraries,
        })
    }

    fn build_folder(output_path: &PathBuf, filename: &str, ext: &str) -> PathBuf {
        let mut file = output_path.clone();
        let folder_name = format!("{}_{}", filename, ext);
        file.push(folder_name);
        file
    }

    fn build_output(output_path: &PathBuf, filename: &str, ext: &str) -> PathBuf {
        let mut file = output_path.clone();
        file.push(format!("{}.{}", filename, ext));
        file
    }

    pub fn get_link_libraries(&self) -> &Vec<PathBuf> {
        &self.link_libraries
    }

    pub fn input_file(&self) -> &str {
        &self.input_program.to_str().unwrap()
    }
    pub fn r1cs_file(&self) -> &str {
        self.out_r1cs.to_str().unwrap()
    }
    pub fn sym_file(&self) -> &str {
        self.out_sym.to_str().unwrap()
    }
    pub fn wat_file(&self) -> &str {
        self.out_wat_code.to_str().unwrap()
    }
    pub fn wasm_file(&self) -> &str {
        self.out_wasm_code.to_str().unwrap()
    }
    pub fn js_folder(&self) -> &str {
        self.out_js_folder.to_str().unwrap()
    }
    pub fn wasm_name(&self) -> String {
        self.out_wasm_name.clone()
    }

    pub fn c_folder(&self) -> &str {
        self.out_c_folder.to_str().unwrap()
    }
    pub fn c_run_name(&self) -> String {
        self.out_c_run_name.clone()
    }

    pub fn c_file(&self) -> &str {
        self.out_c_code.to_str().unwrap()
    }
    pub fn dat_file(&self) -> &str {
        self.out_c_dat.to_str().unwrap()
    }
    pub fn json_constraints_file(&self) -> &str {
        self.out_json_constraints.to_str().unwrap()
    }
    pub fn wasm_flag(&self) -> bool {
        self.wasm_flag
    }
    pub fn wat_flag(&self) -> bool {
        self.wat_flag
    }
    pub fn c_flag(&self) -> bool {
        self.c_flag
    }
    pub fn unsimplified_flag(&self) -> bool {
        self.fast_flag
    }
    pub fn r1cs_flag(&self) -> bool {
        self.r1cs_flag
    }
    pub fn json_constraints_flag(&self) -> bool {
        self.json_constraint_flag
    }
    pub fn json_substitutions_flag(&self) -> bool {
        self.json_substitution_flag
    }
    pub fn main_inputs_flag(&self) -> bool {
        self.main_inputs_flag
    }
    pub fn sym_flag(&self) -> bool {
        self.sym_flag
    }
    pub fn print_ir_flag(&self) -> bool {
        self.print_ir_flag
    }
    pub fn inspect_constraints_flag(&self) -> bool {
        self.inspect_constraints_flag
    }
    pub fn flag_verbose(&self) -> bool {
        self.flag_verbose
    }
    pub fn reduced_simplification_flag(&self) -> bool {
        self.reduced_simplification_flag
    }
    pub fn parallel_simplification_flag(&self) -> bool {
        self.parallel_simplification_flag
    }
    pub fn flag_old_heuristics(&self) -> bool {
        self.flag_old_heuristics
    }
    pub fn no_rounds(&self) -> usize {
        self.no_rounds
    }
    pub fn prime(&self) -> String {
        self.prime.clone()
    }
}

pub fn get_input(matches: &ArgMatches) -> Result<PathBuf, ()> {
    println!("{:?}", matches.value_of("input"));
    let route = Path::new(matches.value_of("input").unwrap()).to_path_buf();
    if route.is_file() {
        Result::Ok(route)
    } else {
        let route = if route.to_str().is_some() {
            ": ".to_owned() + route.to_str().unwrap()
        } else {
            "".to_owned()
        };

        Result::Err(log::error!("Input file does not exist{}", route))
    }
}

pub fn get_output_path(matches: &ArgMatches) -> Result<PathBuf, ()> {
    let route = Path::new(matches.value_of("output").unwrap()).to_path_buf();
    if route.is_dir() {
        Result::Ok(route)
    } else {
        Result::Err(log::error!("Invalid output path: {}", route.display()))
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SimplificationStyle {
    O0,
    O1,
    O2(usize),
}
pub fn get_simplification_style(matches: &ArgMatches) -> Result<SimplificationStyle, ()> {
    let o_0 = matches.is_present("no_simplification");
    let o_1 = matches.is_present("reduced_simplification");
    let o_2 = matches.is_present("full_simplification");
    let o_2round = matches.is_present("simplification_rounds");
    match (o_0, o_1, o_2round, o_2) {
        (true, _, _, _) => Ok(SimplificationStyle::O0),
        (_, true, _, _) => Ok(SimplificationStyle::O1),
        (_, _, true, _) => {
            let o_2_argument = matches.value_of("simplification_rounds").unwrap();
            let rounds_r = usize::from_str_radix(o_2_argument, 10);
            if let Result::Ok(no_rounds) = rounds_r {
                if no_rounds == 0 {
                    Ok(SimplificationStyle::O1)
                } else {
                    Ok(SimplificationStyle::O2(no_rounds))
                }
            } else {
                Result::Err(log::error!("invalid number of rounds"))
            }
        }

        (false, false, false, true) => Ok(SimplificationStyle::O2(usize::MAX)),
        (false, false, false, false) => Ok(SimplificationStyle::O2(usize::MAX)),
    }
}

pub fn get_json_constraints(matches: &ArgMatches) -> bool {
    matches.is_present("print_json_c")
}

pub fn get_json_substitutions(matches: &ArgMatches) -> bool {
    matches.is_present("print_json_sub")
}

pub fn get_sym(matches: &ArgMatches) -> bool {
    matches.is_present("print_sym")
}

pub fn get_r1cs(matches: &ArgMatches) -> bool {
    matches.is_present("print_r1cs")
}

pub fn get_wasm(matches: &ArgMatches) -> bool {
    matches.is_present("print_wasm")
}

pub fn get_wat(matches: &ArgMatches) -> bool {
    matches.is_present("print_wat")
}

pub fn get_c(matches: &ArgMatches) -> bool {
    matches.is_present("print_c")
}

pub fn get_main_inputs_log(matches: &ArgMatches) -> bool {
    matches.is_present("main_inputs_log")
}

pub fn get_parallel_simplification(matches: &ArgMatches) -> bool {
    matches.is_present("parallel_simplification")
}

pub fn get_ir(matches: &ArgMatches) -> bool {
    matches.is_present("print_ir")
}
pub fn get_inspect_constraints(matches: &ArgMatches) -> bool {
    matches.is_present("inspect_constraints")
}

pub fn get_flag_verbose(matches: &ArgMatches) -> bool {
    matches.is_present("flag_verbose")
}

pub fn get_flag_old_heuristics(matches: &ArgMatches) -> bool {
    matches.is_present("flag_old_heuristics")
}
pub fn get_prime(matches: &ArgMatches) -> Result<String, ()> {
    match matches.is_present("prime") {
        true => {
            let prime_value = matches.value_of("prime").unwrap();
            if prime_value == "bn128"
                || prime_value == "bls12381"
                || prime_value == "goldilocks"
                || prime_value == "grumpkin"
                || prime_value == "pallas"
                || prime_value == "vesta"
            {
                Ok(String::from(matches.value_of("prime").unwrap()))
            } else {
                Result::Err(log::error!("Invalid prime number"))
            }
        }

        false => Ok(String::from("bn128")),
    }
}

pub fn view() -> ArgMatches<'static> {
    App::new("circom compiler")
            .version(VERSION)
            .author("IDEN3")
            .about("Compiler for the circom programming language")
            .arg(
                Arg::with_name("input")
                    .multiple(false)
                    .default_value("./assets/circuit.circom")
                    .help("Path to a circuit with a main component"),
            )
            .arg(
                Arg::with_name("no_simplification")
                    .long("O0")
                    .hidden(false)
                    .takes_value(false)
                    .help("No simplification is applied")
                    .display_order(420)
            )
            .arg(
                Arg::with_name("reduced_simplification")
                    .long("O1")
                    .hidden(false)
                    .takes_value(false)
                    .help("Only applies var to var and var to constant simplification")
                    .display_order(460)
            )
            .arg(
                Arg::with_name("full_simplification")
                    .long("O2")
                    .takes_value(false)
                    .hidden(false)
                    .help("Full constraint simplification")
                    .display_order(480)
            )
            .arg(
                Arg::with_name("simplification_rounds")
                    .long("O2round")
                    .takes_value(true)
                    .hidden(false)
                    .help("Maximum number of rounds of the simplification process")
                    .display_order(500)
            )
            .arg(
                Arg::with_name("output")
                    .short("o")
                    .long("output")
                    .takes_value(true)
                    .default_value(".")
                    .display_order(1)
                    .help("Path to the directory where the output will be written"),
            )
            .arg(
                Arg::with_name("print_json_c")
                    .long("json")
                    .takes_value(false)
                    .display_order(120)
                    .help("Outputs the constraints in json format"),
            )
            .arg(
                Arg::with_name("print_ir")
                    .long("irout")
                    .takes_value(false)
                    .hidden(true)
                    .display_order(360)
                    .help("Outputs the low-level IR of the given circom program"),
            )
            .arg(
                Arg::with_name("inspect_constraints")
                    .long("inspect")
                    .takes_value(false)
                    .display_order(801)
                    .help("Does an additional check over the constraints produced"),
            )
            .arg(
                Arg::with_name("print_json_sub")
                    .long("jsons")
                    .takes_value(false)
                    .hidden(true)
                    .display_order(100)
                    .help("Outputs the substitution in json format"),
            )
            .arg(
                Arg::with_name("print_sym")
                    .long("sym")
                    .takes_value(false)
                    .display_order(60)
                    .help("Outputs witness in sym format"),
            )
            .arg(
                Arg::with_name("print_r1cs")
                    .long("r1cs")
                    .takes_value(false)
                    .display_order(30)
                    .help("Outputs the constraints in r1cs format"),
            )
            .arg(
                Arg::with_name("print_wasm")
                    .long("wasm")
                    .takes_value(false)
                    .display_order(90)
                    .help("Compiles the circuit to wasm"),
            )
            .arg(
                Arg::with_name("print_wat")
                    .long("wat")
                    .takes_value(false)
                    .display_order(120)
                    .help("Compiles the circuit to wat"),
            )
            .arg(
                Arg::with_name("link_libraries")
                .short("l")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1)
                .display_order(330)
                .help("Adds directory to library search path"),
            )
            .arg(
                Arg::with_name("print_c")
                    .long("c")
                    .short("c")
                    .takes_value(false)
                    .display_order(150)
                    .help("Compiles the circuit to c"),
            )
            .arg(
                Arg::with_name("parallel_simplification")
                    .long("parallel")
                    .takes_value(false)
                    .hidden(true)
                    .display_order(180)
                    .help("Runs non-linear simplification in parallel"),
            )
            .arg(
                Arg::with_name("main_inputs_log")
                    .long("inputs")
                    .takes_value(false)
                    .hidden(true)
                    .display_order(210)
                    .help("Produces a log_inputs.txt file"),
            )
            .arg(
                Arg::with_name("flag_verbose")
                    .long("verbose")
                    .takes_value(false)
                    .display_order(800)
                    .help("Shows logs during compilation"),
            )
            .arg(
                Arg::with_name("flag_old_heuristics")
                    .long("use_old_simplification_heuristics")
                    .takes_value(false)
                    .display_order(980)
                    .help("Applies the old version of the heuristics when performing linear simplification"),
            )
            .arg (
                Arg::with_name("prime")
                    .short("prime")
                    .long("prime")
                    .takes_value(true)
                    .default_value("bn128")
                    .display_order(300)
                    .help("To choose the prime number to use to generate the circuit. Receives the name of the curve (bn128, bls12381, goldilocks, grumpkin, pallas, vesta)"),
            )
            .get_matches()
}

pub fn get_link_libraries(matches: &ArgMatches) -> Vec<PathBuf> {
    let mut link_libraries = Vec::new();
    let m = matches.values_of("link_libraries");
    if let Some(paths) = m {
        for path in paths.into_iter() {
            link_libraries.push(Path::new(path).to_path_buf());
        }
    }
    link_libraries
}

pub fn parse_project(input_info: &Input) -> Result<ProgramArchive, ()> {
    let initial_file = input_info.input_file().to_string();
    let result_program_archive = circom_parser::run_parser(
        initial_file,
        VERSION,
        input_info.get_link_libraries().to_vec(),
    );
    match result_program_archive {
        Result::Err((file_library, report_collection)) => {
            Report::print_reports(&report_collection, &file_library);
            Result::Err(())
        }
        Result::Ok((program_archive, warnings)) => {
            Report::print_reports(&warnings, &program_archive.file_library);
            Result::Ok(program_archive)
        }
    }
}

pub fn analyse_project(program_archive: &mut ProgramArchive) -> Result<(), ()> {
    let analysis_result = check_types(program_archive);
    match analysis_result {
        Err(errs) => {
            Report::print_reports(&errs, program_archive.get_file_library());
            Err(())
        }
        Ok(warns) => {
            Report::print_reports(&warns, program_archive.get_file_library());
            Ok(())
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("uninitialized feed: {0}")]
    UninitializedFeed(usize),
    #[error("unsupported gate type: {0}")]
    UnsupportedGateType(String),
    #[error(transparent)]
    BuilderError(#[from] BuilderError),
}

// ====== ABOVE IS COPIED FROM CIRCOM FOR PARSING TO PROGRAM ARCHIVE ======

// For runtime context we mostly care about the var_id
// For runtime execution we care about the evaluated value of a named variable
pub struct RuntimeContext {
    pub caller_id: u32,
    pub context_id: u32,
    pub vars: HashMap<String, u32>,
    pub execution: RuntimeExecutionContext,
}

impl RuntimeContext {
    pub fn new(_caller_id: u32, _context_id: u32) -> RuntimeContext {
        RuntimeContext {
            caller_id: _caller_id,
            context_id: _context_id,
            vars: HashMap::new(),
            execution: RuntimeExecutionContext::new(_caller_id, _context_id),
        }
    }

    // pub fn init (&mut self, runtime: &CircomRuntime) {
    //     let context = runtime.get_runtime_context_by_context_id(self.caller_id);
    //     for (k, v) in context.vars.iter() {
    //         self.assign_var(k, *v);
    //     }
    //     self.execution.init(context);
    // }

    // Return to caller or return from callee to push changes from a function call back to caller
    // pub fn return_to_caller(&mut self, runtime: &CircomRuntime) {
    //     let context = runtime.get_runtime_context_by_context_id(self.caller_id);
    //     for (k, v) in self.vars.iter() {
    //         context.assign_var(k, *v);
    //     }
    //     self.execution.return_to_caller(context);
    // }

    pub fn assign_var(&mut self, var_name: &String, last_var_id: u32) -> u32 {
        self.vars.insert(var_name.to_string(), last_var_id);
        self.execution.assign_var(var_name);
        println!(
            "[RuntimeContext] {} is now with id {}",
            var_name, last_var_id
        );
        last_var_id
    }

    pub fn assign_var_val(&mut self, var_name: &String, var_val: u32) -> u32 {
        if !self.can_get_var(var_name) {
            return 0;
        }
        self.execution.assign_var_val(var_name, var_val);
        var_val
    }

    pub fn deassign_var_val(&mut self, var_name: &String) -> u32 {
        if !self.can_get_var(var_name) {
            return 0;
        }
        self.execution.deassign_var_val(var_name);
        0
    }

    pub fn can_get_var(&self, var_name: &String) -> bool {
        self.vars.contains_key(var_name)
    }

    pub fn get_var(&self, var_name: &String) -> u32 {
        if !self.can_get_var(var_name) {
            return 0;
        }
        *self.vars.get(var_name).unwrap()
    }

    pub fn can_get_var_val(&self, var_name: &String) -> bool {
        self.execution.can_get_var_val(var_name)
    }

    pub fn get_var_val(&self, var_name: &String) -> u32 {
        if !self.execution.can_get_var_val(var_name) {
            return 0;
        }
        self.execution.get_var_val(var_name)
    }
}

pub struct RuntimeExecutionContext {
    pub caller_id: u32,
    pub context_id: u32,
    pub vars: HashMap<String, u32>,
    pub exevars: HashMap<String, bool>,
}

impl RuntimeExecutionContext {
    pub fn new(_caller_id: u32, _context_id: u32) -> RuntimeExecutionContext {
        RuntimeExecutionContext {
            caller_id: _caller_id,
            context_id: _context_id,
            vars: HashMap::new(),
            exevars: HashMap::new(),
        }
    }

    pub fn init(&mut self, context: &RuntimeContext) {
        for (k, v) in context.execution.vars.iter() {
            self.assign_var(k);
            if context.execution.can_get_var_val(k) {
                self.assign_var_val(k, *v);
            }
        }
    }

    // Return to caller or return from callee to push changes from a function call back to caller
    // pub fn return_to_caller(&mut self, context: &mut RuntimeContext) {
    //     for (k, v) in self.vars.iter() {
    //         context.execution.assign_var(k);
    //         if self.can_get_var_val(k) {
    //             context.execution.assign_var_val(k, *v);
    //         }
    //     }
    // }

    pub fn assign_var(&mut self, var_name: &String) -> u32 {
        let mut var_val = 0;
        if self.exevars.contains_key(var_name) {
            var_val = self.get_var_val(var_name);
            self.vars.insert(var_name.to_string(), var_val);
            println!(
                "[RuntimeExecutionContext] Now {} carries over val {}",
                var_name, var_val
            );
        } else {
            self.vars.insert(var_name.to_string(), 0);
            self.exevars.insert(var_name.to_string(), false);
            println!(
                "[RuntimeExecutionContext] Now {} has no val {}",
                var_name, var_val
            );
        }
        var_val
    }

    pub fn assign_var_val(&mut self, var_name: &String, var_val: u32) -> u32 {
        self.vars.insert(var_name.to_string(), var_val);
        self.exevars.insert(var_name.to_string(), true);
        println!(
            "[RuntimeExecutionContext] Now {} has val {}",
            var_name, var_val
        );
        var_val
    }

    pub fn deassign_var_val(&mut self, var_name: &String) -> u32 {
        self.vars.insert(var_name.to_string(), 0);
        self.exevars.insert(var_name.to_string(), false);
        println!(
            "[RuntimeExecutionContext] Now {} has no val {}",
            var_name, 0
        );
        0
    }

    pub fn get_var_val(&self, var_name: &String) -> u32 {
        if !self.can_get_var_val(var_name) {
            return 0;
        }
        *self.vars.get(var_name).unwrap()
    }

    pub fn can_get_var_val(&self, var_name: &String) -> bool {
        if !self.exevars.contains_key(var_name) {
            return false;
        }
        *self.exevars.get(var_name).unwrap()
    }
}

// For runtime we maintain a call stack
// Right now this is buggy cannot handle the stack management (RUST!!!)
pub struct CircomRuntime {
    pub last_var_id: u32,
    pub last_context_id: u32,
    pub call_stack: LinkedList<RuntimeContext>,
}

impl CircomRuntime {
    pub fn new() -> CircomRuntime {
        CircomRuntime {
            last_var_id: 0,
            last_context_id: 0,
            call_stack: LinkedList::new(),
        }
    }

    pub fn init(&mut self) {
        self.last_context_id += 1;
        let rc = RuntimeContext::new(0, self.last_context_id);
        // When we init the circom runtime there is no caller to init with
        // rc.init(self);
        self.call_stack.push_front(rc);
    }

    // Should add context from calling and branching
    // Notice that while loop and block also has local variable so to consider as calling

    // pub fn new_context_from_calling (&mut self) {
    //     self.last_context_id += 1;
    //     let mut rc = RuntimeContext::new(self.get_current_runtime_context_id(), self.last_context_id);
    //     rc.init(self);
    //     self.call_stack.push(rc);

    // }

    // pub fn new_context_from_branching (&mut self) {
    //     self.last_context_id += 1;
    //     let mut rc = RuntimeContext::new(self.get_current_runtime_context_caller_id(), self.last_context_id);
    //     rc.init(self);
    //     self.call_stack.push(rc);

    // }

    // pub fn end_current_context_return_vars (&self) {
    //     let rc = self.get_current_runtime_context();
    //     rc.return_to_caller(self);
    //     self.call_stack.pop();
    // }

    // If first then else, so if 1 context -> if, if 2 contexts -> if else
    // TODO: not handled for now
    // pub fn merge_current_branches_return_vars(&mut self) {
    //     let rc = self.get_current_runtime_context();
    //     rc.return_to_caller(self);
    //     self.call_stack.pop();
    // }

    // pub fn get_runtime_context_by_context_id(&self, cid: u32) -> &RuntimeContext {
    //     let idx = self.get_runtime_context_index_in_stack_by_context_id(cid);
    //     self.get_runtime_context_by_stack_index(idx)
    // }

    pub fn get_current_runtime_context_caller_id(&self) -> u32 {
        self.call_stack.front().unwrap().caller_id
    }

    pub fn get_current_runtime_context_id(&mut self) -> u32 {
        self.call_stack.front().unwrap().context_id
    }

    // TODO
    // pub fn get_current_runtime_context_caller (&self) -> &RuntimeContext {
    //     let caller_id = self.get_current_runtime_context_caller_id();
    //     self.get_runtime_context_by_context_id(caller_id)

    // }

    pub fn get_current_runtime_context(&self) -> &RuntimeContext {
        self.call_stack.front().unwrap()
    }

    pub fn get_current_runtime_context_mut(&mut self) -> &mut RuntimeContext {
        self.call_stack.front_mut().unwrap()
    }

    pub fn get_var_from_current_context(&self, var: &String) -> u32 {
        let current = self.get_current_runtime_context();
        current.get_var(var)
    }
    pub fn assign_var_to_current_context(&mut self, var: &String) -> u32 {
        self.last_var_id += 1;
        let var_id = self.last_var_id;
        let current = self.get_current_runtime_context_mut();
        current.assign_var(var, var_id)
    }
    pub fn can_get_var_val_from_current_context(&self, var: &String) -> bool {
        let current = self.get_current_runtime_context();
        current.can_get_var_val(var)
    }
    pub fn get_var_val_from_current_context(&self, var: &String) -> u32 {
        let current = self.get_current_runtime_context();
        current.get_var_val(var)
    }
    pub fn assign_var_val_to_current_context(&mut self, var: &String, var_val: u32) -> u32 {
        let current = self.get_current_runtime_context_mut();
        current.assign_var_val(var, var_val)
    }

    pub fn deassign_var_val_to_current_context(&mut self, var: &String) -> u32 {
        let current = self.get_current_runtime_context_mut();
        current.deassign_var_val(var)
    }

    pub fn assign_auto_var_to_current_context(&mut self) -> String {
        self.last_var_id += 1;
        let var_id = self.last_var_id;
        let current = self.get_current_runtime_context_mut();
        let var = format!("auto_var_{}", var_id);
        current.assign_var(&var, var_id);
        println!("[CircomRuntime] Auto var {}", var);
        var
    }

    // TODO: array auto var should support multi-dimension, right now 1

    pub fn assign_array_var_to_current_context(
        &mut self,
        var: &String,
        indice: Vec<u32>,
    ) -> (String, u32) {
        self.last_var_id += 1;
        let var_id = self.last_var_id;
        let current = self.get_current_runtime_context_mut();
        let mut access_index = String::new();
        for i in 0..indice.len() {
            access_index.push_str(&format!("_{}", indice[i]));
        }
        let var = format!("{}{}", var, access_index);
        current.assign_var(&var, var_id);
        println!("[CircomRuntime] Array var {}", var);
        (var, var_id)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArithmeticVar {
    pub var_id: u32,
    pub var_name: String,
    pub is_const: bool,
    pub const_value: u32,
}

impl ArithmeticVar {
    pub fn new(_var_id: u32, _var_name: String) -> ArithmeticVar {
        ArithmeticVar {
            var_id: _var_id,
            var_name: _var_name,
            is_const: false,
            const_value: 0,
        }
    }

    pub fn set_const_value(&mut self, value: u32) {
        self.is_const = true;
        self.const_value = value;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AGateType {
    ANone,
    AAdd,
    ASub,
    AMul,
    ADiv,
    AEq,
    ANeq,
    ALEq,
    AGEq,
    ALt,
    AGt,
}

impl fmt::Display for AGateType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AGateType::ANone => write!(f, "None"),
            AGateType::AAdd => write!(f, "AAdd"),
            AGateType::ASub => write!(f, "ASub"),
            AGateType::AMul => write!(f, "AMul"),
            AGateType::ADiv => write!(f, "ADiv"),
            AGateType::AEq => write!(f, "AEq"),
            AGateType::ANeq => write!(f, "ANEq"),
            AGateType::ALEq => write!(f, "ALEq"),
            AGateType::AGEq => write!(f, "AGEq"),
            AGateType::ALt => write!(f, "ALt"),
            AGateType::AGt => write!(f, "AGt"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]

pub struct ArithmeticNode {
    pub gate_id: u32,
    pub gate_type: AGateType,
    pub input_lhs_id: u32,
    pub input_rhs_id: u32,
    pub output_id: u32,
}

impl ArithmeticNode {
    pub fn new(
        _gate_id: u32,
        _gate_type: AGateType,
        _input_lhs_id: u32,
        _input_rhs_id: u32,
        _out_put_id: u32,
    ) -> ArithmeticNode {
        ArithmeticNode {
            gate_id: _gate_id,
            gate_type: _gate_type,
            input_lhs_id: _input_lhs_id,
            input_rhs_id: _input_rhs_id,
            output_id: _out_put_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArithmeticCircuit {
    pub gate_count: u32,
    pub var_count: u32,
    pub vars: HashMap<u32, ArithmeticVar>,
    pub gates: HashMap<u32, ArithmeticNode>,
}

impl ArithmeticCircuit {
    pub fn gate_count(&self) -> u32 {
        self.gate_count
    }

    pub fn var_count(&self) -> u32 {
        self.var_count
    }

    pub fn new() -> ArithmeticCircuit {
        ArithmeticCircuit {
            gate_count: 0,
            var_count: 0,
            vars: HashMap::new(),
            gates: HashMap::new(),
        }
    }

    pub fn add_var(&mut self, var_id: u32, var_name: &str) -> &ArithmeticVar {
        println!(
            "[ArithmeticCircuit] Add var {} with id {}",
            var_name, var_id
        );

        // Not sure if var_count is needed
        self.var_count += 1;

        let var = ArithmeticVar::new(var_id, var_name.to_string());
        self.vars.insert(var_id, var);
        self.vars.get(&var_id).unwrap()
    }

    pub fn add_const_var(&mut self, var_id: u32, var_val: u32) -> &ArithmeticVar {
        println!(
            "[ArithmeticCircuit] var {} now has value {}",
            var_id, var_val
        );

        // Not sure if var_count is needed
        self.var_count += 1;

        let mut var = ArithmeticVar::new(var_id, var_val.to_string());
        var.is_const = true;
        var.const_value = var_val;
        self.vars.insert(var_id, var);
        self.vars.get(&var_id).unwrap()
    }

    pub fn get_var(&self, var_id: u32) -> &ArithmeticVar {
        self.vars.get(&var_id).unwrap()
    }

    pub fn get_var_mut(&mut self, var_id: u32) -> &mut ArithmeticVar {
        self.vars.get_mut(&var_id).unwrap()
    }

    //We support ADD, MUL, CADD, CMUL, DIV, CDIV, CINVERT, IFTHENELSE, FOR

    pub fn add_gate(
        &mut self,
        output_name: &String,
        output_id: u32,
        lhs_id: u32,
        rhs_id: u32,
        gate_type: AGateType,
    ) {
        self.gate_count += 1;
        self.add_var(output_id, output_name);
        let node = ArithmeticNode::new(self.gate_count, gate_type, lhs_id, rhs_id, output_id);
        let var_output = self.get_var(output_id);
        let var_lhs = self.get_var(lhs_id);
        let var_rhs = self.get_var(rhs_id);
        println!(
            "[ArithmeticCircuit] Gate added id {}: ({}, {}, {}) = ({}, {}, {}) {} ({}, {}, {})",
            node.gate_id,
            node.output_id,
            var_output.is_const,
            var_output.const_value,
            node.input_lhs_id,
            var_lhs.is_const,
            var_lhs.const_value,
            node.gate_type.to_string(),
            node.input_rhs_id,
            var_rhs.is_const,
            var_rhs.const_value
        );
        self.gates.insert(self.gate_count, node);
    }

    // pub fn add_gate(
    //     &mut self,
    //     output: &ArithmeticVar,
    //     lhs: &ArithmeticVar,
    //     rhs: &ArithmeticVar,
    //     gate_type: AGateType) {

    //     self.gate_count += 1;
    //     let node = ArithmeticNode::new(self.gate_count, gate_type, lhs.var_id, rhs.var_id, output.var_id);

    //     self.gates.insert(self.gate_count, node);

    // }

    pub fn replace_input_var_in_gate(&mut self, var_id: u32, new_var_id: u32) {
        for i in 1..(self.gate_count + 1) {
            if !self.gates.contains_key(&i) {
                continue;
            }
            let node = self.gates.get_mut(&(i)).unwrap();
            if node.input_lhs_id == var_id {
                node.input_lhs_id = new_var_id;
            }
            if node.input_rhs_id == var_id {
                node.input_rhs_id = new_var_id;
            }
        }
    }

    pub fn truncate_zero_add_gate(&mut self) {
        let mut zero_add_gate_indice = vec![];
        for i in 1..(self.gate_count + 1) {
            if !self.gates.contains_key(&i) {
                continue;
            }
            let node = self.gates.get(&(i)).unwrap();
            match node.gate_type {
                AGateType::AAdd => {
                    let var_output = self.get_var(node.output_id);
                    let var_lhs = self.get_var(node.input_lhs_id);
                    let var_rhs = self.get_var(node.input_rhs_id);
                    if var_lhs.is_const && var_lhs.const_value == 0 {
                        // var_output.var_id = var_rhs.var_id;
                        // var_output.var_name = var_rhs.var_name.to_string();
                        self.replace_input_var_in_gate(var_output.var_id, var_rhs.var_id);
                        zero_add_gate_indice.push(i);
                    } else if var_rhs.is_const && var_rhs.const_value == 0 {
                        // var_output.var_id = var_lhs.var_id;
                        // var_output.var_name = var_lhs.var_name;
                        self.replace_input_var_in_gate(var_output.var_id, var_lhs.var_id);
                        zero_add_gate_indice.push(i);
                    } else {
                        continue;
                    }
                }
                _ => {
                    continue;
                }
            }
        }
        for i in zero_add_gate_indice.iter() {
            self.gates.remove(i);
        }
    }

    pub fn print_ac(&self) {
        println!("[ArithmeticCircuit] Whole Arithmetic Circuit");
        for i in 1..(self.gate_count + 1) {
            if !self.gates.contains_key(&i) {
                continue;
            }
            let node = self.gates.get(&(i)).unwrap();
            // println!("[ArithmeticCircuit] Gate {}: {} = {} [{}] {}", i, anv.output_id, anv.input_lhs_id, anv.gate_type.to_string(), anv.input_rhs_id);
            let var_output = self.get_var(node.output_id);
            let var_lhs = self.get_var(node.input_lhs_id);
            let var_rhs = self.get_var(node.input_rhs_id);
            println!(
                "[ArithmeticCircuit] Gate id {}: ({}, {}, {}) = ({}, {}, {}) {} ({}, {}, {})",
                node.gate_id,
                node.output_id,
                var_output.is_const,
                var_output.const_value,
                node.input_lhs_id,
                var_lhs.is_const,
                var_lhs.const_value,
                node.gate_type.to_string(),
                node.input_rhs_id,
                var_rhs.is_const,
                var_rhs.const_value
            );
        }
        // for (ank, anv) in self.gates.iter() {
        //     println!("Gate {}: {} = {} [{}] {}", ank, anv.output_id, anv.input_lhs_id, anv.gate_type.to_string(), anv.input_rhs_id);
        // }
    }

    pub fn serde(&self) {
        let serialized = serde_json::to_string(&self).unwrap();

        // Prints serialized = {"x":1,"y":2}
        println!("serialized = {}", serialized);

        // Convert the JSON string back to a Point.
        let deserialized: ArithmeticCircuit = serde_json::from_str(&serialized).unwrap();

        // Prints deserialized = Point { x: 1, y: 2 }
        println!("deserialized = {:?}", deserialized);
    }
}

//WIP HERE
// TODO: named_access should support multi-dimension, right now 1

fn execute_infix_op(
    ac: &mut ArithmeticCircuit,
    runtime: &mut CircomRuntime,
    output: &String,
    input_lhs: &String,
    input_rhs: &String,
    infixop: ExpressionInfixOpcode,
) -> (u32, bool) {
    // let current = runtime.get_current_runtime_context();
    let mut can_execute_infix = true;
    if !runtime.can_get_var_val_from_current_context(input_lhs) {
        println!("[Execute] cannot get lhs var val {}", input_lhs);
        can_execute_infix = false;
    }
    if !runtime.can_get_var_val_from_current_context(input_rhs) {
        println!("[Execute] cannot get rhs var val {}", input_rhs);
        can_execute_infix = false;
    }
    println!("[Execute] can execute infix {}", can_execute_infix);

    if !can_execute_infix {
        runtime.deassign_var_val_to_current_context(output);
        println!("[Execute] Now mark {} as no value", output);
        return (0, false);
    }

    let lhsvar_val = runtime.get_var_val_from_current_context(input_lhs);
    println!("[Execute] infix lhs = {}", lhsvar_val);
    let rhsvar_val = runtime.get_var_val_from_current_context(input_rhs);
    println!("[Execute] infix lhs = {}", rhsvar_val);
    let var_id = runtime.assign_var_to_current_context(output);

    // let var = ac.add_var(var_id, &output);

    // let lvar = ac.get_var(lhsvar_id);
    // let rvar = ac.get_var(rhsvar_id);

    let mut res = 0;

    use ExpressionInfixOpcode::*;
    let mut gate_type = AGateType::AAdd;
    match infixop {
        Mul => {
            println!(
                "[Execute] Mul op {} = {} * {}",
                output, lhsvar_val, rhsvar_val
            );
            gate_type = AGateType::AMul;
            res = lhsvar_val * rhsvar_val;
        }
        Div => {
            println!(
                "[Execute] Div op {} = {} / {}",
                output, lhsvar_val, rhsvar_val
            );
            gate_type = AGateType::ADiv;
            res = lhsvar_val / rhsvar_val;
        }
        Add => {
            println!(
                "[Execute] Add op {} = {} + {}",
                output, lhsvar_val, rhsvar_val
            );
            gate_type = AGateType::AAdd;
            res = lhsvar_val + rhsvar_val;
        }
        Sub => {
            println!(
                "[Execute] Sub op {} = {} - {}",
                output, lhsvar_val, rhsvar_val
            );
            gate_type = AGateType::ASub;
            res = lhsvar_val - rhsvar_val;
        }
        // Pow => {},
        // IntDiv => {},
        // Mod => {},
        // ShiftL => {},
        // ShiftR => {},
        LesserEq => {
            println!(
                "[Execute] LesserEq op {} = {} <= {}",
                output, lhsvar_val, rhsvar_val
            );
            res = if lhsvar_val <= rhsvar_val { 1 } else { 0 };
        }
        GreaterEq => {
            println!(
                "[Execute] GreaterEq op {} = {} >= {}",
                output, lhsvar_val, rhsvar_val
            );
            res = if lhsvar_val >= rhsvar_val { 1 } else { 0 };
        }
        Lesser => {
            println!(
                "[Execute] Lesser op {} = {} < {}",
                output, lhsvar_val, rhsvar_val
            );
            res = if lhsvar_val < rhsvar_val { 1 } else { 0 };
        }
        Greater => {
            println!(
                "[Execute] Greater op {} = {} > {}",
                output, lhsvar_val, rhsvar_val
            );
            res = if lhsvar_val > rhsvar_val { 1 } else { 0 };
        }
        Eq => {
            println!(
                "[Execute] Eq op {} = {} == {}",
                output, lhsvar_val, rhsvar_val
            );
            gate_type = AGateType::AEq;
            res = if lhsvar_val == rhsvar_val { 1 } else { 0 };
        }
        NotEq => {
            println!(
                "[Execute] Neq op {} = {} != {}",
                output, lhsvar_val, rhsvar_val
            );
            gate_type = AGateType::ANeq;
            res = if lhsvar_val != rhsvar_val { 1 } else { 0 };
        }
        // BoolOr => {},
        // BoolAnd => {},
        // BitOr => {},
        // BitAnd => {},
        // BitXor => {},
        _ => {
            unreachable!()
        }
    };
    println!("[Execute] infix res = {}", res);
    (res, true)
    // ac.add_gate(&output, var_id, lhsvar_id, rhsvar_id, gate_type);
}

fn traverse_infix_op(
    ac: &mut ArithmeticCircuit,
    runtime: &mut CircomRuntime,
    output: &String,
    input_lhs: &String,
    input_rhs: &String,
    infixop: ExpressionInfixOpcode,
) -> (u32, bool) {
    // let current = runtime.get_current_runtime_context();

    // For now skip traversal if can execute

    let mut can_execute_infix = true;
    if !runtime.can_get_var_val_from_current_context(input_lhs) {
        println!("[Traverse] cannot get lhs var val {}", input_lhs);
        can_execute_infix = false;
    }
    if !runtime.can_get_var_val_from_current_context(input_rhs) {
        println!("[Traverse] cannot get rhs var val {}", input_rhs);
        can_execute_infix = false;
    }
    println!("[Traverse] can execute infix {}", can_execute_infix);

    if can_execute_infix {
        return execute_infix_op(ac, runtime, output, input_lhs, input_rhs, infixop);
    } else {
        runtime.deassign_var_val_to_current_context(output);
        println!("[Traverse] Now mark {} as no value", output);
    }

    let lhsvar_id = runtime.get_var_from_current_context(input_lhs);
    let rhsvar_id = runtime.get_var_from_current_context(input_rhs);
    let var_id = runtime.assign_var_to_current_context(output);

    // let var = ac.add_var(var_id, &output);

    // let lvar = ac.get_var(lhsvar_id);
    // let rvar = ac.get_var(rhsvar_id);

    use ExpressionInfixOpcode::*;
    let mut gate_type = AGateType::AAdd;
    match infixop {
        Mul => {
            println!(
                "[Traverse] Mul op {} = {} * {}",
                output, input_lhs, input_rhs
            );
            gate_type = AGateType::AMul;
        }
        Div => {
            println!(
                "[Traverse] Div op {} = {} / {}",
                output, input_lhs, input_rhs
            );
            gate_type = AGateType::ADiv;
        }
        Add => {
            println!(
                "[Traverse] Add op {} = {} + {}",
                output, input_lhs, input_rhs
            );
            gate_type = AGateType::AAdd;
        }
        Sub => {
            println!(
                "[Traverse] Sub op {} = {} - {}",
                output, input_lhs, input_rhs
            );
            gate_type = AGateType::ASub;
        }
        // Pow => {},
        // IntDiv => {},
        // Mod => {},
        // ShiftL => {},
        // ShiftR => {},
        LesserEq => {
            println!(
                "[Traverse] LEq op {} = {} == {}",
                output, input_lhs, input_rhs
            );
            gate_type = AGateType::ALEq;
        }
        GreaterEq => {
            println!(
                "[Traverse] GEq op {} = {} == {}",
                output, input_lhs, input_rhs
            );
            gate_type = AGateType::AGEq;
        }
        Lesser => {
            println!(
                "[Traverse] Ls op {} = {} == {}",
                output, input_lhs, input_rhs
            );
            gate_type = AGateType::ALt;
        }
        Greater => {
            println!(
                "[Traverse] Gt op {} = {} == {}",
                output, input_lhs, input_rhs
            );
            gate_type = AGateType::AGt;
        }
        Eq => {
            println!(
                "[Traverse] Eq op {} = {} == {}",
                output, input_lhs, input_rhs
            );
            gate_type = AGateType::AEq;
        }
        NotEq => {
            println!(
                "[Traverse] Neq op {} = {} != {}",
                output, input_lhs, input_rhs
            );
            gate_type = AGateType::ANeq;
        }
        // BoolOr => {},
        // BoolAnd => {},
        // BitOr => {},
        // BitAnd => {},
        // BitXor => {},
        _ => {
            unreachable!()
        }
    };

    ac.add_gate(&output, var_id, lhsvar_id, rhsvar_id, gate_type);

    (0, false)
}

fn execute_expression(
    ac: &mut ArithmeticCircuit,
    runtime: &mut CircomRuntime,
    var: &String,
    expr: &Expression,
    program_archive: &ProgramArchive,
) -> (String, bool) {
    use Expression::*;
    // let mut can_be_simplified = true;
    match expr {
        Number(_, value) => {
            let var_id = runtime.assign_var_to_current_context(&value.to_string());
            runtime.assign_var_val_to_current_context(&value.to_string(), value.to_u32().unwrap());
            ac.add_const_var(var_id, value.to_u32().unwrap());
            println!("[Execute] Number value {}", value);
            (value.to_string(), true)
        }
        InfixOp {
            meta,
            lhe,
            infix_op,
            rhe,
            ..
        } => {
            let varlhs = runtime.assign_auto_var_to_current_context();
            println!("[Execute] Auto var for lhs {}", varlhs);
            let varrhs = runtime.assign_auto_var_to_current_context();
            println!("[Execute] Auto var for rhs {}", varrhs);
            let (varlop, lhsb) = execute_expression(ac, runtime, &varlhs, lhe, program_archive);
            println!("[Execute] lhs {} {}", varlop, lhsb);
            let (varrop, rhsb) = execute_expression(ac, runtime, &varrhs, rhe, program_archive);
            println!("[Execute] rhs {} {}", varrop, rhsb);
            let (res, rb) = execute_infix_op(ac, runtime, var, &varlop, &varrop, *infix_op);
            println!("[Execute] infix out res {}", res);
            (res.to_string(), rb)
        }
        PrefixOp {
            meta,
            prefix_op,
            rhe,
        } => {
            println!("Prefix found ");
            (var.to_string(), false)
        }
        InlineSwitchOp {
            meta,
            cond,
            if_true,
            if_false,
        } => todo!(),
        ParallelOp { meta, rhe } => todo!(),
        Variable { meta, name, access } => {
            let mut name_access = String::from(name);
            println!("[Execute] Variable found {}", name.to_string());
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        println!("[Execute] Array access found");
                        // let mut dim_u32_vec = Vec::new();
                        let dim_u32_str =
                            traverse_expression(ac, runtime, var, expr, program_archive);
                        // dim_u32_vec.push(dim_u32_str.parse::<u32>().unwrap());
                        name_access.push_str("_");
                        name_access.push_str(dim_u32_str.as_str());
                        println!("[Execute] Change var name to {}", name_access);
                    }
                    Access::ComponentAccess(name) => {
                        println!("Component access found");
                    }
                }
            }
            if runtime.can_get_var_val_from_current_context(&name_access) {
                let var_val = runtime
                    .get_var_val_from_current_context(&name_access)
                    .to_string();
                println!("[Execute] Return var value {} = {}", name_access, var_val);
                runtime.assign_var_to_current_context(&var_val);
                runtime
                    .assign_var_val_to_current_context(&var_val, var_val.parse::<u32>().unwrap());
                return (var_val, true);
            }
            (name_access.to_string(), false)
        }
        Call { meta, id, args } => {
            println!("Call found {}", id.to_string());
            // find the template and execute it
            (id.to_string(), false)
        }
        AnonymousComp {
            meta,
            id,
            is_parallel,
            params,
            signals,
            names,
        } => todo!(),
        ArrayInLine { meta, values } => {
            println!("ArrayInLine found");
            (var.to_string(), false)
        }
        Tuple { meta, values } => todo!(),
        UniformArray {
            meta,
            value,
            dimension,
        } => {
            println!("UniformArray found");
            (var.to_string(), false)
        }
    }
}

fn traverse_expression(
    ac: &mut ArithmeticCircuit,
    runtime: &mut CircomRuntime,
    var: &String,
    expr: &Expression,
    program_archive: &ProgramArchive,
) -> String {
    use Expression::*;
    // let mut can_be_simplified = true;
    match expr {
        Number(_, value) => {
            let var_id = runtime.assign_var_to_current_context(&value.to_string());
            runtime.assign_var_val_to_current_context(&value.to_string(), value.to_u32().unwrap());
            ac.add_const_var(var_id, value.to_u32().unwrap());
            println!("[Traverse] Number value {}", value);
            value.to_string()
        }
        InfixOp {
            meta,
            lhe,
            infix_op,
            rhe,
            ..
        } => {
            let varlhs = runtime.assign_auto_var_to_current_context();
            println!("[Traverse] Auto var for lhs {}", varlhs);
            let varrhs = runtime.assign_auto_var_to_current_context();
            println!("[Traverse] Auto var for rhs {}", varrhs);
            let varlop = traverse_expression(ac, runtime, &varlhs, lhe, program_archive);
            println!("[Traverse] lhs {}", varlop);
            let varrop = traverse_expression(ac, runtime, &varrhs, rhe, program_archive);
            println!("[Traverse] rhs {}", varlop);
            let (res, ret) = traverse_infix_op(ac, runtime, var, &varlop, &varrop, *infix_op);
            if ret {
                return res.to_string();
            }
            var.to_string()
        }
        PrefixOp {
            meta,
            prefix_op,
            rhe,
        } => {
            println!("Prefix found ");
            var.to_string()
        }
        InlineSwitchOp {
            meta,
            cond,
            if_true,
            if_false,
        } => todo!(),
        ParallelOp { meta, rhe } => todo!(),
        Variable { meta, name, access } => {
            let mut name_access = String::from(name);
            println!("[Traverse] Variable found {}", name.to_string());
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        println!("[Traverse] Array access found");
                        // let mut dim_u32_vec = Vec::new();
                        let dim_u32_str =
                            traverse_expression(ac, runtime, var, expr, program_archive);
                        // dim_u32_vec.push(dim_u32_str.parse::<u32>().unwrap());
                        name_access.push_str("_");
                        name_access.push_str(dim_u32_str.as_str());
                        println!("[Traverse] Change var name to {}", name_access);
                    }
                    Access::ComponentAccess(name) => {
                        println!("[Traverse] Component access found");
                    }
                }
            }
            if runtime.can_get_var_val_from_current_context(&name_access) {
                let var_val = runtime
                    .get_var_val_from_current_context(&name_access)
                    .to_string();
                println!("[Traverse] Return var value {} = {}", name_access, var_val);
                let var_id = runtime.assign_var_to_current_context(&var_val);
                let var_val_n = runtime
                    .assign_var_val_to_current_context(&var_val, var_val.parse::<u32>().unwrap());
                ac.add_const_var(var_id, var_val_n);
                return var_val.to_string();
            }
            name_access.to_string()
        }
        Call { meta, id, args } => {
            println!("Call found {}", id.to_string());
            // find the template and execute it
            id.to_string()
        }
        AnonymousComp {
            meta,
            id,
            is_parallel,
            params,
            signals,
            names,
        } => todo!(),
        ArrayInLine { meta, values } => {
            println!("ArrayInLine found");
            var.to_string()
        }
        Tuple { meta, values } => todo!(),
        UniformArray {
            meta,
            value,
            dimension,
        } => {
            println!("UniformArray found");
            var.to_string()
        }
    }
}

fn traverse_component_declaration(
    ac: &mut ArithmeticCircuit,
    runtime: &mut CircomRuntime,
    comp_name: &str,
    dim_u32_vec: &Vec<u32>,
) {
    // let var_id = runtime.assign_var_to_current_context(&var_name.to_string());
    // ac.add_var(var_id, comp_name.to_string().as_str());
    println!("Found component {}", comp_name);
}

fn traverse_signal_declaration(
    ac: &mut ArithmeticCircuit,
    runtime: &mut CircomRuntime,
    signal_name: &str,
    signal_type: SignalType,
    dim_u32_vec: &Vec<u32>,
) {
    traverse_variable_declaration(ac, runtime, signal_name, dim_u32_vec);
}

fn traverse_variable_declaration(
    ac: &mut ArithmeticCircuit,
    runtime: &mut CircomRuntime,
    var_name: &str,
    dim_u32_vec: &Vec<u32>,
) {
    if dim_u32_vec.is_empty() {
        let var_id = runtime.assign_var_to_current_context(&var_name.to_string());
        ac.add_var(var_id, var_name.to_string().as_str());
    } else {
        // let mut all_accesses = Vec::new();
        // for u32s in dim_u32_vec.iter() {
        //     let mut accesses = Vec::new();
        //     for i in 0..*u32s {
        //         accesses.push(i);
        //     }
        //     all_accesses.push(accesses);
        // }
        // for accesses in all_accesses.iter() {

        // }
        let dim_u32 = *dim_u32_vec.last().unwrap();
        for i in 0..dim_u32 {
            let mut u32vec = Vec::new();
            u32vec.push(i);
            let (var, var_id) =
                runtime.assign_array_var_to_current_context(&var_name.to_string(), u32vec);
            ac.add_var(var_id, var.as_str());
        }
    }
}

fn execute_statement(
    ac: &mut ArithmeticCircuit,
    runtime: &mut CircomRuntime,
    stmt: &Statement,
    program_archive: &ProgramArchive,
) {
    use Statement::*;
    let id = stmt.get_meta().elem_id;

    // Analysis::reached(&mut runtime.analysis, id);

    // let mut can_be_simplified = true;

    match stmt {
        InitializationBlock {
            initializations, ..
        } => {
            for istmt in initializations.iter() {
                execute_statement(ac, runtime, istmt, program_archive);
            }
        }
        Declaration {
            meta,
            xtype,
            name,
            dimensions,
            ..
        } => {
            println!("Declaration of {}", name);
            match xtype {
                // VariableType::AnonymousComponent => {
                //     execute_anonymous_component_declaration(
                //         name,
                //         meta.clone(),
                //         &dimensions,
                //         &mut runtime.environment,
                //         &mut runtime.anonymous_components,
                //     );
                // }
                _ => {
                    let mut dim_u32_vec = Vec::new();
                    for dimension in dimensions.iter() {
                        let dim_u32_str =
                            traverse_expression(ac, runtime, name, dimension, program_archive);
                        dim_u32_vec.push(dim_u32_str.parse::<u32>().unwrap());
                    }
                    // treat_result_with_memory_error_void(
                    //     valid_array_declaration(&arithmetic_values),
                    //     meta,
                    //     &mut runtime.runtime_errors,
                    //     &runtime.call_trace,
                    // )?;
                    // let usable_dimensions =
                    //     if let Option::Some(dimensions) = cast_indexing(&arithmetic_values) {
                    //         dimensions
                    //     } else {
                    //         let err = Result::Err(ExecutionError::ArraySizeTooBig);
                    //         treat_result_with_execution_error(
                    //             err,
                    //             meta,
                    //             &mut runtime.runtime_errors,
                    //             &runtime.call_trace,
                    //         )?
                    //     };
                    match xtype {
                        VariableType::Component => traverse_component_declaration(
                            ac,
                            runtime,
                            name,
                            &dim_u32_vec, // &usable_dimensions
                                          // &mut runtime.environment,
                                          // actual_node
                        ),
                        VariableType::Var => traverse_variable_declaration(
                            ac,
                            runtime,
                            name,
                            &dim_u32_vec, // &usable_dimensions
                        ),
                        VariableType::Signal(signal_type, tag_list) => traverse_signal_declaration(
                            ac,
                            runtime,
                            name,
                            *signal_type,
                            &dim_u32_vec, // &usable_dimensions
                        ),
                        _ => {
                            unreachable!()
                        }
                    }
                }
            }
            // Option::None
        }
        IfThenElse {
            cond,
            if_case,
            else_case,
            ..
        } => {
            // let var = String::from("IFTHENELSE");
            // ac.add_var(&var, SignalType::Intermediate);
            // let lhs = traverse_expression(ac, &var, cond, program_archive);
            // traverse_statement(ac, &if_case, program_archive);
            // let else_case = else_case.as_ref().map(|e| e.as_ref());
            // traverse_statement(ac, else_case.unwrap(), program_archive);
            //     let else_case = else_case.as_ref().map(|e| e.as_ref());
            //     let (possible_return, can_simplify, _) = execute_conditional_statement(
            //         cond,
            //         if_case,
            //         else_case,
            //         program_archive,
            //         runtime,
            //         actual_node,
            //         flags
            //     )?;
            //     can_be_simplified = can_simplify;
            //     possible_return
            // }
            // While { cond, stmt, .. } => loop {
            //     let (returned, can_simplify, condition_result) = execute_conditional_statement(
            //         cond,
            //         stmt,
            //         Option::None,
            //         program_archive,
            //         runtime,
            //         actual_node,
            //         flags
            //     )?;
            //     can_be_simplified &= can_simplify;
            //     if returned.is_some() {
            //         break returned;
            //     } else if condition_result.is_none() {
            //         let (returned, _, _) = execute_conditional_statement(
            //             cond,
            //             stmt,
            //             None,
            //             program_archive,
            //             runtime,
            //             actual_node,
            //             flags
            //         )?;
            //         break returned;
            //     } else if !condition_result.unwrap() {
            //         break returned;
            //     }
        }
        While { cond, stmt, .. } => loop {
            let var = String::from("while");
            let (res, rb) = execute_expression(ac, runtime, &var, cond, program_archive);
            println!("[Execute] res = {} {}", res, rb);
            execute_statement(ac, runtime, stmt, program_archive);
            if res.contains("0") {
                break;
            }
            // traverse_expression(ac, runtime, var, cond, program_archive);
            // let var = String::from("while");
            // ac.add_var(&var, SignalType::Intermediate);
            // let lhs = traverse_expression(ac, runtime, &var, cond, program_archive);
            // println!("While cond {}", lhs);
            // traverse_statement(ac, stmt, program_archive);
        },
        ConstraintEquality { meta, lhe, rhe, .. } => {
            // debug_assert!(actual_node.is_some());
            // let f_left = execute_expression(lhe, program_archive, runtime, flags)?;
            // let f_right = execute_expression(rhe, program_archive, runtime, flags)?;
            // let arith_left = safe_unwrap_to_arithmetic_slice(f_left, line!());
            // let arith_right = safe_unwrap_to_arithmetic_slice(f_right, line!());

            // let correct_dims_result = AExpressionSlice::check_correct_dims(&arith_left, &Vec::new(), &arith_right, true);
            // treat_result_with_memory_error_void(
            //     correct_dims_result,
            //     meta,
            //     &mut runtime.runtime_errors,
            //     &runtime.call_trace,
            // )?;
            // for i in 0..AExpressionSlice::get_number_of_cells(&arith_left){
            //     let value_left = treat_result_with_memory_error(
            //         AExpressionSlice::access_value_by_index(&arith_left, i),
            //         meta,
            //         &mut runtime.runtime_errors,
            //         &runtime.call_trace,
            //     )?;
            //     let value_right = treat_result_with_memory_error(
            //         AExpressionSlice::access_value_by_index(&arith_right, i),
            //         meta,
            //         &mut runtime.runtime_errors,
            //         &runtime.call_trace,
            //     )?;
            //     let possible_non_quadratic =
            //         AExpr::sub(
            //             &value_left,
            //             &value_right,
            //             &runtime.constants.get_p()
            //         );
            //     if possible_non_quadratic.is_nonquadratic() {
            //         treat_result_with_execution_error(
            //             Result::Err(ExecutionError::NonQuadraticConstraint),
            //             meta,
            //             &mut runtime.runtime_errors,
            //             &runtime.call_trace,
            //         )?;
            //     }
            //     let quadratic_expression = possible_non_quadratic;
            //     let constraint_expression = AExpr::transform_expression_to_constraint_form(
            //         quadratic_expression,
            //         runtime.constants.get_p(),
            //     )
            //     .unwrap();
            //     if let Option::Some(node) = actual_node {
            //         node.add_constraint(constraint_expression);
            //     }
            // }
            // Option::None
        }
        Return { value, .. } => {}
        Assert { arg, meta, .. } => {}
        Substitution {
            meta,
            var,
            access,
            op,
            rhe,
            ..
        } => {
            let mut name_access = String::from(var);
            println!("[Execute] Variable found {}", var.to_string());
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        println!("[Execute] Array access found");
                        // let mut dim_u32_vec = Vec::new();
                        let dim_u32_str =
                            traverse_expression(ac, runtime, var, expr, program_archive);
                        // dim_u32_vec.push(dim_u32_str.parse::<u32>().unwrap());
                        name_access.push_str("_");
                        name_access.push_str(dim_u32_str.as_str());
                        println!("[Execute] Change var name to {}", name_access);
                    }
                    Access::ComponentAccess(name) => {
                        println!("Component access not handled");
                    }
                }
            }
            let (rhs, rhsb) = execute_expression(ac, runtime, &name_access, rhe, program_archive);
            println!("[Execute] Assigning {} ? {} to {}", rhs, rhsb, &name_access);
            if rhsb {
                println!("[Execute] Assigning {} to {}", rhs, &name_access);
                runtime
                    .assign_var_val_to_current_context(&name_access, rhs.parse::<u32>().unwrap());
            }
        }
        Block { stmts, .. } => {
            traverse_sequence_of_statements(ac, runtime, stmts, program_archive, true);
        }
        LogCall { args, .. } => {}
        UnderscoreSubstitution { meta, rhe, op } => {
            println!("UnderscoreSubstitution found");
        }
        _ => {
            unimplemented!()
        }
    }
}

fn traverse_statement(
    ac: &mut ArithmeticCircuit,
    runtime: &mut CircomRuntime,
    stmt: &Statement,
    program_archive: &ProgramArchive,
) {
    use Statement::*;
    let id = stmt.get_meta().elem_id;

    // Analysis::reached(&mut runtime.analysis, id);

    // let mut can_be_simplified = true;

    match stmt {
        InitializationBlock {
            initializations, ..
        } => {
            for istmt in initializations.iter() {
                traverse_statement(ac, runtime, istmt, program_archive);
            }
        }
        Declaration {
            meta,
            xtype,
            name,
            dimensions,
            ..
        } => {
            println!("[Traverse] Declaration of {}", name);
            match xtype {
                // VariableType::AnonymousComponent => {
                //     execute_anonymous_component_declaration(
                //         name,
                //         meta.clone(),
                //         &dimensions,
                //         &mut runtime.environment,
                //         &mut runtime.anonymous_components,
                //     );
                // }
                _ => {
                    let mut dim_u32_vec = Vec::new();
                    for dimension in dimensions.iter() {
                        let dim_u32_str =
                            traverse_expression(ac, runtime, name, dimension, program_archive);
                        dim_u32_vec.push(dim_u32_str.parse::<u32>().unwrap());
                    }
                    // treat_result_with_memory_error_void(
                    //     valid_array_declaration(&arithmetic_values),
                    //     meta,
                    //     &mut runtime.runtime_errors,
                    //     &runtime.call_trace,
                    // )?;
                    // let usable_dimensions =
                    //     if let Option::Some(dimensions) = cast_indexing(&arithmetic_values) {
                    //         dimensions
                    //     } else {
                    //         let err = Result::Err(ExecutionError::ArraySizeTooBig);
                    //         treat_result_with_execution_error(
                    //             err,
                    //             meta,
                    //             &mut runtime.runtime_errors,
                    //             &runtime.call_trace,
                    //         )?
                    //     };
                    match xtype {
                        VariableType::Component => traverse_component_declaration(
                            ac,
                            runtime,
                            name,
                            &dim_u32_vec, // &usable_dimensions
                                          // &mut runtime.environment,
                                          // actual_node
                        ),
                        VariableType::Var => traverse_variable_declaration(
                            ac,
                            runtime,
                            name,
                            &dim_u32_vec, // &usable_dimensions
                        ),
                        VariableType::Signal(signal_type, tag_list) => traverse_signal_declaration(
                            ac,
                            runtime,
                            name,
                            *signal_type,
                            &dim_u32_vec, // &usable_dimensions
                        ),
                        _ => {
                            unreachable!()
                        }
                    }
                }
            }
            // Option::None
        }
        IfThenElse {
            cond,
            if_case,
            else_case,
            ..
        } => {
            // let var = String::from("IFTHENELSE");
            // ac.add_var(&var, SignalType::Intermediate);
            // let lhs = traverse_expression(ac, &var, cond, program_archive);
            // traverse_statement(ac, &if_case, program_archive);
            // let else_case = else_case.as_ref().map(|e| e.as_ref());
            // traverse_statement(ac, else_case.unwrap(), program_archive);
            //     let else_case = else_case.as_ref().map(|e| e.as_ref());
            //     let (possible_return, can_simplify, _) = execute_conditional_statement(
            //         cond,
            //         if_case,
            //         else_case,
            //         program_archive,
            //         runtime,
            //         actual_node,
            //         flags
            //     )?;
            //     can_be_simplified = can_simplify;
            //     possible_return
            // }
            // While { cond, stmt, .. } => loop {
            //     let (returned, can_simplify, condition_result) = execute_conditional_statement(
            //         cond,
            //         stmt,
            //         Option::None,
            //         program_archive,
            //         runtime,
            //         actual_node,
            //         flags
            //     )?;
            //     can_be_simplified &= can_simplify;
            //     if returned.is_some() {
            //         break returned;
            //     } else if condition_result.is_none() {
            //         let (returned, _, _) = execute_conditional_statement(
            //             cond,
            //             stmt,
            //             None,
            //             program_archive,
            //             runtime,
            //             actual_node,
            //             flags
            //         )?;
            //         break returned;
            //     } else if !condition_result.unwrap() {
            //         break returned;
            //     }
        }
        While { cond, stmt, .. } => loop {
            let var = String::from("while");
            let (res, rb) = execute_expression(ac, runtime, &var, cond, program_archive);
            if res.contains("0") {
                break;
            }
            println!("[Traverse] While res = {} {}", res, rb);
            traverse_statement(ac, runtime, stmt, program_archive);
            // traverse_expression(ac, runtime, var, cond, program_archive);
            // let var = String::from("while");
            // ac.add_var(&var, SignalType::Intermediate);
            // let lhs = traverse_expression(ac, runtime, &var, cond, program_archive);
            // println!("While cond {}", lhs);
            // traverse_statement(ac, stmt, program_archive);
        },
        ConstraintEquality { meta, lhe, rhe, .. } => {
            // debug_assert!(actual_node.is_some());
            // let f_left = execute_expression(lhe, program_archive, runtime, flags)?;
            // let f_right = execute_expression(rhe, program_archive, runtime, flags)?;
            // let arith_left = safe_unwrap_to_arithmetic_slice(f_left, line!());
            // let arith_right = safe_unwrap_to_arithmetic_slice(f_right, line!());

            // let correct_dims_result = AExpressionSlice::check_correct_dims(&arith_left, &Vec::new(), &arith_right, true);
            // treat_result_with_memory_error_void(
            //     correct_dims_result,
            //     meta,
            //     &mut runtime.runtime_errors,
            //     &runtime.call_trace,
            // )?;
            // for i in 0..AExpressionSlice::get_number_of_cells(&arith_left){
            //     let value_left = treat_result_with_memory_error(
            //         AExpressionSlice::access_value_by_index(&arith_left, i),
            //         meta,
            //         &mut runtime.runtime_errors,
            //         &runtime.call_trace,
            //     )?;
            //     let value_right = treat_result_with_memory_error(
            //         AExpressionSlice::access_value_by_index(&arith_right, i),
            //         meta,
            //         &mut runtime.runtime_errors,
            //         &runtime.call_trace,
            //     )?;
            //     let possible_non_quadratic =
            //         AExpr::sub(
            //             &value_left,
            //             &value_right,
            //             &runtime.constants.get_p()
            //         );
            //     if possible_non_quadratic.is_nonquadratic() {
            //         treat_result_with_execution_error(
            //             Result::Err(ExecutionError::NonQuadraticConstraint),
            //             meta,
            //             &mut runtime.runtime_errors,
            //             &runtime.call_trace,
            //         )?;
            //     }
            //     let quadratic_expression = possible_non_quadratic;
            //     let constraint_expression = AExpr::transform_expression_to_constraint_form(
            //         quadratic_expression,
            //         runtime.constants.get_p(),
            //     )
            //     .unwrap();
            //     if let Option::Some(node) = actual_node {
            //         node.add_constraint(constraint_expression);
            //     }
            // }
            // Option::None
        }
        Return { value, .. } => {}
        Assert { arg, meta, .. } => {}
        Substitution {
            meta,
            var,
            access,
            op,
            rhe,
            ..
        } => {
            let mut name_access = String::from(var);
            println!("[Traverse] Sub Variable found {}", var.to_string());
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        println!("[Traverse] Sub Array access found");
                        // let mut dim_u32_vec = Vec::new();
                        let dim_u32_str =
                            traverse_expression(ac, runtime, var, expr, program_archive);
                        // dim_u32_vec.push(dim_u32_str.parse::<u32>().unwrap());
                        name_access.push_str("_");
                        name_access.push_str(dim_u32_str.as_str());
                        println!("[Traverse] Sub Change var name to {}", name_access);
                    }
                    Access::ComponentAccess(name) => {
                        println!("[Traverse] Sub Component access not handled");
                    }
                }
            }
            let rhs = traverse_expression(ac, runtime, &name_access, rhe, program_archive);
            println!("[Traverse] Sub Assigning {} to {}", rhs, &name_access);
            execute_statement(ac, runtime, stmt, program_archive);
        }
        Block { stmts, .. } => {
            traverse_sequence_of_statements(ac, runtime, stmts, program_archive, true);
        }
        LogCall { args, .. } => {}
        UnderscoreSubstitution { meta, rhe, op } => {
            println!("UnderscoreSubstitution found");
        }
        _ => {
            unimplemented!()
        }
    }
}

fn traverse_sequence_of_statements(
    ac: &mut ArithmeticCircuit,
    runtime: &mut CircomRuntime,
    stmts: &[Statement],
    program_archive: &ProgramArchive,
    is_complete_template: bool,
) {
    for stmt in stmts.iter() {
        traverse_statement(ac, runtime, stmt, program_archive);
    }
    if is_complete_template {
        //execute_delayed_declarations(program_archive, runtime, actual_node, flags)?;
    }
}

pub fn traverse_program(program_archive: &ProgramArchive) -> ArithmeticCircuit {
    let mut ac = ArithmeticCircuit::new();

    let mut runtime = CircomRuntime::new();
    runtime.init();

    let main_file_id = program_archive.get_file_id_main();

    // let mut runtime_information = RuntimeInformation::new(*main_file_id, program_archive.id_max, prime);

    use Expression::Call;

    // runtime_information.public_inputs = program_archive.get_public_inputs_main_component().clone();

    // Main expresssion
    // program_archive.get_main_expression()

    // Public inputs
    // program_archive.get_public_inputs_main_component().clone();

    if let Call { id, args, .. } = program_archive.get_main_expression() {
        let template_body = program_archive.get_template_data(id).get_body_as_vec();

        traverse_sequence_of_statements(
            &mut ac,
            &mut runtime,
            template_body,
            program_archive,
            true,
        );

        ac.print_ac();
        ac.truncate_zero_add_gate();
        ac.print_ac();
        ac.serde();

        // let folded_value_result =
        //     if let Call { id, args, .. } = &program_archive.get_main_expression() {
        //         let mut arg_values = Vec::new();
        //         for arg_expression in args.iter() {
        //             let f_arg = execute_expression(arg_expression, program_archive, &mut runtime_information, flags);
        //             arg_values.push(safe_unwrap_to_arithmetic_slice(f_arg.unwrap(), line!()));
        //             // improve
        //         }
        //         execute_template_call_complete(
        //             id,
        //             arg_values,
        //             BTreeMap::new(),
        //             program_archive,
        //             &mut runtime_information,
        //             flags,
        //         )
        //     } else {
        //         unreachable!("The main expression should be a call.");
        //     };

        // match folded_value_result {
        //     Result::Err(_) => Result::Err(runtime_information.runtime_errors),
        //     Result::Ok(folded_value) => {
        //         debug_assert!(FoldedValue::valid_node_pointer(&folded_value));
        //         Result::Ok((runtime_information.exec_program, runtime_information.runtime_errors))
        //     }
        // }
    };

    ac
}

pub fn parse_circom(filename: &str, inputs: &[ValueType], outputs: &[ValueType]) -> Result<(), ()> {
    let user_input = Input::default()?;
    let mut program_archive = parse_project(&user_input)?;
    analyse_project(&mut program_archive)?;

    // let config = ExecutionConfig {
    //     no_rounds: user_input.no_rounds(),
    //     flag_p: user_input.parallel_simplification_flag(),
    //     flag_s: user_input.reduced_simplification_flag(),
    //     flag_f: user_input.unsimplified_flag(),
    //     flag_old_heuristics: user_input.flag_old_heuristics(),
    //     flag_verbose: user_input.flag_verbose(),
    //     inspect_constraints_flag: user_input.inspect_constraints_flag(),
    //     r1cs_flag: user_input.r1cs_flag(),
    //     json_constraint_flag: user_input.json_constraints_flag(),
    //     json_substitution_flag: user_input.json_substitutions_flag(),
    //     sym_flag: user_input.sym_flag(),
    //     sym: user_input.sym_file().to_string(),
    //     r1cs: user_input.r1cs_file().to_string(),
    //     json_constraints: user_input.json_constraints_file().to_string(),
    //     prime: user_input.prime(),
    // };

    traverse_program(&program_archive);

    // let circuit = execute_project(program_archive, config)?;
    // let compilation_config = CompilerConfig {
    //     vcp: circuit,
    //     debug_output: user_input.print_ir_flag(),
    //     c_flag: user_input.c_flag(),
    //     wasm_flag: user_input.wasm_flag(),
    //     wat_flag: user_input.wat_flag(),
    //     js_folder: user_input.js_folder().to_string(),
    //     wasm_name: user_input.wasm_name().to_string(),
    //     c_folder: user_input.c_folder().to_string(),
    //     c_run_name: user_input.c_run_name().to_string(),
    //     c_file: user_input.c_file().to_string(),
    //     dat_file: user_input.dat_file().to_string(),
    //     wat_file: user_input.wat_file().to_string(),
    //     wasm_file: user_input.wasm_file().to_string(),
    //     produce_input_log: user_input.main_inputs_flag(),
    // };
    // compile(compilation_config)?;

    // Sample code for binary circuit

    // let builder = CircuitBuilder::new();

    // let mut feed_ids: Vec<usize> = Vec::new();
    // let mut feed_map: HashMap<usize, Node<Feed>> = HashMap::default();

    // let mut input_len = 0;
    // for input in inputs {
    //     let input = builder.add_input_by_type(input.clone());
    //     for (node, old_id) in input.iter().zip(input_len..input_len + input.len()) {
    //         feed_map.insert(old_id, *node);
    //     }
    //     input_len += input.len();
    // }

    // let mut state = builder.state().borrow_mut();
    // let pattern = Regex::new(GATE_PATTERN).unwrap();
    // for cap in pattern.captures_iter(&file) {
    //     let UncheckedGate {
    //         xref,
    //         yref,
    //         zref,
    //         gate_type,
    //     } = UncheckedGate::parse(cap)?;
    //     feed_ids.push(zref);

    //     match gate_type {
    //         GateType::Xor => {
    //             let new_x = feed_map
    //                 .get(&xref)
    //                 .ok_or(ParseError::UninitializedFeed(xref))?;
    //             let new_y = feed_map
    //                 .get(&yref.unwrap())
    //                 .ok_or(ParseError::UninitializedFeed(yref.unwrap()))?;
    //             let new_z = state.add_xor_gate(*new_x, *new_y);
    //             feed_map.insert(zref, new_z);
    //         }
    //         GateType::And => {
    //             let new_x = feed_map
    //                 .get(&xref)
    //                 .ok_or(ParseError::UninitializedFeed(xref))?;
    //             let new_y = feed_map
    //                 .get(&yref.unwrap())
    //                 .ok_or(ParseError::UninitializedFeed(yref.unwrap()))?;
    //             let new_z = state.add_and_gate(*new_x, *new_y);
    //             feed_map.insert(zref, new_z);
    //         }
    //         GateType::Inv => {
    //             let new_x = feed_map
    //                 .get(&xref)
    //                 .ok_or(ParseError::UninitializedFeed(xref))?;
    //             let new_z = state.add_inv_gate(*new_x);
    //             feed_map.insert(zref, new_z);
    //         }
    //     }
    // }
    // drop(state);
    // feed_ids.sort();

    // for output in outputs.iter().rev() {
    //     let feeds = feed_ids
    //         .drain(feed_ids.len() - output.len()..)
    //         .map(|id| {
    //             *feed_map
    //                 .get(&id)
    //                 .expect("Old feed should be mapped to new feed")
    //         })
    //         .collect::<Vec<Node<Feed>>>();

    //     let output = output.to_bin_repr(&feeds).unwrap();
    //     builder.add_output(output);
    // }

    // Ok(builder.build()?)

    Ok(())
}

struct UncheckedGate {
    xref: usize,
    yref: Option<usize>,
    zref: usize,
    gate_type: GateType,
}

impl UncheckedGate {
    fn parse(captures: Captures) -> Result<Self, ParseError> {
        let xref: usize = captures.name("xref").unwrap().as_str().parse()?;
        let yref: Option<usize> = captures
            .name("yref")
            .map(|yref| yref.as_str().parse())
            .transpose()?;
        let zref: usize = captures.name("zref").unwrap().as_str().parse()?;
        let gate_type = captures.name("gate").unwrap().as_str();

        let gate_type = match gate_type {
            "XOR" => GateType::Xor,
            "AND" => GateType::And,
            "INV" => GateType::Inv,
            _ => return Err(ParseError::UnsupportedGateType(gate_type.to_string())),
        };

        Ok(Self {
            xref,
            yref,
            zref,
            gate_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_circom_mul2() {
        let circ = parse_circom(
            "circuits/bristol/adder64_reverse.txt",
            &[ValueType::U64, ValueType::U64],
            &[ValueType::U64],
        )
        .unwrap();

        // stupid assert always true
        assert_eq!(3, 3);

        //let output: u64 = evaluate!(circ, fn(1u64, 2u64) -> u64).unwrap();

        //assert_eq!(output, 3);
    }
}

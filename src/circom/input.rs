use std::path::{Path, PathBuf};

pub struct Input {
    pub input_program: PathBuf,
    pub out_r1cs: PathBuf,
    pub out_json_constraints: PathBuf,
    pub out_json_substitutions: PathBuf,
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

impl Input {
    pub fn new(input_file: &str, output_dir: &str, override_file_name: Option<&str>) -> Input {
        let file_name = match override_file_name {
            Some(f) => f.to_string(),
            None => Path::new(input_file)
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string(),
        };

        Input {
            input_program: input_file.into(),
            out_r1cs: format!("{}/{}.r1cs", output_dir, file_name).into(),
            out_json_constraints: format!("{}/{}_constraints.json", output_dir, file_name).into(),
            out_json_substitutions: format!("{}/{}_substitutions.json", output_dir, file_name)
                .into(),
            out_wat_code: format!("{}/{}_js/{}.wat", output_dir, file_name, file_name).into(),
            out_wasm_code: format!("{}/{}_js/{}.wasm", output_dir, file_name, file_name).into(),
            out_wasm_name: file_name.clone(),
            out_js_folder: format!("{}/{}_js", output_dir, file_name).into(),
            out_c_run_name: file_name.clone(),
            out_c_folder: format!("{}/{}_cpp", output_dir, file_name).into(),
            out_c_code: format!("{}/{}_cpp/{}.cpp", output_dir, file_name, file_name).into(),
            out_c_dat: format!("{}/{}_cpp/{}.dat", output_dir, file_name, file_name).into(),
            out_sym: format!("{}/{}.sym", output_dir, file_name).into(),
            c_flag: false,
            wasm_flag: false,
            wat_flag: false,
            r1cs_flag: false,
            sym_flag: false,
            json_constraint_flag: false,
            json_substitution_flag: false,
            main_inputs_flag: false,
            print_ir_flag: false,
            fast_flag: false,
            reduced_simplification_flag: false,
            parallel_simplification_flag: false,
            flag_old_heuristics: false,
            inspect_constraints_flag: false,
            no_rounds: 18446744073709551615,
            flag_verbose: false,
            prime: "bn128".into(),
            link_libraries: vec![],
        }
    }

    pub fn build_output(output_path: &Path, filename: &str, ext: &str) -> PathBuf {
        let mut file = output_path.to_path_buf();
        file.push(format!("{}.{}", filename, ext));
        file
    }

    pub fn get_link_libraries(&self) -> &Vec<PathBuf> {
        &self.link_libraries
    }

    pub fn input_file(&self) -> &str {
        self.input_program.to_str().unwrap()
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
    pub fn json_substitutions_file(&self) -> &str {
        self.out_json_substitutions.to_str().unwrap()
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
pub mod input_processing {
    use std::path::{Path, PathBuf};

    use crate::circom::compilation::VERSION;
    use clap::{App, Arg, ArgMatches};

    use super::Input;

    pub fn generate_input(input_file: PathBuf, output_dir: PathBuf) -> Result<Input, ()> {
        let matches = view();
        let mut file_name = input_file
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let c_flag = get_c(&matches);

        if c_flag && (file_name == "main" || file_name == "fr" || file_name == "calcwit") {
            println!("The name {} is reserved in Circom when using de --c flag. The files generated for your circuit will use the name {}_c instead of {}.", file_name, file_name, file_name);
            file_name = format!("{}_c", file_name)
        };

        let mut input = Input::new(
            input_file.to_str().unwrap(),
            output_dir.to_str().unwrap(),
            Some(&file_name),
        );

        let o_style = get_simplification_style(&matches)?;

        input.wat_flag = get_wat(&matches);
        input.wasm_flag = get_wasm(&matches);
        input.c_flag = c_flag;
        input.r1cs_flag = get_r1cs(&matches);
        input.sym_flag = get_sym(&matches);
        input.main_inputs_flag = get_main_inputs_log(&matches);
        input.json_constraint_flag = get_json_constraints(&matches);
        input.json_substitution_flag = get_json_substitutions(&matches);
        input.print_ir_flag = get_ir(&matches);
        input.no_rounds = if let SimplificationStyle::O2(r) = o_style {
            r
        } else {
            0
        };
        input.fast_flag = o_style == SimplificationStyle::O0;
        input.reduced_simplification_flag = o_style == SimplificationStyle::O1;
        input.parallel_simplification_flag = get_parallel_simplification(&matches);
        input.inspect_constraints_flag = get_inspect_constraints(&matches);
        input.flag_old_heuristics = get_flag_old_heuristics(&matches);
        input.flag_verbose = get_flag_verbose(&matches);
        input.prime = get_prime(&matches)?;
        input.link_libraries = get_link_libraries(&matches);

        Ok(input)
    }

    pub fn get_input(matches: &ArgMatches) -> Result<PathBuf, ()> {
        let route = Path::new(matches.value_of("input").unwrap()).to_path_buf();
        if route.is_file() {
            Result::Ok(route)
        } else {
            let route = if route.to_str().is_some() {
                ": ".to_owned() + route.to_str().unwrap()
            } else {
                "".to_owned()
            };
            Result::Err(eprintln!("Input file does not exist {}", route))
        }
    }

    pub fn get_output_path(matches: &ArgMatches) -> Result<PathBuf, ()> {
        let route = Path::new(matches.value_of("output").unwrap()).to_path_buf();
        if route.is_dir() {
            Result::Ok(route)
        } else {
            Result::Err(eprintln!("invalid output path"))
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
                let rounds_r = o_2_argument.parse::<usize>();
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
                    || prime_value == "secq256r1"
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
                    .help("Only applies signal to signal and signal to constant simplification")
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
                    .default_value("./output/")
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
                    .long("simplification_substitution")
                    .takes_value(false)
                    .display_order(980)
                    .help("Outputs the substitution applied in the simplification phase in json format"),
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
                    .help("To choose the prime number to use to generate the circuit. Receives the name of the curve (bn128, bls12381, goldilocks, grumpkin, pallas, vesta, secq256r1)"),
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
}

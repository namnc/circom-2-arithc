use circom_compiler::compiler_interface::{self, Config, VCP};
use circom_virtual_fs::{FileSystem, FsResult};

pub const VERSION: &str = "2.0.0";

pub struct CompilerConfig {
    pub cwd: String,
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

pub fn compile(fs: &mut dyn FileSystem, config: CompilerConfig) -> FsResult<()> {
    compiler_interface::run_compiler(
        fs,
        config.vcp,
        Config {
            debug_output: config.debug_output,
            produce_input_log: config.produce_input_log,
            wat_flag: config.wat_flag,
        },
        VERSION,
    )?;

    Ok(())
}

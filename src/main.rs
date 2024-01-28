use std::env::current_dir;

use circom_2_arithc::{compiler::Input, program::{parse_circom, ProgramError}};
use dotenv::dotenv;
use env_logger::{init_from_env, Env};

fn main() -> Result<(), ()>{
    dotenv().ok();
    init_from_env(Env::default().filter_or("LOG_LEVEL", "info"));
    let input = Input::new()?;
    match  parse_circom(&input) {
        Err(_) => Err(()),
        Ok(_) => Ok(())  
    }
}

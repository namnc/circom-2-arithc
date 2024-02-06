use std::fs::File;

use circom_2_arithc::{compiler::Input, program::parse_circom};
use dotenv::dotenv;
use env_logger::{init_from_env, Env};
use std::io::Write;

fn main() -> Result<(), ()> {
    dotenv().ok();
    init_from_env(Env::default().filter_or("LOG_LEVEL", "info"));
    let input = Input::new()?;
    match parse_circom(&input) {
        Err(err) => {
            println!("Error {}", err);
            Ok(())
        },
        Ok(cir) => {
            let output_name = input.out_mpc;
            let mut data_file = File::create(output_name).expect("Creation file failed!");
            data_file
                .write_all(serde_json::to_string(&cir).unwrap().as_bytes())
                .expect("Write file failed!");
            Ok(())
        }
    }
}

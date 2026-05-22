use dotenvy::dotenv;
use evm_monitor::cli::address;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let args: Vec<String> = env::args().skip(1).collect();
    address::run(&args)
}

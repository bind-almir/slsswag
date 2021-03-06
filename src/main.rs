use std::env;
use std::process;
use slsswag::Params;

fn main() {

    let args: Vec<String> = env::args().collect();

    let params = Params::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = slsswag::run(params) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}
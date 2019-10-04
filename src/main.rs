use std::env::args;
use rox::{
    Lox,
};

fn main() {
    let mut args = args();
    let _ = args.next();
    let args: Vec<String> = args.collect();
    let mut lox = Lox::new();
    match args.len() {
        0 => lox.run_prompt().expect("failed to run prompt"),
        1 => lox.run_file(&args[0]).expect("Failed to run file"),
        _ => {
            eprintln!("Usage roxc [script]");
            ::std::process::exit(64);
        }
    }
}
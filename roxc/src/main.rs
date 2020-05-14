use roxc::{Chunk, OpCode, Value, VM, Error};
use std::path::PathBuf;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    if let Some(arg) = args.next() {
        let path = PathBuf::from(arg);
        if path.exists() {
            run_file(path)
        } else {
            println!("Usage: clox [path]");
            std::process::exit(64);
        }
    } else {
        repl()
    }
}

fn repl() -> Result<()> {
    use std::io::{Read, Write};
    let mut buf = vec![0; 1024];
    let mut input = std::io::stdin();
    let mut output = std::io::stdout();
    loop {
        output.write_all(b"> ")?;
        output.flush()?;
        let line = match input.read(&mut buf) {
            Ok(len) => {
                &buf[0..len]
            },
            Err(e) => {
                eprintln!("error: {:?}", e);
                break;
            }
        };
        interpret(line)?;
    }
    Ok(())
}

fn run_file(path: PathBuf) -> Result<()> {
    let bytes = std::fs::read(path)?;
    match interpret(&bytes) {
        Ok(_) => (),
        Err(Error::Compiler(msg)) => {
            eprintln!("{}", msg);
            std::process::exit(65);
        },
        Err(Error::Runtime(msg)) => {
            eprintln!("{}", msg);
            std::process::exit(70)
        },
    }

    Ok(())
}

fn interpret(bytes: &[u8]) -> std::result::Result<(), Error> {
    println!("{:?}", bytes);
    Ok(())
}

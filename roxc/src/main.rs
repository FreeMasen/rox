use roxc::{Error,VM};
use std::path::PathBuf;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn main() -> Result<()> {
    use tracing_subscriber::{EnvFilter, FmtSubscriber};
    
    tracing_log::LogTracer::init().unwrap();
    let fmtsubscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(fmtsubscriber)
        .expect("setting default subscriber failed");
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
#[tracing::instrument]
fn repl() -> Result<()> {
    // use std::io::Write;
    // let input = std::io::stdin();
    // let mut output = std::io::stdout();
    // loop {
    //     let mut buf = String::new();
    //     output.write_all(b"> ")?;
    //     output.flush()?;
    //     let line = match input.read_line(&mut buf) {
    //         Ok(len) => &buf[0..len],
    //         Err(e) => {
    //             eprintln!("error: {:?}", e);
    //             break;
    //         }
    //     };
    //     interpret(line)?;
    // }
    interpret(r#""yes" + "no""#).unwrap();
    Ok(())
}
#[tracing::instrument]
fn run_file(path: PathBuf) -> Result<()> {
    let bytes = std::fs::read_to_string(path)?;
    match interpret(&bytes) {
        Ok(_) => (),
        Err(Error::Compiler(msg)) => {
            eprintln!("{}", msg);
            std::process::exit(65);
        }
        Err(Error::Runtime(msg)) => {
            eprintln!("{}", msg);
            std::process::exit(70)
        }
    }

    Ok(())
}
#[tracing::instrument(skip(source))]
fn interpret(source: &str) -> std::result::Result<(), Error> {
    let mut vm = VM::default();

    vm.interpret(source)
}

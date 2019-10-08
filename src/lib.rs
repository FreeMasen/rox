use std::{fs::read_to_string, io::stdin, path::Path};

mod error;
mod expr;
mod interpreter;
mod parser;
mod scanner;
mod statement;
pub mod token;

pub use error::Error;
use interpreter::Interpreter;
pub use scanner::Scanner;

type SimpleResult<T> = Result<T, Error>;

pub struct Lox {
    had_error: bool,
}

impl Lox {
    pub fn new() -> Self {
        Self { had_error: false }
    }
    pub fn run_file<T>(&mut self, path: T) -> SimpleResult<()>
    where
        T: AsRef<Path>,
    {
        let lox = read_to_string(path).map_err(|e| Error::Runtime(format!("IO Error: {}", e)))?;
        let mut int = Interpreter::new();
        self.run(lox, &mut int)?;
        if self.had_error {
            ::std::process::exit(65);
        }
        Ok(())
    }
    pub fn run_prompt(&mut self) -> SimpleResult<()> {
        let reader = stdin();
        let mut int = Interpreter::new();
        loop {
            let mut line = String::new();
            write_prompt();
            reader
                .read_line(&mut line)
                .map_err(|e| Error::Runtime(format!("IO Error: {}", e)))?;
            let _ = self.run(line, &mut int);
            self.had_error = false;
        }
    }
    fn run(&mut self, s: String, int: &mut Interpreter) -> SimpleResult<()> {
        let scanner = Scanner::new(s)?;

        let mut parser = parser::Parser::new(scanner);

        while let Some(stmt) = parser.next() {
            match &stmt {
                Ok(stmt) => {
                    int.interpret(&stmt)?;
                }
                Err(e) => {
                    self.error(parser.line(), e.clone());
                    parser.sync();
                }
            }
        }
        Ok(())
    }

    fn error(&mut self, line: usize, e: Error) {
        self.report(line, "", &format!("{}", e));
    }
    fn report(&mut self, line: usize, file: &str, msg: &str) {
        println!("[line {}] Error {}: {}", line, file, msg);
        self.had_error = true;
    }
}

fn write_prompt() {
    use std::io::{stdout, Write};
    let mut out = stdout();
    let _ = out.write(b"> ");
    let _ = out.flush();
}

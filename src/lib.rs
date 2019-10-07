use std::{
    path::Path,
    fs::read_to_string,
    io::{
        stdin,
    },

};

mod scanner;
mod error;
pub mod token;
mod expr;
mod parser;
mod interpreter;

pub use scanner::Scanner;
pub use error::Error;

type SimpleResult<T> = Result<T, Error>;

pub struct Lox {
    had_error: bool
}


impl Lox {
    pub fn new() -> Self {
        Self {
            had_error: false,
        }
    }
    pub fn run_file<T>(&mut self, path: T) -> SimpleResult<()>
    where T: AsRef<Path> {
        self.run(read_to_string(path).map_err(|e| Error::Runtime(format!("IO Error: {}", e)))?)?;
        if self.had_error {
            ::std::process::exit(65);
        }
        Ok(())
    }
    pub fn run_prompt(&mut self) -> SimpleResult<()> {
        let reader = stdin();
        loop {
            let mut line = String::new();
            print!("> ");
            reader.read_line(&mut line).map_err(|e| Error::Runtime(format!("IO Error: {}", e)))?;
            let _ = self.run(line);
            self.had_error = false;
        }
        Ok(())
    }
    fn run(&mut self, s: String) -> SimpleResult<()> {
        let scanner = Scanner::new(s)?;
       
        let mut parser = parser::Parser::new(scanner);
        let int = interpreter::Interpreter;
        while !parser.is_at_end() {
            match parser.expression() {
                Ok(expr) => match int.evaluate(&expr) {
                    Ok(val) => println!("{}", val),
                    Err(e) => {
                        self.error(parser.line(), e);
                    },
                },
                Err(e) => {
                    self.error(parser.line(), e);
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






use std::{fs::read_to_string, io::stdin, path::Path};
use log::{
    trace,
    error,
};
mod error;
mod expr;
mod interpreter;
mod parser;
mod scanner;
mod stmt;
pub mod token;
mod env;
mod callable;
mod globals;
mod func;
mod value;
mod class;

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
        trace!("Running a file");
        let lox = read_to_string(path).map_err(|e| Error::Runtime(format!("IO Error: {}", e)))?;
        let mut int = Interpreter::new();
        self.run(lox, &mut int)?;
        if self.had_error {
            ::std::process::exit(65);
        }
        Ok(())
    }
    pub fn run_prompt(&mut self) -> SimpleResult<()> {
        trace!("Running a prompt");
        let reader = stdin();
        let mut int = Interpreter::new();
        let mut indent = 0;
        loop {
            let mut line = String::new();
            write_prompt(indent);
            loop {
                reader
                    .read_line(&mut line)
                    .map_err(|e| Error::Runtime(format!("IO Error: {}", e)))?;
                if line.ends_with("\r\n") {
                    line.pop();
                    line.pop();
                    line.push('\n');
                }
                if indent == 0 && line.ends_with(";\n") {
                    break;
                }
                if line.ends_with("{\n") {
                    indent += 1;
                }
                if line.ends_with("}\n") {
                    indent = indent.saturating_sub(1);
                }
                if indent == 0 {
                    break;
                }
                write_prompt(indent);
            }
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
                    error!("Error: {}", e);
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

fn write_prompt(indent: usize) {
    use std::io::{stdout, Write};
    let mut out = stdout();
    if indent > 0 {
        let _ = out.write("  ".repeat(indent).as_bytes());
    } else {
        let _ = out.write(b"> ");
    }
    let _ = out.flush();
}

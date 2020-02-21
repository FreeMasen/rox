use log::{error, trace};
use std::{fs::read_to_string, io::stdin, path::Path};
mod callable;
mod class;
mod env;
mod error;
mod expr;
mod func;
mod globals;
mod interpreter;
mod parser;
mod resolver;
mod scanner;
mod stmt;
pub mod token;
mod value;

pub use error::Error;
use interpreter::Interpreter;
pub use scanner::Scanner;

type SimpleResult<T> = Result<T, Error>;
pub struct Lox {
    had_error: bool,
}
impl Default for Lox {
    fn default() -> Self {
        Self { had_error: false } 
    }
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
        let mut lox =
            read_to_string(path).map_err(|e| Error::Runtime(format!("IO Error: {}", e)))?;
        if !lox.ends_with('\n') {
            lox.push('\n');
        }
        let mut int = Interpreter::new();
        let mut resv = resolver::Resolver::new();
        self.run(lox, &mut int, &mut resv)?;
        if self.had_error {
            ::std::process::exit(65);
        }
        Ok(())
    }
    pub fn run_prompt(&mut self) -> SimpleResult<()> {
        trace!("Running a prompt");
        let reader = stdin();
        let mut int = Interpreter::default();
        let mut resv = resolver::Resolver::new();
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
            let _ = self.run(line, &mut int, &mut resv);
            self.had_error = false;
        }
    }
    fn run(&mut self, s: String, int: &mut Interpreter, resv: &mut resolver::Resolver) -> SimpleResult<()> {
        let scanner = Scanner::new(s)?;

        let parser = parser::Parser::new(scanner);
        let stmts: Vec<stmt::Stmt> = parser.collect::<Result<Vec<stmt::Stmt>, Error>>()?;
        for stmt in &stmts {
            resv.resolve_stmt(stmt)?;
        }
        int.locals = dbg!(resv.depths.clone());
        for stmt in stmts {
            int.interpret(&stmt)?;
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

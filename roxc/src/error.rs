#[derive(Debug)]
pub enum Error {
    Runtime(String),
    Compiler(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        use Error::*;
        match self {
            Runtime(msg) => write!(f, "Runtime error: {}", msg),
            Compiler(msg) => write!(f, "Compiler error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

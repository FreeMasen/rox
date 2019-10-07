#[derive(Clone, Debug)]
pub enum Error {
    Scanner(String),
    Parser(String),
    Runtime(String),
}
impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Error::Scanner(s) => format!("Scanning error: {}", s).fmt(f),
            Error::Parser(s) => format!("Parser error: {}", s).fmt(f),
            Error::Runtime(s) => format!("Runtime error: {}", s).fmt(f),
        }
    }
}
impl ::std::error::Error for Error {}
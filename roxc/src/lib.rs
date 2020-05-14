mod chunk;
mod error;
mod op;
mod value;
mod vm;
mod scanner;

pub use chunk::Chunk;
pub use error::Error;
pub use op::OpCode;
pub use value::Value;
pub use vm::VM;

pub type Result<T> = core::result::Result<T, error::Error>;

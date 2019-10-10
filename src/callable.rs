use crate::{
    interpreter::Interpreter,
    error::Error,
    value::Value,
};
use std::fmt::Debug;

pub trait Callable 
where Self: Debug {
    fn name(&self) -> &str {
        "unknown"
    }
    fn arity(&self) -> usize {
        0
    }
    fn call(&self, int: &mut Interpreter, args: &[Value]) -> Result<Value, Error>;
}
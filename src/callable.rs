use crate::{
    interpreter::Interpreter,
    error::Error,
    value::Value,
};
use std::fmt::{Debug, Display};

pub trait Callable 
where Self: Debug + Display {
    fn name(&self) -> &str {
        "unknown"
    }
    fn arity(&self) -> usize {
        0
    }
    fn call(&self, int: &mut Interpreter, args: &[Value]) -> Result<Value, Error>;
}
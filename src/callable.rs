use crate::{
    interpreter::{Interpreter, Value},
    error::Error,
};
use std::fmt::Debug;

pub trait Callable 
where Self: Debug {
    fn arity(&self) -> usize {
        0
    }
    fn call(&self, int: &mut Interpreter, args: &[Value]) -> Result<Value, Error>;
}
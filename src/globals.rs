use crate::{
    callable::Callable,
    error::Error,
    interpreter::{Interpreter, Value},
};
use chrono::prelude::*;
#[derive(Debug, Clone)]
pub struct Clock;

impl Callable for Clock {
    fn call(&self, _: &mut Interpreter, _: &[Value]) -> Result<Value, Error> {
        let now = Local::now();
        let unix: DateTime<Local> = DateTime::from(::std::time::UNIX_EPOCH);
        let dur = now.signed_duration_since(unix);
        let mil = dur.num_milliseconds().abs() as u64;
        Ok(Value::Number(mil as f64))
    }
}
#[derive(Debug, Clone)]
pub struct Mod;

impl Callable for Mod {
    fn call(&self, _: &mut Interpreter, args: &[Value]) -> Result<Value, Error> {
        if let Some(Value::Number(lhs)) = args.get(0) {
            if let Some(Value::Number(rhs)) = args.get(1) {
                return Ok(Value::Number(lhs % rhs));
            }
        }
        Err(Error::Runtime(format!("invalid arguments provided to mod: {:?}", args)))
    }
}


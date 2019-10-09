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


use crate::{callable::Callable, error::Error, interpreter::Interpreter, value::Value};

#[derive(Debug, Clone)]
pub struct Clock;
impl NativeFunc for Clock {}
pub trait NativeFunc
where
    Self: Callable,
{
}

impl Callable for Clock {
    fn name(&self) -> &str {
        "clock"
    }
    fn call(&self, _: &mut Interpreter, _: &[Value]) -> Result<Value, Error> {
        let now = ::std::time::SystemTime::now();
        let dur = now.duration_since(::std::time::UNIX_EPOCH)
            .map_err(|e| Error::Runtime(format!("Error calculating current timestamp:\n{}", e)))?;
        Ok(Value::Number(dur.as_secs_f64() * 1000.0))
    }
}
#[derive(Debug, Clone)]
pub struct Mod;
impl Callable for Mod {
    fn name(&self) -> &str {
        "[native func mod]"
    }
    fn arity(&self) -> usize {
        2
    }
    fn call(&self, _: &mut Interpreter, args: &[Value]) -> Result<Value, Error> {
        if let Some(Value::Number(lhs)) = args.get(0) {
            if let Some(Value::Number(rhs)) = args.get(1) {
                return Ok(Value::Number(lhs % rhs));
            }
        }
        Err(Error::Runtime(format!(
            "invalid arguments provided to mod: {:?}",
            args
        )))
    }
}

impl ::std::fmt::Display for Clock {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "[native fn clock]")
    }
}
impl ::std::fmt::Display for Mod {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "[native fn mod]")
    }
}

use crate::{callable::Callable, error::Error, interpreter::Interpreter, value::Value};

#[derive(Debug, Clone)]
pub enum Global {
    Clock(Clock),
    Mod(Mod),
}

impl ::std::fmt::Display for Global {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Global::Clock(c) => c.fmt(f),
            Global::Mod(m) => m.fmt(f),
        }
    }
}

impl Callable for Global {
    fn name(&self) -> &str {
        match self {
            Global::Clock(c) => c.name(),
            Global::Mod(m) => m.name(),
        }
    }
    fn arity(&self) -> usize {
        match self {
            Global::Clock(c) => c.arity(),
            Global::Mod(m) => m.arity(),
        }
    }
    fn call(&self, int: &mut Interpreter, args: &[Value]) -> Result<Value, Error> {
        match self {
            Global::Clock(c) => c.call(int, args),
            Global::Mod(m) => m.call(int, args),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Clock;

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

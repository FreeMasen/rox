use crate::{
    callable::Callable, env::Env, error::Error, interpreter::Interpreter, stmt::Stmt, value::Value,
};

#[derive(Debug, Clone)]
pub struct Func {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub env: Env,
    pub env_idx: usize,
}

impl Callable for Func {
    fn name(&self) -> &str {
        &self.name
    }
    fn arity(&self) -> usize {
        self.params.len()
    }
    fn call(&mut self, int: &mut Interpreter, args: &[Value]) -> Result<Value, Error> {
        let tail_env = int.env.split_to_base();
        int.env.append(self.env.clone());

        for (name, value) in self.params.iter().zip(args.iter().cloned()) {
            int.env.define(name, Some(value));
        }
        let ret = match int.execute_block(&mut self.body) {
            Ok(_) => Ok(Value::Nil),
            Err(Error::Return(v)) => Ok(v),
            Err(e) => Err(e),
        };

        self.env = int.env.clone_to_base();
        int.env.append(tail_env);
        int.env.assign(self.name(), Value::Func(self.clone()))?;
        ret
    }
}

impl ::std::fmt::Display for Func {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "[fn {}]", self.name)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn nested_funcs() {
        let _ = pretty_env_logger::try_init();
        let lox = "
var test = 0;
fun makeCounter() {
  var i = 0;
  fun count() {
    i = i + 1;
    return i;
  }

  return count;
}

var counter = makeCounter();
var test1 = counter();
var test2 = counter();
";
        let mut int = Interpreter::new();
        let mut p =
            crate::parser::Parser::new(crate::scanner::Scanner::new(lox.to_string()).unwrap());
        int.interpret(&mut p.next().unwrap().unwrap()).unwrap();
        int.interpret(&mut p.next().unwrap().unwrap()).unwrap();
        int.interpret(&mut p.next().unwrap().unwrap()).unwrap();

        int.interpret(&mut p.next().unwrap().unwrap()).unwrap();
        let test = int.env.get("test1").expect("Failed to get test1 from env");
        assert_eq!(test, Value::Number(1f64), "test1 was not 1");
        int.interpret(&mut p.next().unwrap().unwrap()).unwrap();
        let test2 = int.env.get("test2").expect("Failed to get test2 from env");
        assert_eq!(test2, Value::Number(2f64), "test2 was not 2");
    }

    #[test]
    fn recursive() {
        let _ = pretty_env_logger::try_init();
        let lox = "
        fun fib(n) {
            if (n < 2) return n;
            return fib(n - 1) + fib(n - 2);
        }
        var test = fib(4);";
        let mut int = Interpreter::new();
        let mut p =
            crate::parser::Parser::new(crate::scanner::Scanner::new(lox.to_string()).unwrap());
        let mut fib = p.next().unwrap().expect("failed to define fib");
        int.interpret(&mut fib).expect("failed to define fib def");
        let mut test = p.next().unwrap().expect("failed to parse test assignment");
        int.interpret(&mut test)
            .expect("failed to evalue test assignment");
        let test = int.env.get("test").expect("Failed to get test from env");
        assert_eq!(test, Value::Number(3f64));
    }
}

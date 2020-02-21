use crate::{callable::Callable, error::Error, interpreter::Interpreter, stmt::Stmt, value::Value};
use log::trace;

#[derive(Debug, Clone)]
pub struct Func {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub env_id: usize,
}

impl Callable for Func {
    fn name(&self) -> &str {
        &self.name
    }
    fn arity(&self) -> usize {
        self.params.len()
    }
    fn call(&self, int: &mut Interpreter, args: &[Value]) -> Result<Value, Error> {
        trace!("calling {}", self.name);
        int.env.descend(); // create scope for function args
        for (name, value) in self.params.iter().cloned().zip(args.iter().cloned()) {
            int.env.define(name, Some(value));
        }
        let ret = match int.execute_block(&self.body) {
            Ok(_) => Ok(Value::Nil),
            Err(Error::Return(v)) => Ok(v),
            Err(e) => Err(e),
        };
        int.env.ascend();
        int.current_depth = int.env.depth();
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
test = counter();
test = counter();
";
        let mut int = Interpreter::new();
        let p =
            crate::parser::Parser::new(crate::scanner::Scanner::new(lox.to_string()).unwrap());
        let stmts = p.collect::<Result<Vec<crate::stmt::Stmt>, Error>>().unwrap();
        let mut r = crate::resolver::Resolver::new();
        for stmt in &stmts {
            r.resolve_stmt(stmt).unwrap();
        }
        int.locals = dbg!(r.depths);
        let mut p = stmts.iter();
        int.interpret(&p.next().unwrap()).unwrap(); // define test
        int.interpret(&p.next().unwrap()).unwrap(); // define makeCounter
        int.interpret(&p.next().unwrap()).unwrap(); // call make counter

        int.interpret(&p.next().unwrap()).unwrap();
        let test = int.env.get("test", 1).expect("Failed to get test from env");
        assert_eq!(test, Value::Number(1f64));
        int.interpret(&p.next().unwrap()).unwrap();
        let test2 = int.env.get("test", 1).expect("Failed to get test from env");
        assert_eq!(test2, Value::Number(2f64));
        println!("test: {:?}", test);
    }
}

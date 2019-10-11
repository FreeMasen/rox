use crate::{
    callable::Callable,
    error::Error,
    interpreter::Interpreter,
    stmt::Stmt,
    value::Value,
};
use log::trace;

#[derive(Debug, Clone)]
pub struct Func {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
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
        if let Some(closure) = int.closures.get_mut(&self.name) {
            trace!("found closure for {}", &self.name);
            int.env.descend_into(closure.clone());
        } else {
            // this shouldn't be possible but there is no
            // harm in it happening
            int.env.descend();
        }
        int.env.descend(); // create scope for function args
        for (name, value) in self.params.iter().cloned().zip(args.iter().cloned()) {
            int.env.define(name, Some(value));
        }
        let ret = match int.execute_block(&self.body) {
            Ok(_) => Ok(Value::Nil),
            Err(Error::Return(v)) => Ok(v),
            Err(e) => Err(e),
        };
        int.env.ascend(); // ascend out of function arg defs
        int.closures.insert(self.name.to_string(), int.env.ascend_out_of()?);
        ret
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
        let mut p = crate::parser::Parser::new(crate::scanner::Scanner::new(lox.to_string()).unwrap());
        int.interpret(&p.next().unwrap().unwrap()).unwrap();
        int.interpret(&p.next().unwrap().unwrap()).unwrap();
        int.interpret(&p.next().unwrap().unwrap()).unwrap();
        
        int.interpret(&p.next().unwrap().unwrap()).unwrap();
        let test = int.env.get("test").expect("Failed to get test from env");
        assert_eq!(test, Value::Number(1f64));
        int.interpret(&p.next().unwrap().unwrap()).unwrap();
        let test2 = int.env.get("test").expect("Failed to get test from env");
        assert_eq!(test2, Value::Number(2f64));
        println!("test: {:?}", test);
    }
}
use crate::{
    callable::Callable,
    error::Error,
    interpreter::{Value, Interpreter},
    stmt::Stmt,
};
#[derive(Debug, Clone)]
pub struct Func {
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}

impl Callable for Func {
    fn call(&self, int: &mut Interpreter, args: &[Value]) -> Result<Value, Error> {
        int.env.descend();
        for (name, value) in self.params.iter().cloned().zip(args.iter().cloned()) {
            int.env.define(name, Some(value));
        }
        int.execute_block(&self.body)?;
        int.env.ascend();
        Ok(Value::Nil)
    }
}
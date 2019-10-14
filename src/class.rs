use crate::{
    callable::Callable,
    error::Error,
    interpreter::Interpreter,
    stmt::Function,
    value::Value,
};
#[derive(Clone, Debug)]
pub struct Class {
    pub name: String,
    pub methods: Vec<Function>,
}
#[derive(Clone, Debug)]
pub struct ClassInstance {
    pub class: Class,
}

impl Callable for Class {
    fn name(&self) -> &str {
        &self.name
    }
    fn arity(&self) -> usize {
        0
    }
    fn call(&self, int: &mut Interpreter, args: &[Value]) -> Result<Value, Error> { 
        unimplemented!()
    }
}

impl ::std::fmt::Display for Class {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "[ctor {}]", self.name)
    }
}
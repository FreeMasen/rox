use crate::{
    callable::Callable, env::Env, error::Error, interpreter::Interpreter, stmt::Function, value::Value, func::Func
};
use std::collections::HashMap;
#[derive(Clone, Debug)]
pub struct Class {
    pub name: String,
    pub methods: Vec<Function>,
}
#[derive(Clone, Debug,)]
pub struct ClassInstance {
    pub class: Class,
    pub fields: HashMap<String, Value>,
    pub methods: HashMap<String, Func>,
    pub env: Env
}

#[derive(Clone, Debug)]
pub struct Method {
    pub func: Func,
    pub this: HashMap<String, Value>,
}
impl Callable for Class {
    fn name(&self) -> &str {
        &self.name
    }
    fn arity(&self) -> usize {
        0
    }
    fn call(&self, int: &mut Interpreter, args: &[Value]) -> Result<Value, Error> {
        let mut methods = HashMap::new();
        for func in &self.methods {
            methods.insert(func.name.to_string(), Func {
                name: func.name.to_string(),
                env_id: int.current_depth + 1,
                params: func.params.clone(),
                body: func.body.clone(),
            });
        }
        let ret = ClassInstance {
            fields: HashMap::new(),
            class: self.clone(),
            methods: methods,
            env: Env::new(int.current_depth),
        };
        
        let ret = Value::Class(ret);
        Ok(ret)
    }
}

impl ::std::fmt::Display for Class {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "[ctor {}]", self.name)
    }
}

impl ClassInstance {
    pub fn get(&self, key: &str) -> Result<Value, Error> {
        if let Some(val) = self.fields.get(key) {
            Ok(val.clone())
        } else if let Some(val) = self.methods.get(key) {
            Ok(Value::Func(val.clone()))
        } else {
            Err(Error::Runtime(format!(
                "Undefined propety on {} instance: {}",
                self.class.name(),
                key
            )))
        }
    }
    
    pub fn get_mut(&mut self, key: &str) -> Result<&mut Value, Error> {
        if let Some(val) = self.fields.get_mut(key) {
            Ok(val)
        } else {
            Err(Error::Runtime(format!(
                "Undefined propety on {} instance: {}",
                self.class.name(),
                key
            )))
        }
    }
    pub fn set(&mut self, key: &str, value: Value) {
        if let Some(val) = self.fields.get_mut(key) {
            *val = value;
        } else {
            self.fields.insert(key.to_string(), value);
        }
    }
}

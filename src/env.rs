use crate::{
    error::Error,
    globals::*,
    interpreter::Value,
};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Env {
    values: HashMap<String, Value>,
    enclosing: Option<Box<Env>>,
}

impl Env {
    fn global() -> Self {
        let mut values = HashMap::new();
        values.insert(String::from("clock"), Value::Func(Rc::new(Clock)));
        values.insert(String::from("mod"), Value::Func(Rc::new(Mod)));
        Self {
            values,
            enclosing: None,
        }
    }
    pub fn root() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(Box::new(Self::global()))
        }
    }
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }
    
    pub fn descend(&mut self) {
        let parent = ::std::mem::replace(self, Self::new());
        self.enclosing = Some(Box::new(parent));
    }

    pub fn ascend(&mut self) {
        let parent = ::std::mem::replace(&mut self.enclosing, None);
        if let Some(parent) = parent {
            *self = *parent;
        } 
    }

    pub fn assign(&mut self, s: &str, new: Value) -> Result<Value, Error> {
        let old = self.get_mut(s)?;
        *old = new.clone();
        Ok(new)
    }

    pub fn define(&mut self, s: String, val: Option<Value>) {
        let resolved = val.unwrap_or_else(|| Value::Nil);
        self.values.insert(s, resolved);
    }

    pub fn get(&self, s: &str) -> Result<Value, Error> {
        if let Some(val) = self.values.get(s) {
            Ok(val.clone())
        } else if let Some(ref enc) = self.enclosing {
            enc.get(s)
        } else {
            Err(Error::Runtime(format!("variable {:?} is not yet defined", s)))
        }
    }

    pub fn get_mut(&mut self, s: &str) -> Result<&mut Value, Error> {
        if let Some(value) = self.values.get_mut(s) {
            Ok(value)
        } else if let Some(ref mut enc) = self.enclosing {
            enc.get_mut(s)
        } else {
            Err(Error::Runtime(format!("variable {:?} is not yet defined", s)))
        }
    }
}
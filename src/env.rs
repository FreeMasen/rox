use crate::{error::Error, globals::*, value::Value};
use std::collections::HashMap;
use std::rc::Rc;
#[derive(Debug, Clone)]
pub struct Env {
    values: HashMap<String, Value>,
    enclosing: Option<Box<Env>>,
}

impl Env {
    fn global() -> Self {
        let mut values = HashMap::new();
        values.insert(String::from("clock"), Value::Global(Rc::new(Clock)));
        values.insert(String::from("mod"), Value::Global(Rc::new(Mod)));
        let mut ret = Self::new();
        ret.values = values;
        ret
    }
    pub fn root() -> Self {
        let globals = Self::global();
        let mut ret = Self::new();
        ret.enclosing = Some(Box::new(globals));
        ret
    }
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    /// Creates a new env with
    /// the provided value cloned into
    /// the `enclosing` property
    pub fn with_cloned(env: &Env) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(Box::new(env.clone())),
        }
    }

    pub fn descend(&mut self) {
        let parent = ::std::mem::replace(self, Self::new());
        self.enclosing = Some(Box::new(parent));
    }

    pub fn descend_into(&mut self, other: Env) {
        let parent = ::std::mem::replace(self, other);
        self.enclosing = Some(Box::new(parent));
    }

    pub fn ascend(&mut self) {
        let parent = ::std::mem::replace(&mut self.enclosing, None);
        if let Some(parent) = parent {
            *self = *parent;
        }
    }

    pub fn ascend_out_of(&mut self) -> Result<Env, Error> {
        let parent = ::std::mem::replace(&mut self.enclosing, None);
        if let Some(parent) = parent {
            let ret = self.clone();
            *self = *parent;
            Ok(ret)
        } else {
            Err(Error::Runtime(format!(
                "Error, attempted to ascend out of env with no parent"
            )))
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
            Err(Error::Runtime(format!(
                "variable {:?} is not yet defined",
                s
            )))
        }
    }

    pub fn get_mut(&mut self, s: &str) -> Result<&mut Value, Error> {
        if let Some(value) = self.values.get_mut(s) {
            Ok(value)
        } else if let Some(ref mut enc) = self.enclosing {
            enc.get_mut(s)
        } else {
            Err(Error::Runtime(format!(
                "variable {:?} is not yet defined",
                s
            )))
        }
    }
}

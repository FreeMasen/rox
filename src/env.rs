use crate::{error::Error, value::Value};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Env {
    values: HashMap<String, Value>,
    enclosing: Option<Box<Env>>,
    pub depth: usize,
}

impl Env {
    fn global() -> Self {
        let mut values = HashMap::new();
        values.insert(String::from("clock"), Value::clock());
        values.insert(String::from("mod"), Value::modulo());
        let mut ret = Self::new(0);
        ret.values = values;
        ret
    }
    pub fn root() -> Self {
        let globals = Self::global();
        let mut ret = Self::new(1);
        ret.enclosing = Some(Box::new(globals));
        ret
    }
    pub fn new(depth: usize) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
            depth,
        }
    }

    pub fn descend(&mut self) {
        log::trace!("decending from, {}", self.depth);
        let parent = ::std::mem::replace(self, Self::new(self.depth + 1));
        self.enclosing = Some(Box::new(parent));
    }

    pub fn ascend_out_of(&mut self) -> Result<Self, Error> {
        let parent = ::std::mem::replace(&mut self.enclosing, None);
        if let Some(parent) = parent {
            let ret = std::mem::replace(self, *parent);
            Ok(ret)
        } else {
            Err(Error::Runtime(String::from(
                "Error, attempted to ascend out of env with no parent",
            )))
        }
    }

    pub fn revert_to(&mut self, depth: usize) -> Result<Vec<Self>, Error> {
        let mut envs = vec![];
        while self.depth > depth {
            let old_self = self.ascend_out_of()?;
            envs.push(old_self)
        }
        Ok(envs)
    }

    pub fn descend_into(&mut self, enc: Env) {
        log::trace!("decending from, {}", self.depth);
        let parent = ::std::mem::replace(self, enc);
        self.enclosing = Some(Box::new(parent));
    }

    pub fn ascend(&mut self) {
        log::trace!("ascending from, {}", self.depth);
        let parent = ::std::mem::replace(&mut self.enclosing, None);
        if let Some(parent) = parent {
            *self = *parent;
        }
    }

    pub fn assign(&mut self, s: &str, new: Value) -> Result<Value, Error> {
        let old = self.get_mut(s, self.depth)?;
        *old = new.clone();
        Ok(new)
    }

    pub fn define(&mut self, s: String, val: Option<Value>) {
        let resolved = val.unwrap_or_else(|| Value::Nil);
        self.values.insert(s, resolved);
    }

    pub fn get(&self, s: &str, id: usize) -> Result<Value, Error> {
        log::trace!("{}: {:#?}", id, self.depth);
        if self.depth > id {
            if let Some(ref inner) = self.enclosing {
                return inner.get(s, id);
            }
        }
        if let Some(val) = self.values.get(s) {
            Ok(val.clone())
        } else if let Some(ref enc) = self.enclosing {
            enc.get(s, id)
        } else {
            Err(Error::Runtime(format!(
                "variable {:?} is not yet defined",
                s
            )))
        }
    }

    pub fn get_mut(&mut self, s: &str, depth: usize) -> Result<&mut Value, Error> {
        if self.depth > depth {
            if let Some(ref mut inner) = self.enclosing {
                return inner.get_mut(s, depth);
            }
        }
        if let Some(value) = self.values.get_mut(s) {
            Ok(value)
        } else if let Some(ref mut enc) = self.enclosing {
            enc.get_mut(s, depth)
        } else {
            Err(Error::Runtime(format!(
                "variable {:?} is not yet defined",
                s
            )))
        }
    }
}

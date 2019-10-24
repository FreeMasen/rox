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
    /// Create a new environment, setting the current environment
    /// to the `enclosing` 
    pub fn descend(&mut self) {
        log::trace!("decending from, {}", self.depth);
        let parent = ::std::mem::replace(self, Self::new(self.depth + 1));
        self.enclosing = Some(Box::new(parent));
    }
    /// Move the environment up one level, returning the
    /// old child environment
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
    /// Unwraps the environment to the specified depth
    /// returning a vector of the environments removed.
    /// In the return value the previous leaf environment will be
    /// at index 0 while the nearest child will be in the last position
    /// ```text
    /// original
    /// ----
    ///  root
    ///   |_child 1
    ///      |_child 2
    ///        |_child 3
    ///          |_child 4
    ///            |_child 5
    ///              |_child 6
    /// ---
    /// revert_to(2)
    /// ---
    /// root
    ///   |_child 1
    ///     |_child 2
    /// 
    /// return:[child 6, child 5, child 4, child 3]
    /// ```
    pub fn revert_to(&mut self, depth: usize) -> Result<Vec<Self>, Error> {
        let mut envs = vec![];
        while self.depth > depth {
            let old_self = self.ascend_out_of()?;
            envs.push(old_self)
        }
        Ok(envs)
    }
    /// Same as descend but uses the provided environment
    /// for the new child
    pub fn descend_into(&mut self, enc: Env) {
        log::trace!("decending from, {}", self.depth);
        let parent = ::std::mem::replace(self, enc);
        self.enclosing = Some(Box::new(parent));
    }
    /// Move up one level, discarding the old child environment
    pub fn ascend(&mut self) {
        log::trace!("ascending from, {}", self.depth);
        let parent = ::std::mem::replace(&mut self.enclosing, None);
        if let Some(parent) = parent {
            *self = *parent;
        }
    }
    /// Assign a new value to a previously defined
    /// variable
    pub fn assign(&mut self, s: &str, new: Value) -> Result<Value, Error> {
        let old = self.get_mut(s, self.depth)?;
        *old = new.clone();
        Ok(new)
    }
    /// Define a new variable by name, setting to Nil if 
    /// no value is provided
    pub fn define(&mut self, s: String, val: Option<Value>) {
        let resolved = val.unwrap_or_else(|| Value::Nil);
        self.values.insert(s, resolved);
    }
    /// Get a clone of a value from the environment
    /// skipping any environment's 
    /// who's depth is greater than the depth provided
    pub fn get(&self, s: &str, depth: usize) -> Result<Value, Error> {
        log::trace!("{}: {:#?}", depth, self.depth);
        if self.depth > depth {
            if let Some(ref inner) = self.enclosing {
                return inner.get(s, depth);
            }
        }
        if let Some(val) = self.values.get(s) {
            Ok(val.clone())
        } else if let Some(ref enc) = self.enclosing {
            enc.get(s, depth)
        } else {
            Err(Error::Runtime(format!(
                "variable {:?} is not yet defined",
                s
            )))
        }
    }
    /// Get a mutable reference to a value in the
    /// environment, skipping any environment's 
    /// who's depth is greater than the depth provided
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

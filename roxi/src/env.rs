use crate::{error::Error, value::Value};
use hash_chain::ChainMap;

#[derive(Debug, Clone)]
pub struct EnvStub {
    pub values: ChainMap<String, Value>,
    pub idx: usize,
}

#[derive(Debug, Clone)]
pub struct Env {
    values: ChainMap<String, Value>,
}

impl Env {
    pub fn depth(&self) -> usize {
        self.values.child_len()
    }

    pub fn root() -> Self {
        let mut start = Self::global();
        start.values.new_child();
        start
    }

    fn global() -> Self {
        let mut values = ChainMap::default();
        values.insert(String::from("clock"), Value::clock());
        values.insert(String::from("mod"), Value::modulo());
        let mut ret = Self::new(0);
        ret.values = values;
        ret
    }
    
    pub fn new(depth: usize) -> Self {
        let mut values = ChainMap::default();
        for _ in 0..depth {
            values.new_child()
        }
        Self {
            values,
        }
    }

    pub fn descend(&mut self) {
        log::trace!("decending from, {} {:?}", self.depth(), self);
        self.values.new_child()
    }

    pub fn ascend(&mut self) {
        log::trace!("ascending from, {}", self.depth());
        let _ = self.values.remove_child();
    }

    pub fn assign(&mut self, s: &str, new: Value) -> Result<Value, Error> {
        let old = self.get_mut(s)?;
        let _ = std::mem::replace(old, new.clone());
        Ok(new)
    }

    pub fn define(&mut self, s: &str, val: Option<Value>) {
        let resolved = val.unwrap_or_else(|| Value::Nil);
        self.values.insert(s.to_string(), resolved);
    }

    pub fn get(&self, s: &str) -> Result<Value, Error> {
        log::trace!("get {:?} {}", s, self.depth());
        if let Some(val) = self.values.get(s) {
            Ok(val.clone())
        } else {
            Err(Error::Runtime(format!(
                "variable {:?} is not yet defined",
                s
            )))
        }
    }

    pub fn get_mut(&mut self, s: &str) -> Result<&mut Value, Error> {
        if let Some(val) = self.values.get_mut(s) {
            Ok(val)
        } else {
            Err(Error::Runtime(format!(
                "variable {:?} is not yet defined",
                s
            )))
        }
    }

    pub fn split_to_base(&mut self) -> Self {
        self.split(2)
    }

    pub fn split(&mut self, idx: usize) -> Self {
        let mut values = self.values.split_off(idx);
        if values.child_len() == 0 {
            values.new_child()
        }
        Self {
            values,
        }
    }

    pub fn append(&mut self, mut other: Self) {
        self.values.append(&mut other.values)
    }

    pub fn clone_to_base(&self) -> Self {
        let mut other = self.clone();
        other.split(2)
    }
}

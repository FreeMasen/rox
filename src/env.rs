use crate::{error::Error, value::Value};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Env {
    maps: Vec<HashMap<String, Value>>,
}

type Map = HashMap<String, Value>;

impl Env {
    pub fn depth(&self) -> usize {
        self.maps.len()
    }
    fn global() -> Map {
        let mut values = HashMap::new();
        values.insert(String::from("clock"), Value::clock());
        values.insert(String::from("mod"), Value::modulo());
        values
    }
    
    pub fn new() -> Self {
        Self {
            maps: vec![Self::global(), HashMap::new()],
        }
    }
    /// Create a new environment, setting the current environment
    /// to the `enclosing` 
    pub fn descend(&mut self) {
        log::trace!("decending from, {}", self.maps.len());
        self.maps.push(HashMap::new());
    }
    pub fn descend_into(&mut self, maps: Vec<Map>) {
        self.maps.extend(maps)
    }
    /// Move the environment up one level, returning the
    /// old child environment
    pub fn ascend_out_of(&mut self) -> Result<Map, Error> {
        if self.maps.len() > 1 {
            // this should never fail because of
            // the check above
            Ok(self.maps.pop().unwrap())
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
    pub fn revert_to(&mut self, depth: usize) -> Result<Vec<Map>, Error> {
        log::trace!("reverting to {} from {}", depth, self.depth());
        let mut envs = vec![];
        while self.maps.len() > depth {
            envs.push(self.maps.pop().unwrap());
        }
        Ok(envs)
    }

    /// Move up one level, discarding the old child environment
    pub fn ascend(&mut self) {
        let _ = self.ascend_out_of();
    }
    /// Assign a new value to a previously defined
    /// variable
    pub fn assign(&mut self, s: &str, new: Value) -> Result<Value, Error> {
        let old = self.get_mut(s, self.maps.len())?;
        *old = new.clone();
        Ok(new)
    }
    pub fn assign_at(&mut self, s: &str, new: Value, depth: usize) -> Result<(), Error> {
        *self.get_mut(s, depth)? = new;
        Ok(())
    }
    /// Define a new variable by name, setting to Nil if 
    /// no value is provided
    pub fn define(&mut self, s: String, val: Option<Value>) {
        let resolved = val.unwrap_or_else(|| Value::Nil);
        if let Some(values) = self.maps.last_mut() {
            values.insert(s, resolved);
        }
    }
    /// Get a clone of a value from the environment
    /// skipping any environment's 
    /// who's depth is greater than the depth provided
    pub fn get(&self, s: &str, depth: usize) -> Result<Value, Error> {
        log::trace!("{}: {:#?}", depth, self.maps.len());
        let maps = if let Some(i) = self.maps.get(0..depth) {
            i
        } else {
            &self.maps
        }.iter().rev();
        for map in maps {
            if let Some(val) = map.get(s) {
                return Ok(val.clone());
            }
        }
        Err(Error::Runtime(format!(
            "variable {:?} is not yet defined",
            s
        )))
    }
    /// Get a mutable reference to a value in the
    /// environment, skipping any environment's 
    /// who's depth is greater than the depth provided
    pub fn get_mut(&mut self, s: &str, depth: usize) -> Result<&mut Value, Error> {
        if self.depth() >= depth {
            for map in self.maps.iter_mut().rev() {
                if let Some(val) = map.get_mut(s) {
                    return Ok(val);
                }
            }
        } else {
            if let Some(maps) = self.maps.get_mut(0..depth) {
                for map in maps.iter_mut().rev() {
                    if let Some(val) = map.get_mut(s) {
                        return Ok(val);
                    }
                }
            }
        }
        Err(Error::Runtime(format!(
            "variable {:?} is not yet defined",
            s
        )))
    }
}

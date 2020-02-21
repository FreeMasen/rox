use crate::{
    callable::Callable, error::Error, func::Func, interpreter::Interpreter, stmt::Function,
    value::Value,
};
use std::collections::HashMap;
#[derive(Clone, Debug)]
pub struct Class {
    pub name: String,
    pub methods: Vec<Function>,
    pub super_class: Option<Box<Class>>,
}
#[derive(Clone, Debug)]
pub struct ClassInstance {
    pub class: Class,
    pub fields: HashMap<String, Value>,
    pub methods: HashMap<String, Method>,
}

#[derive(Clone, Debug)]
pub struct Method {
    pub func: Func,
    pub this_depth: usize,
    pub this_name: String,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}", self.this_name, self.func.name)
    }
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
        let mut init: Option<Method> = None;
        for def in &self.methods {
            let func = Func {
                name: def.name.to_string(),
                env_id: int.current_depth + 1,
                params: def.params.clone(),
                body: def.body.clone(),
            };
            let meth = Method {
                func,
                this_depth: int.current_depth,
                this_name: String::new(), // will get replaced at caller
            };
            if meth.func.name == "init" {
                init = Some(meth)
            } else {
                methods.insert(def.name.to_string(), meth);
            }
        }
        let ret = ClassInstance {
            fields: HashMap::new(),
            class: self.clone(),
            methods,
        };

        if let Some(mut init) = init {
            int.env.descend();
            init.this_name = "*".to_string();
            init.this_depth = int.env.depth();
            let prev_depth = int.current_depth;
            int.current_depth = int.env.depth();
            int.env.define("*".to_string(), Some(Value::Class(ret)));
            init.call(int, args)?;
            let updated = int.env.get("*", int.env.depth())?;
            int.env.ascend();
            int.current_depth = prev_depth;
            Ok(updated)
        } else {
            Ok(Value::Class(ret))
        }
    }
}

impl Callable for Method {
    fn name(&self) -> &str {
        self.func.name()
    }
    fn arity(&self) -> usize {
        self.func.arity()
    }
    fn call(&self, int: &mut Interpreter, args: &[Value]) -> Result<Value, Error> {
        let this = int.env.get(&self.this_name, self.this_depth)?;
        int.env.descend();
        int.current_depth = int.env.depth();
        int.env.define("this".to_string(), Some(this));
        for (name, value) in self.func.params.iter().cloned().zip(args.iter().cloned()) {
            int.env.define(name, Some(value));
        }
        let ret = match int.execute_block(&self.func.body) {
            Ok(_) => Ok(Value::Nil),
            Err(Error::Return(v)) => Ok(v),
            Err(e) => Err(e),
        };
        let updated_this = int.env.get("this", int.current_depth)?;
        int.env.ascend();
        int.env.assign(&self.this_name, updated_this)?;
        ret
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
            Ok(Value::Method(val.clone()))
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

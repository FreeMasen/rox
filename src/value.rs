use crate::{
    callable::Callable, 
    class::{ClassInstance, Class}, 
    expr::Literal, func::Func,
    globals::NativeFunc,
};

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
    Func(Func),
    Init(Class),
    NativeFunc(NativeFunc),
    Class(ClassInstance),
}
impl From<Literal> for Value {
    fn from(other: Literal) -> Self {
        match other {
            Literal::String(s) => Self::String(s),
            Literal::Number(n) => Self::Number(n),
            Literal::Bool(b) => Self::Bool(b),
            Literal::Nil => Self::Nil,
        }
    }
}

impl ::std::fmt::Display for Value {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Number(n) => n.fmt(f),
            Value::Bool(b) => b.fmt(f),
            Value::Nil => write!(f, "nil"),
            Value::Func(func) => write!(f, "{}", func),
            Value::Class(class) => write!(f, "[{} instance]", class.class.name),
            Value::Init(class) => write!(f, "[ctor {}]", class.name()),
            Value::NativeFunc(c) => write!(f, "[native fn {}]", c.name()),
        }
    }
}

impl ::std::cmp::PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Number(l), Value::Number(r)) => l == r,
            (Value::Bool(l), Value::Bool(r)) => l == r,
            (Value::Nil, Value::Nil) => true,
            (Value::Func(l), Value::Func(r)) => l.name() == r.name(),
            _ => false,
        }
    }
}

impl Value {
    pub fn clock() -> Self {
        Value::NativeFunc(NativeFunc::Clock(crate::globals::Clock))
    }
    pub fn modulo() -> Self {
        Value::NativeFunc(NativeFunc::Mod(crate::globals::Mod))
    }

}
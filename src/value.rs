use crate::{
    callable::Callable,
    expr::Literal
};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
    Func(Rc<dyn Callable>),
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
            Value::Func(func) => write!(f, "[fn {}]", func.name()),
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
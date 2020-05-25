use std::{
    borrow::Cow,
    collections::HashMap,
    hash::BuildHasherDefault,
};
use hashers::fnv::FNV1aHasher64;
type HashTable<'a> = HashMap<Cow<'a, str>, Value, BuildHasherDefault<FNV1aHasher64>>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Nil,
    Obj { idx: usize }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Obj<'a> {
    String(Cow<'a, str>),
    HashTable { entries: HashTable<'a> }
}

impl<'a> Obj<'a> {
    pub fn string(s: &'a str) -> Self {
        Obj::String(Cow::Borrowed(s))
    }
    pub fn hash_table(entries: HashTable<'a>) -> Self {
        Obj::HashTable { entries }
    }
}

impl core::ops::Add for Value {
    type Output = Value;
    fn add(self, other: Value) -> Value {
        use Value::*;
        match (self, other) {
            (Number(l), Number(r)) => Number(l + r),
            (Nil, Nil) => Nil,
            _ => unimplemented!("cannot add {:?} and {:?}", self, other),
        }
    }
}

impl core::ops::Sub for Value {
    type Output = Value;
    fn sub(self, other: Value) -> Value {
        use Value::*;
        match (self, other) {
            (Number(l), Number(r)) => Number(l - r),
            _ => unimplemented!("cannot add {:?} and {:?}", self, other),
        }
    }
}
impl core::ops::Mul for Value {
    type Output = Value;
    fn mul(self, other: Value) -> Value {
        use Value::*;
        match (self, other) {
            (Number(l), Number(r)) => Number(l * r),
            _ => unimplemented!("cannot add {:?} and {:?}", self, other),
        }
    }
}

impl core::ops::Div for Value {
    type Output = Value;
    fn div(self, other: Value) -> Value {
        use Value::*;
        match (self, other) {
            (Number(l), Number(r)) => Number(l / r),
            _ => unimplemented!("cannot add {:?} and {:?}", self, other),
        }
    }
}

impl core::ops::Neg for Value {
    type Output = Value;
    fn neg(self) -> Self {
        use Value::*;
        if let Number(n) = self {
            Number(-n)
        } else {
            panic!("Only numbers can be negated");
        }
    }
}

impl core::ops::Not for Value {
    type Output = Value;
    fn not(self) -> Self {
        use Value::*;
        match self {
            Boolean(b) => Boolean(!b),
            Nil => Boolean(true),
            _ => Boolean(false),
        }
    }
}

impl core::cmp::PartialOrd for Value {
    fn partial_cmp(&self, other: &Value) -> Option<core::cmp::Ordering> {
        use Value::*;
        match (self, other) {
            (Number(l), Number(r)) => l.partial_cmp(r),
            _ => None,
        }
    }
}

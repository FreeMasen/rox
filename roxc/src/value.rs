#[derive(Debug, Clone, Copy)]
pub enum Value {
    Number(f64),
}

impl core::ops::Add for Value {
    type Output = Value;
    fn add(self, other: Value) -> Value {
        use Value::*;
        match (self, other) {
            (Number(l), Number(r)) => Number(l + r),
            // _ => unimplemented!("cannot add {:?} and {:?}", self, other),
        }
    }
}

impl core::ops::Sub for Value {
    type Output = Value;
    fn sub(self, other: Value) -> Value {
        use Value::*;
        match (self, other) {
            (Number(l), Number(r)) => Number(l - r),
            // _ => unimplemented!("cannot add {:?} and {:?}", self, other),
        }
    }
}
impl core::ops::Mul for Value {
    type Output = Value;
    fn mul(self, other: Value) -> Value {
        use Value::*;
        match (self, other) {
            (Number(l), Number(r)) => Number(l * r),
            // _ => unimplemented!("cannot add {:?} and {:?}", self, other),
        }
    }
}

impl core::ops::Div for Value {
    type Output = Value;
    fn div(self, other: Value) -> Value {
        use Value::*;
        match (self, other) {
            (Number(l), Number(r)) => Number(l / r),
            // _ => unimplemented!("cannot add {:?} and {:?}", self, other),
        }
    }
}

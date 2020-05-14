#[derive(Clone, Copy, Debug)]
pub enum OpCode {
    Constant { idx: usize },
    Return,
    Negate,
    Add,
    Sub,
    Mul,
    Div,
}

impl std::fmt::Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use OpCode::*;
        match self {
            Constant { .. } => write!(f, "Constant"),
            Return | Negate | Add | Sub | Mul | Div => core::fmt::Debug::fmt(&self, f),
        }
    }
}

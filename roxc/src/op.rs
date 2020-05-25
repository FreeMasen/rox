#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OpCode {
    Constant { idx: usize },
    Return,
    Negate,
    Add,
    Sub,
    Mul,
    Div,
    True,
    False,
    Nil,
    Not,
    Eq,
    Gtr,
    Less,
}

impl std::fmt::Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use OpCode::*;
        match self {
            Constant { .. } => write!(f, "Constant"),
            True => write!(f, "true"),
            False => write!(f, "false"),
            Nil => write!(f, "nil"),
            _ => core::fmt::Debug::fmt(&self, f),
        }
    }
}

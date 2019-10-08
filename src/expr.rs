use super::error::Error;
use super::token::Token;
#[derive(Debug, Clone)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Var(String),
    Assign {
        name: String,
        value: Box<Expr>,
    },
    Log {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
}
#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

impl ::std::fmt::Display for Literal {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Literal::String(s) => write!(f, "\"{}\"", s),
            Literal::Number(n) => n.fmt(f),
            Literal::Bool(b) => b.fmt(f),
            Literal::Nil => write!(f, "nil"),
        }
    }
}

impl Expr {
    pub fn accept<T>(&self, visitor: &mut impl ExprVisitor<T>) -> Result<T, Error> {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => visitor.visit_bin(left, operator, right),
            Expr::Grouping(_) => visitor.visit_group(self),
            Expr::Literal(lit) => visitor.visit_lit(lit),
            Expr::Unary { operator, right } => visitor.visit_un(operator, right),
            Expr::Var(name) => visitor.visit_var(name),
            Expr::Assign { name, value } => visitor.visit_assign(name, value),
            Expr::Log {
                left,
                operator,
                right,
            } => visitor.visit_log(left, operator, right),
        }
    }

    pub fn binary(left: Expr, right: Expr, op: Token) -> Self {
        Expr::Binary {
            left: Box::new(left),
            right: Box::new(right),
            operator: op,
        }
    }

    pub fn unary(op: Token, right: Expr) -> Self {
        Expr::Unary {
            operator: op,
            right: Box::new(right),
        }
    }
    pub fn grouping(inner: Expr) -> Self {
        Expr::Grouping(Box::new(inner))
    }
    pub fn assign(name: String, value: Expr) -> Self {
        Expr::Assign {
            name,
            value: Box::new(value),
        }
    }
    pub fn log(left: Expr, right: Expr, op: Token) -> Self {
        Expr::Log {
            left: Box::new(left),
            operator: op,
            right: Box::new(right),
        }
    }
}

pub trait ExprVisitor<T> {
    fn visit_bin(&mut self, left: &Expr, op: &Token, right: &Expr) -> Result<T, Error>;
    fn visit_group(&mut self, group: &Expr) -> Result<T, Error>;
    fn visit_lit(&self, lit: &Literal) -> Result<T, Error>;
    fn visit_un(&mut self, op: &Token, ex: &Expr) -> Result<T, Error>;
    fn visit_var(&mut self, name: &str) -> Result<T, Error>;
    fn visit_assign(&mut self, name: &str, value: &Expr) -> Result<T, Error>;
    fn visit_log(&mut self, left: &Expr, op: &Token, right: &Expr) -> Result<T, Error>;
}

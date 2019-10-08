use super::{
    error::Error,
    expr::{Expr, ExprVisitor, Literal},
    statement::{Stmt, StmtVisitor},
    token::{Token, TokenType},
};

use std::collections::HashMap;

pub struct Interpreter {
    pub env: Env,
}

pub struct Env {
    values: HashMap<String, Literal>,
}

impl Env {
    pub fn define(&mut self, s: String, val: Option<Literal>) {
        self.values.insert(s, val.unwrap_or(Literal::Nil));
    }

    pub fn get(&self, s: &str) -> Result<Literal, Error> {
        if let Some(val) = self.values.get(s) {
            Ok(val.clone())
        } else {
            Err(Error::Runtime(format!("variable {} is not yet defined", s)))
        }
    }
}

type IntResult = Result<Literal, Error>;

impl ExprVisitor<IntResult> for Interpreter {
    fn visit_bin(&self, left: &Expr, op: &Token, right: &Expr) -> IntResult {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;
        let ret = match (&op.kind, &left, &right) {
            (TokenType::Minus, Literal::Number(lhs), Literal::Number(rhs)) => {
                Literal::Number(lhs - rhs)
            }
            (TokenType::Slash, Literal::Number(lhs), Literal::Number(rhs)) => {
                Literal::Number(lhs / rhs)
            }
            (TokenType::Star, Literal::Number(lhs), Literal::Number(rhs)) => {
                Literal::Number(lhs * rhs)
            }
            (TokenType::Plus, Literal::Number(lhs), Literal::Number(rhs)) => {
                Literal::Number(lhs + rhs)
            }
            (TokenType::Greater, Literal::Number(lhs), Literal::Number(rhs)) => {
                Literal::Bool(lhs > rhs)
            }
            (TokenType::GreaterEqual, Literal::Number(lhs), Literal::Number(rhs)) => {
                Literal::Bool(lhs >= rhs)
            }
            (TokenType::Less, Literal::Number(lhs), Literal::Number(rhs)) => {
                Literal::Bool(lhs < rhs)
            }
            (TokenType::LessEqual, Literal::Number(lhs), Literal::Number(rhs)) => {
                Literal::Bool(lhs <= rhs)
            }
            (TokenType::Plus, Literal::String(lhs), Literal::String(rhs)) => {
                Literal::String(format!("{}{}", lhs, rhs))
            }
            (TokenType::EqualEqual, l, r) => Literal::Bool(Self::is_equal(l, r)),
            (TokenType::BangEqual, l, r) => Literal::Bool(!Self::is_equal(l, r)),
            _ => {
                return Err(Error::Runtime(format!(
                    "Invalid binary operation: {:?} {:?} {:?}",
                    left, op, right
                )))
            }
        };
        Ok(ret)
    }
    fn visit_group(&self, group: &Expr) -> IntResult {
        if let Expr::Grouping(inner) = group {
            self.evaluate(inner)
        } else {
            Err(Error::Runtime("Visited group unexectedly".into()))
        }
    }
    fn visit_lit(&self, lit: &Literal) -> IntResult {
        Ok(lit.clone())
    }
    fn visit_un(&self, op: &Token, ex: &Expr) -> IntResult {
        let right = self.evaluate(ex)?;
        let ret = match (&op.kind, right) {
            (TokenType::Minus, Literal::Number(n)) => Literal::Number(-n),
            (TokenType::Plus, Literal::Number(n)) => Literal::Number(n),
            (TokenType::Bang, a) => Literal::Bool(!Self::is_truthy(&a)),
            _ => {
                return Err(Error::Runtime(format!(
                    "Invalid unary operation {:?}, {:?}",
                    op, ex
                )))
            }
        };
        Ok(ret)
    }
    fn visit_ident(&self, name: &str) -> IntResult {
        self.env.get(name)
    }
}

impl StmtVisitor<()> for Interpreter {
    fn visit_expr_stmt(&mut self, expr: &Expr) -> Result<(), Error> {
        self.evaluate(expr)?;
        Ok(())
    }
    fn visit_print_stmt(&mut self, expr: &Expr) -> Result<(), Error> {
        println!("{}", self.evaluate(expr)?);
        Ok(())
    }
    fn visit_var_stmt(&mut self, name: String, expr: Option<Expr>) -> Result<(), Error> {
        let value = if let Some(expr) = expr {
            let val = expr.accept(self)?;
            Some(val)
        } else {
            None
        };
        self.env.define(name, value);
        Ok(())
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Env {
                values: HashMap::new(),
            }
        }
    }
    pub fn interpret(&mut self, stmt: &Stmt) -> Result<(), Error> {
        stmt.accept(self)
    }

    pub fn evaluate(&self, expr: &Expr) -> IntResult {
        expr.accept(self)
    }

    fn is_truthy(lit: &Literal) -> bool {
        match lit {
            Literal::Nil => false,
            Literal::Bool(b) => *b,
            _ => true,
        }
    }

    fn is_equal(lhs: &Literal, rhs: &Literal) -> bool {
        match (lhs, rhs) {
            (Literal::Nil, Literal::Nil) => true,
            (Literal::String(l), Literal::String(r)) => l == r,
            (Literal::Number(l), Literal::Number(r)) => l == r,
            (Literal::Bool(l), Literal::Bool(r)) => l == r,
            _ => false,
        }
    }
}

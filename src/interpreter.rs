use super::{
    error::Error,
    expr::{Expr, ExprVisitor, Literal},
    statement::{Stmt, StmtVisitor},
    token::{Token, TokenType},
};

use std::collections::HashMap;

pub struct Interpreter {
    pub env: Env,
    pub indent: usize,
}
#[derive(Debug, Clone)]
pub struct Env {
    values: HashMap<String, Literal>,
    enclosing: Option<Box<Env>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }
    pub fn with(enclosing: Env) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(Box::new(enclosing)),
        }
    }

    pub fn close(&self) -> Result<Self, Error> {
        if let Some(ret) = &self.enclosing {
            Ok(*ret.clone())
        } else {
            Err(Error::Runtime(format!("Invalid environment state")))
        }
    }

    pub fn assign(&mut self, s: &str, new: Literal) -> Result<Literal, Error> {
        let old = self.get_mut(s)?;
        *old = new.clone();
        Ok(new)
    }

    pub fn define(&mut self, s: String, val: Option<Literal>) {
        self.values.insert(s, val.unwrap_or(Literal::Nil));
    }

    pub fn get(&self, s: &str) -> Result<Literal, Error> {
        if let Some(val) = self.values.get(s) {
            Ok(val.clone())
        } else if let Some(ref enc) = self.enclosing {
            enc.get(s)
        } else {
            Err(Error::Runtime(format!("variable {:?} is not yet defined", s)))
        }
    }

    pub fn get_mut(&mut self, s: &str) -> Result<&mut Literal, Error> {
        if let Some(value) = self.values.get_mut(s) {
            Ok(value)
        } else if let Some(ref mut enc) = self.enclosing {
            enc.get_mut(s)
        } else {
            Err(Error::Runtime(format!("variable {:?} is not yet defined", s)))
        }
    }
}

type IntResult = Result<Literal, Error>;

impl ExprVisitor<Literal> for Interpreter {
    fn visit_bin(&mut self, left: &Expr, op: &Token, right: &Expr) -> IntResult {
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

    fn visit_group(&mut self, group: &Expr) -> IntResult {
        if let Expr::Grouping(inner) = group {
            self.evaluate(inner)
        } else {
            Err(Error::Runtime("Visited group unexectedly".into()))
        }
    }

    fn visit_lit(&self, lit: &Literal) -> IntResult {
        Ok(lit.clone())
    }

    fn visit_un(&mut self, op: &Token, ex: &Expr) -> IntResult {
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

    fn visit_var(&mut self, name: &str) -> IntResult {
        self.env.get(name)
    }

    fn visit_assign(&mut self, name: &str, expr: &Expr) -> IntResult {
        let val = self.evaluate(expr)?;
        self.env.assign(name, val)
    }

    fn visit_log(&mut self, left: &Expr, op: &Token, right: &Expr) -> IntResult {
        let left = self.evaluate(left)?;
        let ret = match (&op.kind, Self::is_truthy(&left)) {
            (TokenType::Or, true) => Literal::Bool(true),
            (TokenType::Or, false) | (TokenType::And, true) => {
                Literal::Bool(Self::is_truthy(&self.evaluate(right)?))
            }
            _ => Literal::Bool(false),
        };
        Ok(ret)
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

    fn visit_block_stmt(&mut self, list: &[Stmt]) -> Result<(), Error> {
        self.execute_block(list)?;
        Ok(())
    }

    fn visit_if_stmt(
        &mut self,
        test: &Expr,
        cons: &Stmt,
        alt: &Option<Box<Stmt>>,
    ) -> Result<(), Error> {
        let boolean = self.evaluate(test)?;
        if Self::is_truthy(&boolean) {
            self.interpret(cons)
        } else if let Some(alt) = alt {
            self.interpret(alt)
        } else {
            Ok(())
        }
    }

    fn visit_while_stmt(&mut self, test: &Expr, body: &Stmt) -> Result<(), Error> {
        while Self::is_truthy(&self.evaluate(test)?) {
            self.interpret(body)?;
        }
        Ok(())
    }

    fn visit_for_stmt(
        &mut self,
        init: &Option<Box<Stmt>>,
        test: &Option<Expr>,
        update: &Option<Expr>,
        body: &Stmt,
    ) -> Result<(), Error> {
        let mut while_body = vec![];
        if let Some(inner) = init {
            while_body.push((**inner).clone());
        }
        while_body.push(body.clone());
        if let Some(update) = update {
            while_body.push(Stmt::Expr(update.clone()));
        }
        let cond = if let Some(test) = test {
            test.clone()
        } else {
            Expr::Literal(Literal::Bool(true))
        };
        self.visit_while_stmt(&cond, &Stmt::Block(while_body))
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Env::new(),
            indent: 0,
        }
    }

    pub fn interpret(&mut self, stmt: &Stmt) -> Result<(), Error> {
        stmt.accept(self)
    }

    pub fn evaluate(&mut self, expr: &Expr) -> IntResult {
        expr.accept(self)
    }

    pub fn execute_block(&mut self, stmts: &[Stmt]) -> Result<(), Error> {
        self.env = Env::with(self.env.clone());
        for stmt in stmts {
            if let Err(e) = self.interpret(stmt) {
                self.env = self.env.close()?;
                return Err(e);
            }
        }
        self.env = self.env.close()?;
        Ok(())
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


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn while_block() {
        let lox = "
var i = 0;
while (i < 100) {
    print i;
    i = i + 1;
}
";
        let mut int = Interpreter::new();
        let mut parser = crate::parser::Parser::new(
            crate::scanner::Scanner::new(lox.into()).unwrap()
        );
        while let Some(stmt) = parser.next() {
            int.interpret(&stmt.unwrap()).unwrap();
        }
    }
}
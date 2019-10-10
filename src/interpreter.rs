use super::{
    callable::Callable,
    error::Error,
    expr::{Expr, ExprVisitor, Literal},
    env::{Env},
    func::Func,
    stmt::{Stmt, StmtVisitor},
    token::{Token, TokenType},
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
            Value::Func(_) => write!(f, "[function]"),
        }
    }
}
pub struct Interpreter {
    pub env: Env,
}

type IntResult = Result<Value, Error>;

impl ExprVisitor<Value> for Interpreter {
    fn visit_bin(&mut self, left: &Expr, op: &Token, right: &Expr) -> IntResult {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;
        let ret = match (&op.kind, &left, &right) {
            (TokenType::Minus, Value::Number(lhs), Value::Number(rhs)) => {
                Value::Number(lhs - rhs)
            }
            (TokenType::Slash, Value::Number(lhs), Value::Number(rhs)) => {
                Value::Number(lhs / rhs)
            }
            (TokenType::Star, Value::Number(lhs), Value::Number(rhs)) => {
                Value::Number(lhs * rhs)
            }
            (TokenType::Plus, Value::Number(lhs), Value::Number(rhs)) => {
                Value::Number(lhs + rhs)
            }
            (TokenType::Greater, Value::Number(lhs), Value::Number(rhs)) => {
                Value::Bool(lhs > rhs)
            }
            (TokenType::GreaterEqual, Value::Number(lhs), Value::Number(rhs)) => {
                Value::Bool(lhs >= rhs)
            }
            (TokenType::Less, Value::Number(lhs), Value::Number(rhs)) => {
                Value::Bool(lhs < rhs)
            }
            (TokenType::LessEqual, Value::Number(lhs), Value::Number(rhs)) => {
                Value::Bool(lhs <= rhs)
            }
            (TokenType::Plus, Value::String(lhs), Value::String(rhs)) => {
                Value::String(format!("{}{}", lhs, rhs))
            }
            (TokenType::EqualEqual, l, r) => Value::Bool(Self::is_equal(l, r)),
            (TokenType::BangEqual, l, r) => Value::Bool(!Self::is_equal(l, r)),
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
        Ok(lit.clone_into())
    }

    fn visit_un(&mut self, op: &Token, ex: &Expr) -> IntResult {
        let right = self.evaluate(ex)?;
        let ret = match (&op.kind, right) {
            (TokenType::Minus, Value::Number(n)) => Value::Number(-n),
            (TokenType::Plus, Value::Number(n)) => Value::Number(n),
            (TokenType::Bang, a) => Value::Bool(!Self::is_truthy(&a)),
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
            (TokenType::Or, true) => Value::Bool(true),
            (TokenType::Or, false) | (TokenType::And, true) => {
                Value::Bool(Self::is_truthy(&self.evaluate(right)?))
            }
            _ => Value::Bool(false),
        };
        Ok(ret)
    }

    fn visit_call(&mut self, callee: &Expr, arguments: &[Expr]) -> IntResult {
        let mut callee = self.evaluate(callee)?;
        let args = arguments.iter().map(|e| self.evaluate(e)).collect::<Result<Vec<Value>, Error>>()?;
        if let Value::Func(ref mut f) = callee {
            f.call(self, &args)
        } else {
            Err(Error::Runtime(format!("Attempt to call a something that is not a function {}", callee)))
        }
    }
}

impl StmtVisitor<Option<Value>> for Interpreter {
    fn visit_expr_stmt(&mut self, expr: &Expr) -> Result<Option<Value>, Error> {
        self.evaluate(expr)?;
        Ok(None)
    }

    fn visit_print_stmt(&mut self, expr: &Expr) -> Result<Option<Value>, Error> {
        println!("{}", self.evaluate(expr)?);
        Ok(None)
    }

    fn visit_var_stmt(&mut self, name: String, expr: Option<Expr>) -> Result<Option<Value>, Error> {
        let value = if let Some(expr) = expr {
            let val = expr.accept(self)?;
            Some(val)
        } else {
            None
        };
        self.env.define(name, value);
        Ok(None)
    }

    fn visit_block_stmt(&mut self, list: &[Stmt]) -> Result<Option<Value>, Error> {
        self.execute_block(list)?;
        Ok(None)
    }
    
    fn visit_if_stmt(
        &mut self,
        test: &Expr,
        cons: &Stmt,
        alt: &Option<Box<Stmt>>,
    ) -> Result<Option<Value>, Error> {
        let boolean = self.evaluate(test)?;
        if Self::is_truthy(&boolean) {
            self.interpret(cons)
        } else if let Some(alt) = alt {
            self.interpret(alt)
        } else {
            Ok(None)
        }
    }

    fn visit_while_stmt(&mut self, test: &Expr, body: &Stmt) -> Result<Option<Value>, Error> {
        while Self::is_truthy(&self.evaluate(test)?) {
            self.interpret(body)?;
        }
        Ok(None)
    }

    fn visit_func_decl(&mut self, name: &str, params: &[String], body: &[Stmt]) -> Result<Option<Value>, Error> {
        let func = Func {
            params: params.to_vec(),
            body: body.to_vec(),
        };
        let func = Rc::new(func);
        let func = Value::Func(func);
        self.env.define(name.to_string(), Some(func));
        Ok(None)
    }

    fn visit_return_stmt(&mut self, expr: &Option<Expr>) -> Result<Option<Value>, Error> {
        Ok(if let Some(expr) = expr {
            Some(self.evaluate(expr)?)
        } else {
            Some(Value::Nil)
        })
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Env::root(),
        }
    }

    pub fn interpret(&mut self, stmt: &Stmt) -> Result<Option<Value>, Error> {
        stmt.accept(self)
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<Value, Error> {
        expr.accept(self)
    }

    pub fn execute_block(&mut self, stmts: &[Stmt]) -> Result<Option<Value>, Error> {
        self.env.descend();
        for stmt in stmts {
            match self.interpret(stmt) {
                Ok(maybe) => {
                    if let Some(val) = maybe {
                        self.env.ascend();
                        return Ok(Some(val));
                    }
                },
                Err(e) => {
                    self.env.ascend();
                    return Err(e);
                }
            }
        }
        self.env.ascend();
        Ok(None)
    }

    fn is_truthy(lit: &Value) -> bool {
        match lit {
            Value::Nil => false,
            Value::Bool(b) => *b,
            _ => true,
        }
    }

    fn is_equal(lhs: &Value, rhs: &Value) -> bool {
        match (lhs, rhs) {
            (Value::Nil, Value::Nil) => true,
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Number(l), Value::Number(r)) => l == r,
            (Value::Bool(l), Value::Bool(r)) => l == r,
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
    #[test]
    fn for_loop() {
        let lox = "
for (var i = 0; i < 10; i = i + 1) {
    print i;
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
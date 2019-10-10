use super::{
    error::Error,
    expr::{Expr, ExprVisitor, Literal},
    env::{Env},
    func::Func,
    stmt::{Stmt, StmtVisitor},
    token::{Token, TokenType},
    value::Value,
};
use std::{
    collections::HashMap,
    rc::Rc,
};

use log::trace;

pub struct Interpreter {
    pub env: Env,
    pub closures: HashMap<String, Env>,
    pub recur: usize,
}

type IntResult = Result<Value, Error>;

impl ExprVisitor<Value> for Interpreter {
    fn visit_bin(&mut self, left: &Expr, op: &Token, right: &Expr) -> IntResult {
        trace!("visit_bin {:?}. {:?} {:?}", left, op.lexeme, right);
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
        trace!("visit_group {:?}", group);
        if let Expr::Grouping(inner) = group {
            self.evaluate(inner)
        } else {
            Err(Error::Runtime("Visited group unexpectedly".into()))
        }
    }

    fn visit_lit(&self, lit: &Literal) -> IntResult {
        trace!("visit_lit {}", lit);
        Ok(lit.clone_into())
    }

    fn visit_un(&mut self, op: &Token, ex: &Expr) -> IntResult {
        trace!("visit_unary {:?} {:?}", op.lexeme, ex);
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
        trace!("visit_var {}", name);
        self.env.get(name)
    }

    fn visit_assign(&mut self, name: &str, expr: &Expr) -> IntResult {
        trace!("visit_assign {:?} {:?}", name, expr);
        let val = self.evaluate(expr)?;
        self.env.assign(name, val)
    }

    fn visit_log(&mut self, left: &Expr, op: &Token, right: &Expr) -> IntResult {
        trace!("visit_log {:?} {:?} {:?}", left, op.lexeme, right);
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
        trace!("visit_call {:?} {:?}", callee, arguments);
        let callee = self.evaluate(callee)?;
        let args = arguments.iter().map(|e| self.evaluate(e)).collect::<Result<Vec<Value>, Error>>()?;
        if let Value::Func(ref f) = callee {
            if f.arity() != args.len() {
                return Err(Error::Runtime(format!("{} was expecting {} arguments but {} were provided", f.name(), f.arity(), args.len())))
            }
            f.call(self, &args)
        } else {
            Err(Error::Runtime(format!("Attempt to call a something that is not a function {}", callee)))
        }
    }
}

impl StmtVisitor<Option<Value>> for Interpreter {
    fn visit_expr_stmt(&mut self, expr: &Expr) -> Result<Option<Value>, Error> {
        trace!("visit_expr_stmt {:?}", expr);
        self.evaluate(expr)?;
        Ok(None)
    }

    fn visit_print_stmt(&mut self, expr: &Expr) -> Result<Option<Value>, Error> {
        trace!("visit_expr_stmt {:?}", expr);
        println!("{}", self.evaluate(expr)?);
        Ok(None)
    }

    fn visit_var_stmt(&mut self, name: String, expr: Option<Expr>) -> Result<Option<Value>, Error> {
        trace!("visit_var_stmt {:?} {:?}", name, expr);
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
        trace!("visit_block_stmt {:?}", list);
        self.execute_block(list)?;
        Ok(None)
    }
    
    fn visit_if_stmt(
        &mut self,
        test: &Expr,
        cons: &Stmt,
        alt: &Option<Box<Stmt>>,
    ) -> Result<Option<Value>, Error> {
        trace!("visit_if_stmt {:?} {:?} {:?}", test, cons, alt);
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
        trace!("visit_while_stmt {:?} {:?}", test, body);
        while Self::is_truthy(&self.evaluate(test)?) {
            match self.interpret(body) {
                Ok(Some(val)) => return Ok(Some(val)),
                Err(e) => return Err(e),
                _ => continue,
            }
        }
        Ok(None)
    }

    fn visit_func_decl(&mut self, name: &str, params: &[String], body: &[Stmt]) -> Result<Option<Value>, Error> {
        trace!("visit_func_decl {:?} {:?} {:?}", name, params, body);
        let func = Func {
            name: name.to_string(),
            params: params.to_vec(),
            body: body.to_vec(),
        };
        self.env.define(name.to_string(), Some(Value::Func(Rc::new(func))));
        let closure = self.env.clone();
        self.closures.insert(name.to_string(), closure);
        Ok(None)
    }

    fn visit_return_stmt(&mut self, expr: &Option<Expr>) -> Result<Option<Value>, Error> {
        trace!("visit_return_stmt {:?}", expr);
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
            closures: HashMap::new(),
            recur: 0,
        }
    }

    pub fn interpret(&mut self, stmt: &Stmt) -> Result<Option<Value>, Error> {
        self.recur += 1;
        let ret = stmt.accept(self);
        trace!("{} completing interpret {:?}", self.recur, ret);
        self.recur = self.recur.saturating_sub(1);
        ret
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

    #[test]
    fn func_if() {
        let _ = pretty_env_logger::try_init();
        let lox = "
fun isEven(n) {
    if (mod(n, 2) == 0) {
        return true;
    }
    return false;
}
var pre1 = mod(1, 2) == 0;
var test1 = isEven(1);
var pre2 = mod(2, 2) == 0;
var test2 = isEven(2);
";
        let mut int = Interpreter::new();
        let mut parser = crate::parser::Parser::new(
            crate::scanner::Scanner::new(lox.into()).unwrap()
        );
        while let Some(stmt) = parser.next() {
            int.interpret(&stmt.unwrap()).unwrap();
        }
        let test1 = int.env.get("test1").expect("Unable to get test1");
        let pre1 = int.env.get("pre1").expect("Unable to get pre1");
        assert_eq!(test1, pre1);
        let test2 = int.env.get("test2").expect("Unable to get test");
        let pre2 = int.env.get("pre2").expect("Unable to get pre2");
        assert_eq!(test2, pre2);
    }
}
use super::{
    callable::Callable,
    class::Class,
    env::Env,
    error::Error,
    expr::{Expr, ExprVisitor, Literal},
    func::Func,
    stmt::{Function, Stmt, StmtVisitor},
    token::{Token, TokenType},
    value::Value,
};

use log::trace;

pub struct Interpreter {
    pub env: Env,
    pub closures: Vec<Env>,
    pub recur: usize,
    pub current_depth: usize,
}

type IntResult = Result<Value, Error>;

impl ExprVisitor<Value> for Interpreter {
    fn visit_bin(&mut self, left: &Expr, op: &Token, right: &Expr) -> IntResult {
        trace!("visit_bin {:?}. {:?} {:?}", left, op.lexeme, right);
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;
        let ret = match (&op.kind, &left, &right) {
            (TokenType::Minus, Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs - rhs),
            (TokenType::Slash, Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs / rhs),
            (TokenType::Star, Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs * rhs),
            (TokenType::Plus, Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs + rhs),
            (TokenType::Greater, Value::Number(lhs), Value::Number(rhs)) => Value::Bool(lhs > rhs),
            (TokenType::GreaterEqual, Value::Number(lhs), Value::Number(rhs)) => {
                Value::Bool(lhs >= rhs)
            }
            (TokenType::Less, Value::Number(lhs), Value::Number(rhs)) => Value::Bool(lhs < rhs),
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
        trace!("visit_var {}, {}", name, self.current_depth);
        self.env.get(name, self.env.depth)
    }

    fn visit_assign(&mut self, name: &str, expr: &Expr) -> IntResult {
        trace!("visit_assign {:?} {:?}", name, expr);
        let mut val = self.evaluate(expr)?;
        if let Value::Class(ref mut inst) = val {
            for (_, method) in inst.methods.iter_mut() {
                method.this_name = name.to_string()
            }
        }
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
        let old_depth = self.current_depth;
        let callee = self.evaluate(callee)?;
        let args = arguments
            .iter()
            .map(|e| self.evaluate(e))
            .collect::<Result<Vec<Value>, Error>>()?;
        let ret = match callee {
            Value::Func(ref c) => self.handle_callable(c, &args),
            Value::Init(ref c) => self.handle_callable(c, &args),
            Value::NativeFunc(ref c) => self.handle_callable(c, &args),
            Value::Method(ref m) => self.handle_callable(m, &args),
            _ => Err(Error::Runtime(format!(
                "Attempt to call a something that is not a function {}",
                callee
            ))),
        };
        self.current_depth = old_depth;
        ret
    }

    fn visit_get(&mut self, object: &Expr, name: &str) -> IntResult {
        trace!("visit_get {:?} {:?}", object, name);
        if let Value::Class(class) = self.evaluate(object)? {
            class.get(name)
        } else {
            Err(Error::Runtime(format!(
                "cannot find property {} on {:?}",
                name, object
            )))
        }
    }
    fn visit_set(&mut self, object: &Expr, name: &str, value: &Expr) -> IntResult {
        trace!("visit_set {:?} {:?} {:?}", object, name, value);
        let value = self.evaluate(value)?;
        if let Value::Class(inst) = self.evaluate_mut(object)? {
            inst.set(name, value.clone());
        }

        Ok(value)
    }
    fn visit_this(&mut self) -> IntResult {
        trace!("visit_this");
        self.env.get("this", self.current_depth)
    }
}

impl StmtVisitor<()> for Interpreter {
    fn visit_expr_stmt(&mut self, expr: &Expr) -> Result<(), Error> {
        trace!("visit_expr_stmt {:?}", expr);
        self.evaluate(expr)?;
        Ok(())
    }

    fn visit_print_stmt(&mut self, expr: &Expr) -> Result<(), Error> {
        trace!("visit_expr_stmt {:?}", expr);
        println!("{}", self.evaluate(expr)?);
        Ok(())
    }

    fn visit_var_stmt(&mut self, name: String, expr: Option<Expr>) -> Result<(), Error> {
        trace!("visit_var_stmt {:?} {:?}", name, expr);
        let value = if let Some(expr) = expr {
            let mut val = match expr.accept(self) {
                Ok(val) | Err(Error::Return(val)) => val,
                Err(e) => return Err(e),
            };
            trace!("defining {} with {}", name, val);
            if let Value::Class(ref mut inst) = val {
                for (_, meth) in inst.methods.iter_mut() {
                    meth.this_name = name.to_string()
                }
            }
            Some(val)
        } else {
            None
        };
        self.env.define(name, value);
        Ok(())
    }

    fn visit_block_stmt(&mut self, list: &[Stmt]) -> Result<(), Error> {
        trace!("visit_block_stmt {:?}", list);
        self.execute_block(list)?;
        Ok(())
    }

    fn visit_if_stmt(
        &mut self,
        test: &Expr,
        cons: &Stmt,
        alt: &Option<Box<Stmt>>,
    ) -> Result<(), Error> {
        trace!("visit_if_stmt {:?} {:?} {:?}", test, cons, alt);
        let boolean = self.evaluate(test)?;
        if Self::is_truthy(&boolean) {
            self.interpret(cons)?;
        } else if let Some(alt) = alt {
            self.interpret(alt)?;
        }
        Ok(())
    }

    fn visit_while_stmt(&mut self, test: &Expr, body: &Stmt) -> Result<(), Error> {
        trace!("visit_while_stmt {:?} {:?}", test, body);
        while Self::is_truthy(&self.evaluate(test)?) {
            self.interpret(body)?;
        }
        Ok(())
    }

    fn visit_func_decl(
        &mut self,
        name: &str,
        params: &[String],
        body: &[Stmt],
    ) -> Result<(), Error> {
        trace!("visit_func_decl {:?} {:?} {:?}", name, params, body);
        let func = Func {
            name: name.to_string(),
            params: params.to_vec(),
            body: body.to_vec(),
            env_id: self.env.depth + 1,
        };
        self.env.define(name.to_string(), Some(Value::Func(func)));
        let closure = self.env.clone();
        self.closures.push(closure);
        Ok(())
    }

    fn visit_return_stmt(&mut self, expr: &Option<Expr>) -> Result<(), Error> {
        trace!("visit_return_stmt {:?}", expr);
        let ret = if let Some(expr) = expr {
            self.evaluate(expr)?
        } else {
            Value::Nil
        };
        Err(Error::Return(ret))
    }
    fn visit_class(&mut self, name: &str, methods: &[Function]) -> Result<(), Error> {
        trace!("visit_return_stmt {} {:?}", name, methods.len());
        self.env.define(name.to_string(), None);
        let class = Class {
            name: name.to_string(),
            methods: methods.to_vec(),
        };
        let value = Value::Init(class);
        self.env.assign(name, value)?;
        Ok(())
    }
}
impl Default for Interpreter {
    fn default() -> Self {
        let env = Env::root();
        Self {
            current_depth: env.depth,
            env,
            closures: Vec::new(),
            recur: 0,
        }
    }
}
impl Interpreter {
    pub fn new() -> Self {
        let env = Env::root();
        Self {
            current_depth: env.depth,
            env,
            closures: Vec::new(),
            recur: 0,
        }
    }

    pub fn interpret(&mut self, stmt: &Stmt) -> Result<(), Error> {
        trace!("interpret: {:?}", stmt);
        self.recur += 1;
        let ret = stmt.accept(self);
        trace!("{} completing interpret {:?}", self.recur, ret);
        self.recur = self.recur.saturating_sub(1);
        ret?;
        Ok(())
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<Value, Error> {
        expr.accept(self)
    }

    pub fn evaluate_mut(&mut self, expr: &Expr) -> Result<&mut Value, Error> {
        match expr {
            Expr::Var(name) => self.env.get_mut(name, self.current_depth),
            Expr::Get { object, name } => {
                if let Value::Class(inst) = self.evaluate_mut(object)? {
                    inst.get_mut(name)
                } else {
                    Err(Error::Runtime(
                        "Invalid left hand side of assignment".to_string(),
                    ))
                }
            }
            Expr::This => self.env.get_mut("this", self.current_depth),
            _ => Err(Error::Runtime(
                "Invalid left hand side of assignment".to_string(),
            )),
        }
    }

    pub fn execute_block(&mut self, stmts: &[Stmt]) -> Result<(), Error> {
        let old_depth = self.current_depth;
        self.env.descend();
        self.current_depth = self.env.depth;
        for stmt in stmts {
            if let Err(e) = self.interpret(stmt) {
                self.env.ascend();
                return Err(e);
            }
        }
        self.env.ascend();
        self.current_depth = old_depth;
        Ok(())
    }

    fn handle_callable<T>(&mut self, f: &T, arguments: &[Value]) -> Result<Value, Error>
    where
        T: Callable + ?Sized,
    {
        if f.arity() != arguments.len() {
            return Err(Error::Runtime(format!(
                "{} was expecting {} arguments but {} were provided",
                f,
                f.arity(),
                arguments.len()
            )));
        }

        match f.call(self, arguments) {
            Ok(val) => Ok(val),
            Err(Error::Return(ret)) => Ok(ret),
            Err(e) => Err(e),
        }
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
            (Value::Number(l), Value::Number(r)) => l.eq(r),
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
        let mut parser =
            crate::parser::Parser::new(crate::scanner::Scanner::new(lox.into()).unwrap());
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
        let mut parser =
            crate::parser::Parser::new(crate::scanner::Scanner::new(lox.into()).unwrap());
        while let Some(stmt) = parser.next() {
            int.interpret(&stmt.unwrap()).unwrap();
        }
    }

    #[test]
    fn this() {
        let _ = pretty_env_logger::try_init();
        let lox = r#"
class Junk {
  stuff() {
    print "stuff";
  }
  things(one) {
    this.one = one;
  }
}
var junk = Junk();
junk.things("hahah");
print junk.one;
"#;
        let mut int = Interpreter::new();
        let mut parser =
            crate::parser::Parser::new(crate::scanner::Scanner::new(lox.into()).unwrap());
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
        let mut parser =
            crate::parser::Parser::new(crate::scanner::Scanner::new(lox.into()).unwrap());
        while let Some(stmt) = parser.next() {
            int.interpret(&stmt.unwrap()).unwrap();
        }
        let test1 = int.env.get("test1", 1).expect("Unable to get test1");
        let pre1 = int.env.get("pre1", 1).expect("Unable to get pre1");
        assert_eq!(test1, pre1);
        let test2 = int.env.get("test2", 1).expect("Unable to get test");
        let pre2 = int.env.get("pre2", 1).expect("Unable to get pre2");
        assert_eq!(test2, pre2);
    }
}

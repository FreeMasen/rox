use crate::{
    error::{Error},
    expr::{Expr, ExprVisitor, Literal, ScopedExpr},
    interpreter::Interpreter,
    parser::Parser,
    stmt::{Stmt, StmtVisitor, ScopedStmt},
    token::Token,
};
use std::{cell::Cell,
        collections::HashMap,
};
use log::trace;

#[derive(Clone, Copy)]
enum FuncType {
    None,
    Func,
    Init,
    Method
}
pub struct Resolver {
    pub scopes: Vec<HashMap<String, bool>>,
    current_func: FuncType,
}

impl StmtVisitor<()> for Resolver {
    fn visit_print_stmt(&mut self, expr: &Expr) -> Result<(), Error> {
        trace!("Resolver::visit_expr_stmt {:?}", expr);
        self.resolve_expr(expr)?;
        Ok(())
    }
    fn visit_expr_stmt(&mut self, expr: &Expr) -> Result<(), Error> {
        trace!("Resolver::visit_expr_stmt {:?}", expr);
        self.resolve_expr(expr)?;
        Ok(())
    }
    fn visit_var_stmt(&mut self, name: &str, expr: &Option<Expr>) -> Result<(), Error> {
        trace!("Resolver::visit_var_stmt {:?} {:?}", name, expr);
        self.declare(name)?;
        if let Some(expr) = expr {
            self.resolve_expr(expr)?;
        }
        Ok(())
    }
    fn visit_block_stmt(&mut self, list: &[Stmt]) -> Result<(), Error> {
        trace!("Resolver::visit_block_stmt {:?}", list);
        self.resolve_stmt_list(list)?;
        Ok(())
    }
    fn visit_if_stmt(
        &mut self,
        test: &Expr,
        cons: &Stmt,
        alt: &Option<Box<Stmt>>,
    ) -> Result<(), Error> {
        trace!("Resolver::visit_if_stmt {:?} {:?} {:?}", test, cons, alt);
        self.resolve_expr(test)?;
        self.resolve_stmt(cons)?;
        if let Some(alt) = alt {
            self.resolve_stmt(alt)?;
        }
        Ok(())
    }
    fn visit_while_stmt(&mut self, test: &Expr, body: &Stmt) -> Result<(), Error> {
        trace!("Resolver::visit_while_stmt {:?} {:?}", test, body);
        self.resolve_expr(test)?;
        self.resolve_stmt(body)?;
        Ok(())
    }
    fn visit_func_decl(&mut self, name: &str, params: &[String], body: &[Stmt]) -> Result<(), Error> {
        trace!("Resolver::visit_func_decl {:?} {:?} {:?}", name, params, body);
        self.declare(name)?;
        self.define(name);
        self.resolve_func(params, body, FuncType::Func)?;
        Ok(())
    }
    fn visit_return_stmt(&mut self, expr: &Option<Expr>) -> Result<(), Error> {
        trace!("Resolver::visit_return_stmt {:?}", expr);
        if let FuncType::None = self.current_func {
            return Err(Error::Parser(format!("cannot return from outside of a function or method")));
        }
        if let Some(expr) = expr {
            if let FuncType::Init = self.current_func {
                return Err(Error::Parser(format!("Cannot return a value from an initializer")));
            }
            self.resolve_expr(expr)?;
        }
        Ok(())
    }
}

impl ExprVisitor<()> for Resolver {
    fn visit_bin(&mut self, left: &Expr, _: &Token, right: &Expr) -> Result<(), Error> {
        trace!("Resolver::visit_bin {:?}  {:?}", left, right);
        self.resolve_expr(left)?;
        self.resolve_expr(right)?;
        Ok(())
    }
    fn visit_group(&mut self, group: &Expr) -> Result<(), Error> {
        trace!("Resolver::visit_group {:?}", group);
        self.resolve_expr(group)?;
        Ok(())
    }
    fn visit_lit(&self, _: &Literal) -> Result<(), Error> {
        trace!("Resolver::visit_lit");
        Ok(())
    }
    fn visit_un(&mut self, _: &Token, ex: &Expr) -> Result<(), Error> {
        trace!("Resolver::visit_unary {:?}", ex);
        self.resolve_expr(ex)?;
        Ok(())
    }
    fn visit_var(&mut self, name: &str) -> Result<(), Error> {
        trace!("Resolver::visit_var {}", name);
        if let Some(scope) = self.scopes.last() {
            if let Some(entry) = scope.get(name) {
                if entry == &false {
                    return Err(Error::Parser(format!("Cannot read local variable in its own initializer ({})", name)));
                }
            }
        }
        Ok(())
    }
    fn visit_assign(&mut self, name: &str, value: &Expr, scope: &Cell<Option<usize>>) -> Result<(), Error> {
        trace!("Resolver::visit_assign {:?} {:?} {:?}", name, value, scope);
        self.resolve_expr(value)?;
        scope.set(self.resolve_local(name));
        Ok(())
    }
    fn visit_log(&mut self, left: &Expr, _: &Token, right: &Expr) -> Result<(), Error> {
        trace!("Resolver::visit_log {:?} {:?}", left, right);
        self.resolve_expr(left)?;
        self.resolve_expr(right)?;
        Ok(())
    }
    fn visit_call(&mut self, callee: &Expr, arguments: &[Expr]) -> Result<(), Error> {
        trace!("Resolver::visit_call {:?} {:?}", callee, arguments);
        self.resolve_expr(callee)?;
        for arg in arguments {
            self.resolve_expr(arg)?;
        }
        Ok(())
    }
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            current_func: FuncType::None,
            scopes: Vec::new(),
        }
    }

    pub fn resolve_stmt_list(&mut self, stmts: &[Stmt]) -> Result<(), Error> {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }
    pub fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<(), Error> {
        stmt.accept(self)?;
        Ok(())
    }
    pub fn resolve_expr(&mut self, expr: &Expr) -> Result<(), Error> {
        expr.accept(self)?;
        Ok(())
    }
    fn resolve_func(&mut self, params: &[String], body: &[Stmt], ty: FuncType) -> Result<(), Error> {
        let enclosing = self.current_func;
        self.current_func = ty;
        self.begin_scope();
        for param in params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_stmt_list(body)?;
        self.end_scope();
        self.current_func = enclosing;
        Ok(())
    }
    pub fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    pub fn end_scope(&mut self) {
        self.scopes.pop();
    }
    pub fn declare(&mut self, name: &str) -> Result<(), Error> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(name) {
                return Err(Error::Parser(format!("{} has already been declared in this scope", name)));
            } else {
                scope.insert(name.to_string(), false);
            }
        }
        
        Ok(())
    }

    pub fn define(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            if let Some(entry) = scope.get_mut(name) {
                *entry = true;
            }
        }
    }

    pub fn resolve_local(&mut self, name: &str) -> Option<usize> {
        let scope_len = self.scopes.len().saturating_sub(1);
        for (i, scope) in self.scopes.iter_mut().enumerate().rev() {
            if scope.contains_key(name) {
                return Some(scope_len - i);
            }
        }
        return None;
    }
}
use super::{error::Error, expr::Expr};
#[derive(Debug, Clone)]
pub enum Stmt {
    Print(Expr),
    Expr(Expr),
    Var {
        name: String,
        value: Option<Expr>,
    },
    Block(Vec<Stmt>),
    If {
        test: Expr,
        consequence: Box<Stmt>,
        alternate: Option<Box<Stmt>>,
    },
    While {
        test: Expr,
        body: Box<Stmt>,
    },
    Func(Function),
    Return(Option<Expr>),
    Class {
        name: String,
        methods: Vec<Function>,
    },
}
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}

impl Stmt {
    pub fn accept<T>(&mut self, visitor: &mut impl StmtVisitor<T>) -> Result<T, Error> {
        match self {
            Stmt::Print(inner) => visitor.visit_print_stmt(inner),
            Stmt::Expr(inner) => visitor.visit_expr_stmt(inner),
            Stmt::Var { name, value } => visitor.visit_var_stmt(name, value),
            Stmt::Block(list) => visitor.visit_block_stmt(list),
            Stmt::If {
                test,
                consequence,
                alternate,
            } => visitor.visit_if_stmt(test, consequence, alternate),
            Stmt::While { test, body } => visitor.visit_while_stmt(test, body),
            Stmt::Func(Function { name, params, body }) => {
                visitor.visit_func_decl(name, params, body)
            }
            Stmt::Return(expr) => visitor.visit_return_stmt(expr),
            Stmt::Class { name, methods } => visitor.visit_class(name, methods),
        }
    }
}

pub trait StmtVisitor<T> {
    fn visit_print_stmt(&mut self, expr: &mut Expr) -> Result<T, Error>;
    fn visit_expr_stmt(&mut self, expr: &mut Expr) -> Result<T, Error>;
    fn visit_var_stmt(&mut self, name: &str, expr: &mut Option<Expr>) -> Result<T, Error>;
    fn visit_block_stmt(&mut self, list: &mut [Stmt]) -> Result<T, Error>;
    fn visit_if_stmt(
        &mut self,
        test: &mut Expr,
        cons: &mut Stmt,
        alt: &mut Option<Box<Stmt>>,
    ) -> Result<T, Error>;
    fn visit_while_stmt(&mut self, test: &mut Expr, body: &mut Stmt) -> Result<T, Error>;
    fn visit_func_decl(&mut self, name: &str, params: &[String], body: &[Stmt])
        -> Result<T, Error>;
    fn visit_return_stmt(&mut self, expr: &mut Option<Expr>) -> Result<T, Error>;
    fn visit_class(&mut self, name: &str, methods: &mut [Function]) -> Result<T, Error>;
}

use super::{
    error::Error,
    expr::{
        Expr,
    }
};

pub enum Stmt {
    Print(Expr),
    Expr(Expr)
}

impl Stmt {
    pub fn accept<T>(&self, visitor: &mut impl StmtVisitor<T>) -> Result<T, Error> {
        match self {
            Stmt::Print(inner) => visitor.visit_print_stmt(inner),
            Stmt::Expr(inner) => visitor.visit_expr_stmt(inner),
        }
    }
}

pub trait StmtVisitor<T> {
    fn visit_print_stmt(&mut self, expr: &Expr) -> Result<T, Error>;
    fn visit_expr_stmt(&mut self, expr: &Expr) -> Result<T, Error>;
}
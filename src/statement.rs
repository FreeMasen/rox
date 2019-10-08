use super::{error::Error, expr::Expr};

pub enum Stmt {
    Print(Expr),
    Expr(Expr),
    Var { name: String, value: Option<Expr> },
}

impl Stmt {
    pub fn accept<T>(&self, visitor: &mut impl StmtVisitor<T>) -> Result<T, Error> {
        match self {
            Stmt::Print(inner) => visitor.visit_print_stmt(inner),
            Stmt::Expr(inner) => visitor.visit_expr_stmt(inner),
            Stmt::Var { name, value } => visitor.visit_var_stmt(name.clone(), value.clone()),
        }
    }
}

pub trait StmtVisitor<T> {
    fn visit_print_stmt(&mut self, expr: &Expr) -> Result<T, Error>;
    fn visit_expr_stmt(&mut self, expr: &Expr) -> Result<T, Error>;
    fn visit_var_stmt(&mut self, name: String, expr: Option<Expr>) -> Result<T, Error>;
}

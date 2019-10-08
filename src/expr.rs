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
}
#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Number(f64),
    Bool(bool),
    Ident(String),
    Nil,
}

impl ::std::fmt::Display for Literal {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Literal::String(s) => write!(f, "\"{}\"", s),
            Literal::Number(n) => n.fmt(f),
            Literal::Bool(b) => b.fmt(f),
            Literal::Ident(s) => s.fmt(f),
            Literal::Nil => write!(f, "nil"),
        }
    }
}

impl Expr {
    pub fn accept<T>(&self, visitor: &impl ExprVisitor<T>) -> T {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => visitor.visit_bin(left, operator, right),
            Expr::Grouping(_) => visitor.visit_group(self),
            Expr::Literal(lit) => visitor.visit_lit(lit),
            Expr::Unary { operator, right } => visitor.visit_un(operator, right),
            Expr::Var(name) => visitor.visit_ident(name),
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
}

pub trait ExprVisitor<T> {
    fn visit_bin(&self, left: &Expr, op: &Token, right: &Expr) -> T;
    fn visit_group(&self, group: &Expr) -> T;
    fn visit_lit(&self, lit: &Literal) -> T;
    fn visit_un(&self, op: &Token, ex: &Expr) -> T;
    fn visit_ident(&self, name: &str) -> T;
}

pub struct ExprPrinter;
impl ExprVisitor<String> for ExprPrinter {
    fn visit_bin(&self, left: &Expr, op: &Token, right: &Expr) -> String {
        self.parenthesize(&op.lexeme, &[left, right])
    }
    fn visit_group(&self, group: &Expr) -> String {
        self.parenthesize("group", &[group])
    }
    fn visit_lit(&self, lit: &Literal) -> String {
        format!("{}", lit)
    }
    fn visit_un(&self, op: &Token, ex: &Expr) -> String {
        self.parenthesize(&op.lexeme, &[ex])
    }
    fn visit_ident(&self, name: &str) -> String {
        name.to_string()
    }
}

impl ExprPrinter {
    pub fn print(&self, ex: &Expr) -> String {
        ex.accept(self)
    }
    pub fn parenthesize(&self, name: &str, exprs: &[&Expr]) -> String {
        let mut ret = String::new();
        ret.push('(');
        ret.push_str(name);
        for ex in exprs {
            ret.push(' ');
            ret.push_str(&ex.accept(self))
        }
        ret.push(')');
        ret
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::token::TokenType;
    #[test]
    fn test_pretty_printer() {
        let pp = ExprPrinter;
        let expr = Expr::binary(
            Expr::unary(
                Token::new(TokenType::Minus, "-".to_string(), 1),
                Expr::Literal(Literal::Number(123.0)),
            ),
            Expr::grouping(Expr::Literal(Literal::Number(45.67))),
            Token::new(TokenType::Star, "*".to_string(), 1),
        );
        let expectation = "(* (- 123) (group 45.67))".to_string();
        let result = pp.print(&expr);
        eprintln!("result: {}", result);
        assert_eq!(expectation, result);
    }
}

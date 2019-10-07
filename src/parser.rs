use super::token::{Token, TokenType};
use super::expr::{Expr, Literal};
use super::Scanner;
use super::SimpleResult;
use super::error::Error;
use super::statement::Stmt;

type ParserItem = Result<Stmt, Error>;

pub struct Parser {
    pub scanner: Scanner,
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(scanner: Scanner) -> Self {
        Self {
            scanner: scanner,
            tokens: vec![],
        }
    }

    pub fn line(&self) -> usize {
        self.scanner.line
    }

    pub fn statement(&mut self) -> SimpleResult<Stmt> {
        if self.at(TokenType::Print)? {
            self.print_stmt()
        } else {
            self.expression_stmt()
        }
    }

    pub fn print_stmt(&mut self) -> SimpleResult<Stmt> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Print statments must end with a semi-colon")?;
        Ok(Stmt::Print(value))
    }

    pub fn expression_stmt(&mut self) -> SimpleResult<Stmt> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected semi-colon after expression")?;
        Ok(Stmt::Expr(value))
    }
    
    pub fn expression(&mut self) -> SimpleResult<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> SimpleResult<Expr> {
        let mut expr = self.comparison()?;
        while self.at(TokenType::BangEqual)? || self.at(TokenType::EqualEqual)? {
            let op = self.previous()?;
            let right = self.comparison()?;
            expr = Expr::binary(expr, right, op);
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> SimpleResult<Expr> {
        let mut expr = self.addition()?;
        while self.at(TokenType::Greater)? || self.at(TokenType::GreaterEqual)?
        || self.at(TokenType::Less)? || self.at(TokenType::LessEqual)? {
            let op = self.previous()?;
            let right = self.addition()?;
            expr = Expr::binary(expr, right, op);
        }
        Ok(expr)
    }
    fn addition(&mut self) -> SimpleResult<Expr> {
        let mut expr = self.multiplication()?;
        while self.at(TokenType::Minus)? || self.at(TokenType::Plus)? {
            let op = self.previous()?;
            let right = self.multiplication()?;
            expr = Expr::binary(expr, right, op);
        }
        Ok(expr)
    }
    fn multiplication(&mut self) -> SimpleResult<Expr> {
        let mut expr = self.unary()?;
        while self.at(TokenType::Slash)? || self.at(TokenType::Star)? {
            let op = self.previous()?;
            let right = self.unary()?;
            expr = Expr::binary(expr, right, op);
        }
        Ok(expr)
    }
    fn unary(&mut self) -> SimpleResult<Expr> {
        if self.at(TokenType::Bang)? || self.at(TokenType::Minus)? {
            let op = self.previous()?;
            let right = self.unary()?;
            Ok(Expr::unary(op, right))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> SimpleResult<Expr> {
        Ok(if self.at(TokenType::False)? {
            Expr::Literal(
                self.previous_literal()?
            )
        } else if self.at(TokenType::True)? {
            Expr::Literal(
                self.previous_literal()?
            )
        } else if self.at(TokenType::Nil)? {
            Expr::Literal(
                self.previous_literal()?
            )
        } else if self.at_literal()? {
            Expr::Literal(
                self.previous_literal()?
            )
        } else if self.at(TokenType::LeftParen)? {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression")?;
            Expr::grouping(expr)
        } else {
            return Err(
                Error::Parser(format!("Unexpected expression: {:?}", self.scanner.lookahead()))
            )
        })
    }

    fn previous_literal(&mut self) -> SimpleResult<Literal> {
        Ok(match self.previous()?.kind {
            TokenType::String(s) => Literal::String(s),
            TokenType::Number(n) => Literal::Number(n),
            TokenType::True => Literal::Bool(true),
            TokenType::False => Literal::Bool(false),
            TokenType::Nil => Literal::Nil,
            _ => return Err(Error::Parser("expected literal".into())),
        })
    }
    fn previous(&mut self) -> Result<Token, Error> {
        if let Some(tok) = self.tokens.last() {
            Ok(tok.clone())
        } else {
            Err(Error::Parser("Attempt to get last token when none was found".into()))
        }
    }
    fn at_literal(&mut self) -> Result<bool, Error> {
        if let Some(tok) = self.scanner.lookahead() {
            match tok.kind {
                TokenType::Number(_) 
                | TokenType::String(_) => {
                    self.advance()?;
                    Ok(true)
                },
                _ => Ok(false),
            }
        } else {
            Ok(false)
        }
    }
    fn at(&mut self, ty: TokenType) -> Result<bool, Error> {
        if self.check(ty) {
            self.advance()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn advance(&mut self) -> Result<(), Error> {
        if !self.is_at_end() {
            if let Some(res) = self.scanner.next() {
                self.tokens.push(res?)
            }
        }
        Ok(())
    }

    fn check(&self, ty: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.scanner.lookahead_matches(ty)
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.scanner.done()
    }

    fn consume(&mut self, tok: TokenType, msg: &str) -> SimpleResult<()> {
        if self.check(tok) {
            self.advance()?;
            Ok(())
        } else {
            Err(Error::Parser(msg.to_string()))
        }
    }

    pub fn sync(&mut self) {
        let _ = self.advance();
        while !self.is_at_end() {
            if let Ok(prev) = self.previous() {
                if prev.kind == TokenType::Semicolon {
                    break;
                }
            }
            if self.scanner.lookahead_matches(TokenType::Class )
                || self.scanner.lookahead_matches(TokenType::Fun)
                || self.scanner.lookahead_matches(TokenType::Var)
                || self.scanner.lookahead_matches(TokenType::For)
                || self.scanner.lookahead_matches(TokenType::If)
                || self.scanner.lookahead_matches(TokenType::While)
                || self.scanner.lookahead_matches(TokenType::Print)
                || self.scanner.lookahead_matches(TokenType::Return) {
                    break;
            }
            let _ = self.advance();
        }
    }
}

impl Iterator for Parser {
    type Item = ParserItem;
    fn next(&mut self) -> Option<Self::Item> {
        if self.is_at_end() {
            None
        } else {
            Some(self.statement())
        }
    }
}
use super::error::Error;
use super::expr::{Expr, Literal};
use super::stmt::{Function, Stmt};
use super::Scanner;
use super::SimpleResult;
use rox_shared::{Token, TokenType};

type ParserItem = Result<Stmt, Error>;

pub struct Parser {
    pub scanner: Scanner,
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(scanner: Scanner) -> Self {
        Self {
            scanner,
            tokens: vec![],
        }
    }

    pub fn line(&self) -> usize {
        self.scanner.line
    }

    pub fn decl(&mut self) -> SimpleResult<Stmt> {
        if self.at(TokenType::Var)? {
            self.var_decl()
        } else if self.at(TokenType::Fun)? {
            self.fun_decl("function")
        } else if self.at(TokenType::Class)? {
            self.class_decl()
        } else {
            self.statement()
        }
    }

    pub fn statement(&mut self) -> SimpleResult<Stmt> {
        if self.at(TokenType::Print)? {
            self.print_stmt()
        } else if self.at(TokenType::Return)? {
            self.return_stmt()
        } else if self.at(TokenType::LeftBrace)? {
            self.block_stmt()
        } else if self.at(TokenType::If)? {
            self.if_stmt()
        } else if self.at(TokenType::While)? {
            self.while_stmt()
        } else if self.at(TokenType::For)? {
            self.for_stmt()
        } else {
            self.expression_stmt()
        }
    }

    pub fn var_decl(&mut self) -> SimpleResult<Stmt> {
        let name = self.expect_ident()?;
        let value = if self.at(TokenType::Equal)? {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        Ok(Stmt::Var { name, value })
    }

    pub fn fun_decl(&mut self, kind: &str) -> SimpleResult<Stmt> {
        let func = self.bare_func(kind)?;
        Ok(Stmt::Func(func))
    }

    pub fn bare_func(&mut self, kind: &str) -> SimpleResult<Function> {
        let name = self.expect_ident()?;
        self.consume(
            TokenType::LeftParen,
            &format!("Expected ( after {} identifier", kind),
        )?;
        let mut params = vec![];
        if !self.check(TokenType::RightParen) {
            params.push(self.expect_ident()?);
            while self.at(TokenType::Comma)? {
                params.push(self.expect_ident()?);
                if params.len() > 255 {
                    return Err(Error::Parser(format!(
                        "{} {:?} has too many parameters",
                        kind, name
                    )));
                }
            }
        }
        self.consume(
            TokenType::RightParen,
            &format!("Expected ) after {} arguments", kind),
        )?;
        self.consume(
            TokenType::LeftBrace,
            &format!("Expected {{ after {} arguments", kind),
        )?;
        let body = self.bare_block()?;
        Ok(Function { name, params, body })
    }

    pub fn class_decl(&mut self) -> SimpleResult<Stmt> {
        let ident = self.expect_ident()?;
        self.consume(
            TokenType::LeftBrace,
            &format!("Expected {{ after class name: {}", ident),
        )?;
        let mut methods = Vec::new();
        while !self.is_at_end() && !self.check(TokenType::RightBrace) {
            methods.push(self.bare_func("method")?);
        }
        self.consume(
            TokenType::RightBrace,
            &format!("Expected }} after class methods in {}", ident),
        )?;
        Ok(Stmt::Class {
            name: ident,
            methods,
        })
    }

    pub fn print_stmt(&mut self) -> SimpleResult<Stmt> {
        let value = self.expression()?;
        self.consume(
            TokenType::Semicolon,
            "Print statments must end with a semi-colon",
        )?;
        Ok(Stmt::Print(value))
    }

    pub fn return_stmt(&mut self) -> SimpleResult<Stmt> {
        let val = if !self.check(TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "Expected ; after return")?;
        Ok(Stmt::Return(val))
    }

    pub fn while_stmt(&mut self) -> SimpleResult<Stmt> {
        self.consume(TokenType::LeftParen, "'while' must be followed by a '('")?;
        let test = self.expression()?;
        self.consume(
            TokenType::RightParen,
            "'while ([expresssion] must be followed by a ')'",
        )?;
        let body = self.statement()?;
        Ok(Stmt::While {
            test,
            body: Box::new(body),
        })
    }

    pub fn for_stmt(&mut self) -> SimpleResult<Stmt> {
        self.consume(TokenType::LeftParen, "Expected '(' after if")?;
        let init = if self.at(TokenType::Semicolon)? {
            None
        } else if self.at(TokenType::Var)? {
            Some(self.var_decl()?)
        } else {
            Some(self.expression_stmt()?)
        };
        let cond = if !self.check(TokenType::Semicolon) {
            self.expression()?
        } else {
            Expr::Literal(Literal::Bool(true))
        };
        self.consume(TokenType::Semicolon, "Expected ';' after for loop test")?;
        let update = if !self.check(TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::RightParen, "Expected ')' after if (...")?;
        let body = self.statement()?;
        let mut block = vec![];
        if let Some(init) = init {
            block.push(init);
        }
        let mut w_body = vec![body];
        if let Some(expr) = update {
            w_body.push(Stmt::Expr(expr));
        }
        let w = Stmt::While {
            test: cond,
            body: Box::new(Stmt::Block(w_body)),
        };
        block.push(w);
        Ok(Stmt::Block(block))
    }
    pub fn block_stmt(&mut self) -> SimpleResult<Stmt> {
        Ok(Stmt::Block(self.bare_block()?))
    }

    fn bare_block(&mut self) -> SimpleResult<Vec<Stmt>> {
        let mut ret = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            ret.push(self.decl()?);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(ret)
    }

    pub fn if_stmt(&mut self) -> SimpleResult<Stmt> {
        self.consume(TokenType::LeftParen, "Expected ( after if")?;
        let cond = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ) after if ([expression]")?;
        let cons = self.statement()?;
        let alt = if self.at(TokenType::Else)? {
            let alt = self.statement()?;
            Some(Box::new(alt))
        } else {
            None
        };
        Ok(Stmt::If {
            test: cond,
            consequence: Box::new(cons),
            alternate: alt,
        })
    }

    pub fn expression_stmt(&mut self) -> SimpleResult<Stmt> {
        let value = self.expression()?;
        self.consume(
            TokenType::Semicolon,
            &format!("Expected semi-colon after expression {:?}", value),
        )?;
        Ok(Stmt::Expr(value))
    }

    pub fn expression(&mut self) -> SimpleResult<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> SimpleResult<Expr> {
        let expr = self.logical_or()?;
        if self.at(TokenType::Equal)? {
            let value = self.assignment()?;
            if let Expr::Var(name) = expr {
                Ok(Expr::assign(name, value))
            } else if let Expr::Get { object, name } = expr {
                Ok(Expr::Set {
                    object,
                    name,
                    value: Box::new(value),
                })
            } else {
                Err(Error::Parser(format!(
                    "Expected ident before equals found {:?}",
                    expr
                )))
            }
        } else {
            Ok(expr)
        }
    }

    fn logical_or(&mut self) -> SimpleResult<Expr> {
        let mut expr = self.logical_and()?;
        while self.at(TokenType::Or)? {
            let op = self.previous()?;
            let right = self.logical_and()?;
            expr = Expr::log(expr, right, op);
        }
        Ok(expr)
    }

    fn logical_and(&mut self) -> SimpleResult<Expr> {
        let mut expr = self.equality()?;
        while self.at(TokenType::And)? {
            let op = self.previous()?;
            let right = self.equality()?;
            expr = Expr::log(expr, right, op);
        }
        Ok(expr)
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
        while self.at(TokenType::Greater)?
            || self.at(TokenType::GreaterEqual)?
            || self.at(TokenType::Less)?
            || self.at(TokenType::LessEqual)?
        {
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
            self.call()
        }
    }

    fn call(&mut self) -> SimpleResult<Expr> {
        let mut expr = self.primary()?;
        loop {
            if self.at(TokenType::LeftParen)? {
                expr = self.finish_call(expr)?;
            } else if self.at(TokenType::Dot)? {
                let name = self.expect_ident()?;
                expr = Expr::Get {
                    object: Box::new(expr),
                    name,
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, expr: Expr) -> SimpleResult<Expr> {
        let mut args = vec![];
        if !self.check(TokenType::RightParen) {
            args.push(self.expression()?);
            while self.at(TokenType::Comma)? {
                args.push(self.expression()?);
            }
        }
        self.consume(TokenType::RightParen, "Expected ) at end of function call")?;
        Ok(Expr::Call {
            callee: Box::new(expr),
            arguments: args,
        })
    }

    fn primary(&mut self) -> SimpleResult<Expr> {
        Ok(
            if self.at(TokenType::False)?
                || self.at(TokenType::True)?
                || self.at(TokenType::Nil)?
                || self.at_literal()?
            {
                Expr::Literal(self.previous_literal()?)
            } else if self.at(TokenType::This)? {
                Expr::This
            } else if self.at_ident()? {
                Expr::Var(self.previous_ident()?)
            } else if self.at(TokenType::LeftParen)? {
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expect ')' after expression")?;
                Expr::grouping(expr)
            } else {
                return Err(Error::Parser(format!(
                    "Unexpected expression: {:?}",
                    self.scanner.lookahead()
                )));
            },
        )
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

    fn previous_ident(&mut self) -> SimpleResult<String> {
        if let TokenType::Identifier(value) = self.previous()?.kind {
            Ok(value)
        } else {
            Err(Error::Parser(format!(
                "Expected Identifier found {:?}",
                self.previous()
            )))
        }
    }

    fn previous(&mut self) -> Result<Token, Error> {
        if let Some(tok) = self.tokens.last() {
            Ok(tok.clone())
        } else {
            Err(Error::Parser(
                "Attempt to get last token when none was found".into(),
            ))
        }
    }

    fn at_literal(&mut self) -> Result<bool, Error> {
        if let Some(tok) = self.scanner.lookahead() {
            match tok.kind {
                TokenType::Number(_) | TokenType::String(_) => {
                    self.advance()?;
                    Ok(true)
                }
                _ => Ok(false),
            }
        } else {
            Ok(false)
        }
    }
    fn at_ident(&mut self) -> Result<bool, Error> {
        if let Some(tok) = self.scanner.lookahead() {
            if let TokenType::Identifier(_) = tok.kind {
                self.advance()?;
                Ok(true)
            } else {
                Ok(false)
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

    fn expect_ident(&mut self) -> Result<String, Error> {
        let name = if let Some(tok) = self.scanner.lookahead() {
            if let TokenType::Identifier(name) = &tok.kind {
                name.to_string()
            } else {
                return Err(Error::Parser(format!(
                    "Expected identifier found: {:?}",
                    self.scanner.lookahead()
                )));
            }
        } else {
            return Err(Error::Parser(format!(
                "Expected identifier found: {:?}",
                self.scanner.lookahead()
            )));
        };
        self.advance()?;
        Ok(name)
    }

    fn advance(&mut self) -> Result<(), Error> {
        if !self.is_at_end() {
            if let Some(res) = self.scanner.next() {
                self.tokens.push(res.map_err(|e| Error::Scanner(e))?)
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
            if self.scanner.lookahead_matches(TokenType::Class)
                || self.scanner.lookahead_matches(TokenType::Fun)
                || self.scanner.lookahead_matches(TokenType::Var)
                || self.scanner.lookahead_matches(TokenType::For)
                || self.scanner.lookahead_matches(TokenType::If)
                || self.scanner.lookahead_matches(TokenType::While)
                || self.scanner.lookahead_matches(TokenType::Print)
                || self.scanner.lookahead_matches(TokenType::Return)
            {
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
            Some(self.decl())
        }
    }
}

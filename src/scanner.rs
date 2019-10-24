use super::token::{Token, TokenType};
use super::{Error, SimpleResult};

type ScannerResult = Result<Token, Error>;
type InvertedResult = Result<Option<Token>, Error>;
pub struct Scanner {
    source: Vec<char>,
    start: usize,
    current: usize,
    lookahead: Option<Token>,
    pub line: usize,
}

impl Scanner {
    pub fn new(source: String) -> SimpleResult<Self> {
        let mut ret = Self {
            source: source.chars().collect(),
            start: 0,
            current: 0,
            lookahead: None,
            line: 1,
        };
        let _ = ret.scan_token()?;

        Ok(ret)
    }
    pub fn lookahead(&self) -> &Option<Token> {
        &self.lookahead
    }
    pub fn lookahead_matches(&self, ty: TokenType) -> bool {
        if let Some(t) = self.lookahead().as_ref() {
            t.kind == ty
        } else {
            false
        }
    }
    pub fn scan_tokens(&mut self) -> SimpleResult<Vec<Token>> {
        let ret = self.collect::<Result<Vec<Token>, Error>>()?;
        Ok(ret)
    }

    pub fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    pub fn done(&self) -> bool {
        self.lookahead
            .as_ref()
            .map(|t| t.kind == TokenType::Eof)
            .unwrap_or(true)
    }

    pub fn scan_token(&mut self) -> InvertedResult {
        if self.is_at_end() {
            if self.lookahead.is_none() {
                return Ok(None);
            } else {
                return Ok(::std::mem::replace(&mut self.lookahead, None));
            }
        }
        let next = match self.advance() {
            Some('(') => self.add_token(TokenType::LeftParen),
            Some(')') => self.add_token(TokenType::RightParen),
            Some('{') => self.add_token(TokenType::LeftBrace),
            Some('}') => self.add_token(TokenType::RightBrace),
            Some(',') => self.add_token(TokenType::Comma),
            Some('.') => self.add_token(TokenType::Dot),
            Some('-') => self.add_token(TokenType::Minus),
            Some('+') => self.add_token(TokenType::Plus),
            Some(';') => self.add_token(TokenType::Semicolon),
            Some('*') => self.add_token(TokenType::Star),
            Some('!') => {
                let token = if self.match_next('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token)
            }
            Some('=') => {
                let token = if self.match_next('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token)
            }
            Some('<') => {
                let token = if self.match_next('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token)
            }
            Some('>') => {
                let token = if self.match_next('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token)
            }
            Some('/') => {
                if self.match_next('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                    return self.scan_token();
                } else {
                    self.add_token(TokenType::Slash)
                }
            }
            Some(' ') | Some('\r') | Some('\t') => {
                self.start += 1;
                return self.scan_token();
            }
            Some('\n') => {
                self.line += 1;
                return self.scan_token();
            }
            Some('"') => self.string()?,
            Some(c) => {
                if c.is_digit(10) {
                    self.number()?
                } else if c.is_alphabetic() {
                    self.identifier()?
                } else {
                    self.unknown_token(c)?
                }
            }
            None => self.unknown_token('\0')?,
        };
        let ret = ::std::mem::replace(&mut self.lookahead, Some(next));
        Ok(ret)
    }

    fn unknown_token(&self, c: char) -> Result<Token, Error> {
        Err(Error::Scanner(format!("unknown token found {:?}", c)))
    }

    pub fn advance(&mut self) -> Option<char> {
        self.current += 1;
        self.source.get(self.current - 1).copied()
    }

    pub fn add_token(&mut self, kind: TokenType) -> Token {
        self.add_literal(kind)
    }

    pub fn add_literal(&mut self, kind: TokenType) -> Token {
        let text: String = self.source[self.start..self.current]
            .iter()
            .collect::<String>()
            .trim()
            .to_string();
        Token::new(kind, text, self.line)
    }
    fn match_next(&mut self, e: char) -> bool {
        if self.is_at_end() {
            false
        } else if let Some(c) = self.source.get(self.current) {
            if *c != e {
                false
            } else {
                self.current += 1;
                true
            }
        } else {
            false
        }
    }
    pub fn peek(&self) -> char {
        if let Some(c) = self.source.get(self.current) {
            *c
        } else {
            '\0'
        }
    }

    pub fn string(&mut self) -> ScannerResult {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            let _ = self.advance();
        }
        if self.is_at_end() {
            Err(Error::Scanner("Unterminated string literal".to_string()))
        } else {
            let _ = self.advance();
            let text = self.source[self.start + 1..self.current - 1]
                .iter()
                .collect::<String>()
                .trim()
                .to_string();
            Ok(self.add_literal(TokenType::String(text)))
        }
    }

    pub fn number(&mut self) -> ScannerResult {
        while self.peek().is_digit(10) {
            let _ = self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_digit(10) {
            let _ = self.advance();
            while self.peek().is_digit(10) {
                let _ = self.advance();
            }
        }
        let text = self.source[self.start..self.current]
            .iter()
            .collect::<String>();
        let value = text
            .trim()
            .parse()
            .map_err(|e| Error::Scanner(format!("Unable to parse number {} {}", text, e)))?;
        Ok(self.add_literal(TokenType::Number(value)))
    }

    pub fn identifier(&mut self) -> ScannerResult {
        while self.peek().is_alphanumeric() {
            let _ = self.advance();
        }
        let text = self.source[self.start..self.current]
            .iter()
            .collect::<String>();
        let ty = match text.trim() {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier(text.trim().to_string()),
        };
        Ok(self.add_literal(ty))
    }

    pub fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current]
        }
    }
}

impl Iterator for Scanner {
    type Item = ScannerResult;
    fn next(&mut self) -> Option<Self::Item> {
        self.start = self.current;
        if self.is_at_end() {
            None
        } else {
            self.scan_token().transpose()
        }
    }
}

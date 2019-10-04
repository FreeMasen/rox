use std::{
    path::Path,
    fs::read_to_string,
    io::{
        stdin,
    },

};

type SimpleResult<T> = Result<T, Box<dyn ::std::error::Error>>;

pub struct Lox {
    had_error: bool
}

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    pub line: usize,
}

impl Lox {
    pub fn new() -> Self {
        Self {
            had_error: false,
        }
    }
    pub fn run_file<T>(&mut self, path: T) -> SimpleResult<()>
    where T: AsRef<Path> {
        self.run(read_to_string(path)?)?;
        if self.had_error {
            ::std::process::exit(65);
        }
        Ok(())
    }
    pub fn run_prompt(&mut self) -> SimpleResult<()> {
        let reader = stdin();
        loop {
            let mut line = String::new();
            print!("> ");
            reader.read_line(&mut line)?;
            self.run(line)?;
            self.had_error = false;
        }
        Ok(())
    }
    fn run(&mut self, s: String) -> SimpleResult<()> {
        let mut scanner = Scanner::new(s);
        let tokens = match scanner.scan_tokens() {
            Ok(tokens) => tokens,
            Err(e) => {
                self.error(scanner.line, &format!("{}", e));
                vec![]
            }
        };
        for token in tokens {
            println!("{:?}", token);
        }
        Ok(())
    }

    fn error(&mut self, line: usize, msg: &str) {
        self.report(line, "", msg);
    }
    fn report(&mut self, line: usize, file: &str, msg: &str) {
        println!("[line {}] Error {}: {}", line, file, msg);
        self.had_error = true;
    }
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source: source.chars().collect(),
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
        }
    }
    pub fn scan_tokens(&mut self) -> SimpleResult<Vec<Token>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }
        self.tokens.push(Token::new(TokenType::Eof, String::new(), TokenLiteral::None, self.line));
        Ok(self.tokens.clone())
    }

    pub fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    pub fn scan_token(&mut self) -> SimpleResult<()> {
        match self.advance() {
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
                self.add_token(token);
            },
            Some('=') => {
                let token = if self.match_next('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token);
            },
            Some('<') => {
                let token = if self.match_next('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token)
            },
            Some('>') => {
                let token = if self.match_next('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token)
            },
            Some('/') => if self.match_next('/') {
                while self.peek() != '\n' && !self.is_at_end() {
                    self.advance();
                }
            } else {
                self.add_token(TokenType::Slash)
            }
            Some(' ') 
                | Some('\r') 
                | Some('\t') => (),
            Some('\n') => {
                self.line += 1;
            }
            Some('"') => {
                self.string()?;
            }
            Some(c)  => {
                if c.is_digit(10) {
                    self.number()?;
                } else if c.is_alphabetic() {
                    self.identifier()?;
                } else {
                    self.unknown_token(c)?;
                }
            },
            None => (),
        }
        Ok(())
    }

    fn unknown_token(&self, c: char) -> Result<(), Error> {
        Err(Error(format!("unknown token found {}", c)))
    }

    pub fn advance(&mut self) -> Option<char> {
        self.current += 1;
        self.source.get(self.current - 1).map(|c| *c)
    }

    pub fn add_token(&mut self, kind: TokenType) {
        self.add_literal(kind, TokenLiteral::None);
    }

    pub fn add_literal(&mut self, kind: TokenType, literal: TokenLiteral) {
        let text: String = self.source[self.start..self.current].iter().collect();
        self.tokens.push(Token::new(
            kind,
            text,
            literal,
            self.line
        ))
    }
    fn match_next(&mut self, e: char) -> bool {
        if self.is_at_end() {
            false
        } else {
            if let Some(c) = self.source.get(self.current) {
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
    }
    pub fn peek(&self) -> char {
        if let Some(c) = self.source.get(self.current) {
            *c
        } else {
            '\0'
        }
    }

    pub fn string(&mut self) -> SimpleResult<()> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            let _ = self.advance();
        }
        if self.is_at_end() {
            Err(Box::new(Error(format!("Unterminated string literal"))))
        } else {
            let _ = self.advance();
            let text = self.source[self.start+1..self.current-1].iter().collect();
            self.add_literal(TokenType::String, TokenLiteral::String(text));
            Ok(())
        }
    }

    pub fn number(&mut self) -> SimpleResult<()> {
        while self.peek().is_digit(10) {
            let _ = self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_digit(10) {
            let _ = self.advance();
            while self.peek().is_digit(10) {
                let _ = self.advance();
            }
        }
        let text: String = self.source[self.start..self.current].iter().collect();
        let value = text.parse()?;
        self.add_literal(TokenType::Number, TokenLiteral::Number(value));
        Ok(())
    }

    pub fn identifier(&mut self) -> SimpleResult<()> {
        while self.peek().is_alphanumeric() {
            let _ = self.advance();
        }
        let text: String = self.source[self.start..self.current].iter().collect();
        let (ty, lit) = match text.as_str() {
            "and" => (TokenType::And, TokenLiteral::None),
            "class" => (TokenType::Class, TokenLiteral::None),
            "else" => (TokenType::Else, TokenLiteral::None),
            "false" => (TokenType::False, TokenLiteral::None),
            "for" => (TokenType::For, TokenLiteral::None),
            "fun" => (TokenType::Fun, TokenLiteral::None),
            "if" => (TokenType::If, TokenLiteral::None),
            "nil" => (TokenType::Nil, TokenLiteral::None),
            "or" => (TokenType::Or, TokenLiteral::None),
            "print" => (TokenType::Print, TokenLiteral::None),
            "return" => (TokenType::Return, TokenLiteral::None),
            "super" => (TokenType::Super, TokenLiteral::None),
            "this" => (TokenType::This, TokenLiteral::None),
            "true" => (TokenType::True, TokenLiteral::None),
            "var" => (TokenType::Var, TokenLiteral::None),
            "while" => (TokenType::While, TokenLiteral::None),
            _ => (TokenType::Identifier, TokenLiteral::Ident(text))
        };
        self.add_literal(ty, lit);
        Ok(())
    }

    pub fn peek_next(&self) -> char  {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

}

#[derive(Debug, Clone)]
pub struct Token {
    kind: TokenType,
    lexeme: String,
    literal: TokenLiteral,
    line: usize,
}

impl Token {
    pub fn new(
        kind: TokenType,
        lexeme: String,
        literal: TokenLiteral,
        line: usize,
    ) -> Self {
        Self {
            kind,
            lexeme: lexeme,
            literal,
            line,
        }
    }
}
#[derive(Debug, Clone)]
pub enum TokenLiteral {
    None,
    String(String),
    Number(f64),
    Ident(String),
}
#[derive(Debug, Clone)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Identifier,
    String,
    Number,
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

#[derive(Clone, Debug)]
pub struct Error(String);
impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl ::std::error::Error for Error {}
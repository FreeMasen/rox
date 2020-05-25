use std::{convert::TryFrom, iter::Peekable, str::Chars};

type Result<T> = std::result::Result<T, ScannerError>;
pub enum MatchError {
    MaybeWithNext { yes: TokenType, no: TokenType },
    Ident(char),
    Number,
    String,
    Eof,
    Error,
}
#[derive(Debug)]
pub struct ScannerError {
    pub line: usize,
    pub index: usize,
}
impl ScannerError {
    pub fn new(line: usize, index: usize) -> Self {
        Self { line, index }
    }
}
impl std::fmt::Display for ScannerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Scanner Error line: {} index: {}",self.line, self.index)
    }
}
impl std::error::Error for ScannerError {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Period,
    Minus,
    Plus,
    Semi,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEq,
    Eq,
    EqEq,
    Greater,
    GreaterEq,
    Less,
    LessEq,

    // Literals.
    Ident,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    For,
    Fun,
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

impl TryFrom<char> for TokenType {
    type Error = MatchError;
    fn try_from(ch: char) -> std::result::Result<Self, MatchError> {
        use TokenType::*;
        match ch {
            '(' => Ok(LeftParen),
            ')' => Ok(RightParen),
            '{' => Ok(LeftBrace),
            '}' => Ok(RightBrace),
            ',' => Ok(Comma),
            '.' => Ok(Period),
            '-' => Ok(Minus),
            '+' => Ok(Plus),
            ';' => Ok(Semi),
            '/' => Ok(Slash),
            '*' => Ok(Star),
            '!' => Err(MatchError::MaybeWithNext {
                yes: BangEq,
                no: Bang,
            }),
            '=' => Err(MatchError::MaybeWithNext { yes: EqEq, no: Eq }),
            '>' => Err(MatchError::MaybeWithNext {
                yes: GreaterEq,
                no: Greater,
            }),
            '<' => Err(MatchError::MaybeWithNext {
                yes: LessEq,
                no: Less,
            }),
            '"' => Err(MatchError::String),
            _ if ch.is_digit(10) => Err(MatchError::Number),
            _ if ch.is_alphabetic() => Err(MatchError::Ident(ch)),
            _ => Err(MatchError::Error),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ignoreable {
    Comment,
    Whitespace,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenType,
    pub slice: &'a str,
    pub line: usize,
}

impl<'a> Token<'a> {
    pub fn eof(line: usize) -> Self {
        Self {
            kind: TokenType::Eof,
            slice: "",
            line,
        }
    }
    pub fn new(kind: TokenType, slice: &'a str, line: usize) -> Self {
        Self {
            kind,
            slice,
            line,
        }
    }
}

pub struct Scanner<'a> {
    original: &'a str,
    chars: Peekable<Chars<'a>>,
    pub line: usize,
    found_eof: bool,
    cursor: usize,
    look_ahead: &'a str,
}

impl<'a> std::fmt::Debug for Scanner<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Sanner")
            .field("line", &self.line)
            .field("cursor", &self.cursor)
            .field("look_ahead", &self.look_ahead)
            .finish()
    }
}

impl<'a> Scanner<'a> {
    #[tracing::instrument(skip(s))]
    pub fn new(s: &'a str) -> Self {
        Self {
            original: s,
            chars: s.chars().peekable(),
            line: 1,
            found_eof: false,
            cursor: 0,
            look_ahead: s,
        }
    }
    #[tracing::instrument(skip(self))]
    fn next_token(&mut self) -> Result<Token<'a>> {
        self.skip_whitespace();
        match self.start_token() {
            Ok(kind) => {
                let start = self.cursor;
                self.cursor += 1;
                Ok(self.token(kind, start))
            }
            Err(MatchError::Eof) => Ok(Token {
                slice: &self.original[self.cursor..self.cursor],
                kind: TokenType::Eof,
                line: self.line,
            }),
            Err(MatchError::MaybeWithNext { yes, no }) => {
                let start = self.cursor;
                self.advance();
                let kind = if let Some(&'=') = self.chars.peek() {
                    self.advance();
                    yes
                } else {
                    no
                };
                Ok(self.token(kind, start))
            }
            Err(MatchError::Ident(start)) => Ok(self.ident(start)),
            Err(MatchError::String) => self.string(),
            Err(MatchError::Error) => Err(ScannerError::new(self.line, self.cursor)),
            _ => self.number(),
        }
    }
    #[tracing::instrument(skip(self))]
    fn skip_whitespace(&mut self) {
        while let Some(ignoreable) = self.check_for_ignorable() {
            match ignoreable {
                Ignoreable::Comment => {
                    self.take_through(|c| c == '\n');
                }
                Ignoreable::Whitespace => {
                    self.take_until(|c| !c.is_whitespace());
                }
            }
        }
    }
    #[tracing::instrument(skip(self))]
    fn check_for_ignorable(&mut self) -> Option<Ignoreable> {
        if let Some(look_ahead) = self.original.get(self.cursor..self.cursor + 2) {
            if look_ahead == "//" {
                return Some(Ignoreable::Comment);
            }
            if let Some(ch) = look_ahead.chars().next() {
                if ch.is_whitespace() {
                    return Some(Ignoreable::Whitespace);
                }
            }
        } else if let Some(ch) = self.chars.peek() {
            if ch.is_whitespace() {
                return Some(Ignoreable::Whitespace);
            }
        }
        None
    }
    #[tracing::instrument(skip(self))]
    fn start_token(&mut self) -> std::result::Result<TokenType, MatchError> {
        let _look_ahead = &self.original[self.cursor..];
        let next = self.chars.peek().ok_or(MatchError::Eof)?;
        let ret = TokenType::try_from(*next)?;
        let _ = self.chars.next();
        Ok(ret)
    }
    #[tracing::instrument(skip(self))]
    fn number(&mut self) -> Result<Token<'a>> {
        let start = self.cursor;
        let line = self.line;
        self.advance();
        self.take_until(|c| !c.is_digit(10));
        if let Some(maybe_dot) = self.chars.peek() {
            if *maybe_dot == '.' {
                let mut slice = self.original[self.cursor..].chars();
                let _ = slice.next();
                if let Some(num) = slice.next() {
                    if num.is_digit(10) {
                        self.advance();
                    }
                }
            }
        }
        self.take_until(|c| !c.is_digit(10));
        Ok(Token {
            kind: TokenType::Number,
            line,
            slice: &self.original[start..self.cursor],
        })
    }
    #[tracing::instrument(skip(self))]
    fn string(&mut self) -> Result<Token<'a>> {
        let start = self.cursor;
        self.advance();
        self.take_through(|c| c == '"');
        Ok(self.token(TokenType::String, start))
    }
    #[tracing::instrument(skip(self))]
    fn ident(&mut self, start: char) -> Token<'a> {
        let _look_ahead = &self.original[self.cursor..];
        match start {
            'a' => self.a_keyword(),
            'c' => self.c_keyword(),
            'e' => self.e_keyword(),
            'f' => self.f_keyword(),
            'i' => self.i_keyword(),
            'n' => self.n_keyword(),
            'o' => self.o_keyword(),
            'p' => self.p_keyword(),
            'r' => self.r_keyword(),
            's' => self.s_keyword(),
            't' => self.t_keyword(),
            'v' => self.v_keyword(),
            'w' => self.w_keyword(),
            _ => self.fallback_ident(self.cursor),
        }
    }
    #[tracing::instrument(skip(self))]
    fn a_keyword(&mut self) -> Token<'a> {
        self.single_option_keyword(&['n', 'd'], self.cursor, TokenType::And)
    }
    #[tracing::instrument(skip(self))]
    fn c_keyword(&mut self) -> Token<'a> {
        self.single_option_keyword(&['l', 'a', 's', 's'], self.cursor, TokenType::Class)
    }
    #[tracing::instrument(skip(self))]
    fn e_keyword(&mut self) -> Token<'a> {
        self.single_option_keyword(&['l', 's', 'e'], self.cursor, TokenType::Else)
    }
    #[tracing::instrument(skip(self))]
    fn f_keyword(&mut self) -> Token<'a> {
        let start = self.cursor;
        self.advance();
        match self.chars.peek() {
            Some('a') => self.single_option_keyword(&['l', 's', 'e'], start, TokenType::False),
            Some('o') => self.single_option_keyword(&['r'], start, TokenType::For),
            Some('u') => self.single_option_keyword(&['n'], start, TokenType::Fun),
            _ => self.fallback_ident(start),
        }
    }
    #[tracing::instrument(skip(self))]
    fn i_keyword(&mut self) -> Token<'a> {
        self.single_option_keyword(&['f'], self.cursor, TokenType::If)
    }
    #[tracing::instrument(skip(self))]
    fn n_keyword(&mut self) -> Token<'a> {
        self.single_option_keyword(&['i', 'l'], self.cursor, TokenType::Nil)
    }
    #[tracing::instrument(skip(self))]
    fn o_keyword(&mut self) -> Token<'a> {
        self.single_option_keyword(&['r'], self.cursor, TokenType::Or)
    }
    #[tracing::instrument(skip(self))]
    fn p_keyword(&mut self) -> Token<'a> {
        self.single_option_keyword(&['r', 'i', 'n', 't'], self.cursor, TokenType::Print)
    }
    #[tracing::instrument(skip(self))]
    fn r_keyword(&mut self) -> Token<'a> {
        self.single_option_keyword(&['e', 't', 'u', 'r', 'n'], self.cursor, TokenType::Return)
    }
    #[tracing::instrument(skip(self))]
    fn s_keyword(&mut self) -> Token<'a> {
        self.single_option_keyword(&['u', 'p', 'e', 'r'], self.cursor, TokenType::Super)
    }
    #[tracing::instrument(skip(self))]
    fn t_keyword(&mut self) -> Token<'a> {
        let start = self.cursor;
        self.advance();
        match self.chars.peek() {
            Some('h') => self.single_option_keyword(&['i', 's'], start, TokenType::This),
            Some('r') => self.single_option_keyword(&['u', 'e'], start, TokenType::True),
            _ => self.fallback_ident(start),
        }
    }
    #[tracing::instrument(skip(self))]
    fn v_keyword(&mut self) -> Token<'a> {
        self.single_option_keyword(&['a', 'r'], self.cursor, TokenType::Var)
    }
    #[tracing::instrument(skip(self))]
    fn w_keyword(&mut self) -> Token<'a> {
        self.single_option_keyword(&['h', 'i', 'l', 'e'], self.cursor, TokenType::While)
    }
    #[tracing::instrument(skip(self))]
    fn single_option_keyword(
        &mut self,
        suffix: &[char],
        start: usize,
        kind: TokenType,
    ) -> Token<'a> {
        let _look_ahead = &self.original[self.cursor..];
        self.advance();
        for &ch in suffix {
            if !self.eat(ch) {
                return self.fallback_ident(start);
            }
        }
        if let Some(ch) = self.chars.peek() {
            if ch.is_alphanumeric() {
                self.fallback_ident(start)
            } else {
                self.token(kind, start)
            }
        } else {
            self.token(kind, start)
        }
    }
    #[tracing::instrument(skip(self))]
    fn fallback_ident(&mut self, start: usize) -> Token<'a> {
        self.take_until(|c| !c.is_alphabetic() && !c.is_numeric());
        self.token(TokenType::Ident, start)
    }
    #[tracing::instrument(skip(self))]
    fn eat(&mut self, expect: char) -> bool {
        if let Some(ch) = self.chars.peek() {
            if expect == *ch {
                self.advance();
                return true;
            }
        }
        false
    }
    #[tracing::instrument(skip(f, self))]
    fn take_through<F: Fn(char) -> bool>(&mut self, f: F) {
        self.take_until(f);
        self.advance();
    }
    #[tracing::instrument(skip(f, self))]
    fn take_until<F: Fn(char) -> bool>(&mut self, f: F) {
        while let Some(ch) = self.chars.peek() {
            if f(*ch) {
                break;
            }
            self.advance();
        }
    }
    #[tracing::instrument(skip(self))]
    fn advance(&mut self) {
        if let Some('\n') = self.chars.next() {
            self.line += 1;
        }
        self.cursor += 1;
        self.look_ahead = &self.original[self.cursor..];
    }
    #[tracing::instrument(skip(self))]
    fn token(&self, kind: TokenType, start: usize) -> Token<'a> {
        Token {
            kind,
            line: self.line,
            slice: &self.original[start..self.cursor],
        }
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<Token<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.found_eof {
            None
        } else {
            Some(match self.next_token() {
                Ok(tok) => {
                    if let TokenType::Eof = &tok.kind {
                        self.found_eof = true;
                    }
                    Ok(tok)
                }
                Err(e) => Err(e),
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn keywords() {
        let tokens =
            "and class else false for fun if nil or print return super this true var while";
        let expects = [
            (TokenType::And, "and"),
            (TokenType::Class, "class"),
            (TokenType::Else, "else"),
            (TokenType::False, "false"),
            (TokenType::For, "for"),
            (TokenType::Fun, "fun"),
            (TokenType::If, "if"),
            (TokenType::Nil, "nil"),
            (TokenType::Or, "or"),
            (TokenType::Print, "print"),
            (TokenType::Return, "return"),
            (TokenType::Super, "super"),
            (TokenType::This, "this"),
            (TokenType::True, "true"),
            (TokenType::Var, "var"),
            (TokenType::While, "while"),
            (TokenType::Eof, ""),
        ];
        run_batch(tokens, &expects);
    }

    #[test]
    fn puncts() {
        run_batch(
            "( ) { } , . - + ; / * ! != = == > >= < <=",
            &[
                (TokenType::LeftParen, "("),
                (TokenType::RightParen, ")"),
                (TokenType::LeftBrace, "{"),
                (TokenType::RightBrace, "}"),
                (TokenType::Comma, ","),
                (TokenType::Period, "."),
                (TokenType::Minus, "-"),
                (TokenType::Plus, "+"),
                (TokenType::Semi, ";"),
                (TokenType::Slash, "/"),
                (TokenType::Star, "*"),
                (TokenType::Bang, "!"),
                (TokenType::BangEq, "!="),
                (TokenType::Eq, "="),
                (TokenType::EqEq, "=="),
                (TokenType::Greater, ">"),
                (TokenType::GreaterEq, ">="),
                (TokenType::Less, "<"),
                (TokenType::LessEq, "<="),
                (TokenType::Eof, ""),
            ],
        )
    }

    #[test]
    fn strings() {
        run_batch(
            r#""single line string"
        "multi line string 
        with a bunch of whatspace"
        "#,
            &[
                (TokenType::String, r#""single line string""#),
                (
                    TokenType::String,
                    r#""multi line string 
        with a bunch of whatspace""#,
                ),
                (TokenType::Eof, ""),
            ],
        )
    }

    #[test]
    fn numbers() {
        run_batch(
            "1 1.2 111 999 0987.321",
            &[
                (TokenType::Number, "1"),
                (TokenType::Number, "1.2"),
                (TokenType::Number, "111"),
                (TokenType::Number, "999"),
                (TokenType::Number, "0987.321"),
                (TokenType::Eof, ""),
            ],
        )
    }

    #[test]
    fn idents() {
        run_batch(
            "ifrit forget nope nill ",
            &[
                (TokenType::Ident, "ifrit"),
                (TokenType::Ident, "forget"),
                (TokenType::Ident, "nope"),
                (TokenType::Ident, "nill"),
                (TokenType::Eof, ""),
            ],
        )
    }

    #[test]
    fn skip_whitespace() {
        let whitespace = "      
        \t\t//thigns
        ";
        let s = Scanner::new(whitespace);
        for token in s {
            let t = token.expect("failed to scann eof");
            assert_eq!(t.kind, TokenType::Eof)
        }
    }
    #[test]
    fn multiple_strings() {
        run_batch(r#""first" + "last""#, &[
            (TokenType::String, r#""first""#),
            (TokenType::Plus, "+"),
            (TokenType::String, r#""last""#),
        ]);
    }

    fn run_batch(tokens: &str, expects: &[(TokenType, &str)]) {
        let s = Scanner::new(tokens);
        for (token, expect) in s.zip(expects) {
            let t = token.expect("token was an error");
            assert_eq!(
                &(t.kind, t.slice),
                expect,
                "expected {:?} found {:?} for {:?}",
                expect,
                t.kind,
                t.slice
            );
        }
    }
}

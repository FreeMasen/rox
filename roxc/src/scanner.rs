use crate::error::Error;
use std::{
    str::Chars,
    iter::Peekable,
    convert::TryFrom,
};

type Result<T> = std::result::Result<T, Error>;
pub enum MatchError {
    None,
    MaybeWithNext {
        next: char, 
        yes: TokenType,
        no: TokenType,
    },
    MaybeIdent(char),
    MaybeNumber(char),
    Eof,
}
#[derive(Clone, Copy, Debug)]
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
                next: '=', 
                yes: BangEq,
                no: Bang,
            }),
            '=' => Err(MatchError::MaybeWithNext {
                next: '=',
                yes: EqEq,
                no: Eq
            }),
            '>' => Err(MatchError::MaybeWithNext {
                next: '=',
                yes: GreaterEq,
                no: Greater,
            }),
            '<' => Err(MatchError::MaybeWithNext {
                next: '=',
                yes: LessEq,
                no: Less,
            }),
            _ if ch.is_digit(10) => Err(MatchError::MaybeNumber(ch)),
            _ if ch.is_alphabetic() => Err(MatchError::MaybeIdent(ch)),
            _ => Err(MatchError::None)
        }
    }
}

pub struct Token<'a> {
    pub kind: TokenType,
    pub slice: &'a str,
    pub line: usize,
}

pub struct Scanner<'a> {
    original: &'a str,
    chars: Peekable<Chars<'a>>,
    pub line: usize,
    found_eof: bool,
    cursor: usize,
}

impl<'a> Scanner<'a> {
    fn next_token(&mut self) -> Result<Token<'a>> {
        match self.start_token() {
            Ok(kind) => {
                let start = self.cursor;
                self.cursor += 1;
                Ok(Token {
                    kind,
                    slice: &self.original[start..self.cursor],
                    line: self.line,
                })
            },
            Err(MatchError::None) => {
                Ok(Token {
                    slice: &self.original[self.cursor..self.cursor],
                    kind: TokenType::Eof,
                    line: self.line,
                })
            },
            Err(MatchError::MaybeWithNext { next, yes, no}) => {
                unimplemented!("two? char puncts")
            },
            Err(MatchError::MaybeNumber(first)) => {
                unimplemented!("numbers")
            },
            Err(MatchError::MaybeIdent(first)) => {
                unimplemented!("idents and keywords")
            }
            _ => unimplemented!()
        }
    }

    fn start_token(&mut self) -> std::result::Result<TokenType, MatchError> {
        let next = self.chars.peek().ok_or(MatchError::Eof)?;
        let ret = TokenType::try_from(*next)?;
        let _ = self.chars.next();
        Ok(ret)
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
                Err(e) => Err(e)
            })
        }
        
    }
}
use crate::{
    scanner::{Scanner, ScannerError},
    scanner::{Token, TokenType},
    Chunk, OpCode, Value, Obj,
};
use std::iter::Peekable;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Prec {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Priamary,
}

impl std::ops::Add<usize> for Prec {
    type Output = Prec;
    fn add(self, other: usize) -> Self {
        let n: usize = self.into();
        (n + other).into()
    }
}

impl From<usize> for Prec {
    fn from(n: usize) -> Self {
        use Prec::*;
        match n {
            0 => None,
            1 => Assignment,
            2 => Or,
            3 => And,
            4 => Equality,
            5 => Comparison,
            6 => Term,
            7 => Factor,
            8 => Unary,
            9 => Call,
            10 => Priamary,
            _ => panic!("Overflow Prec"),
        }
    }
}

impl Into<usize> for Prec {
    fn into(self) -> usize {
        use Prec::*;
        match self {
            None => 0,
            Assignment => 1,
            Or => 2,
            And => 3,
            Equality => 4,
            Comparison => 5,
            Term => 6,
            Factor => 7,
            Unary => 8,
            Call => 9,
            Priamary => 10,
        }
    }
}

type ParseFn<'a> = &'a dyn Fn(&mut Compiler<'a>);
type MaybeParseFn<'a> = Option<ParseFn<'a>>;

pub struct Compiler<'a> {
    scanner: Peekable<Scanner<'a>>,
    chunk: Chunk<'a>,
    current: Token<'a>,
    prev: Token<'a>,
    error: Option<ScannerError>,
    panic_mode: bool,
}

impl<'a> std::fmt::Debug for Compiler<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Compiler")
            .field("prev", &self.prev)
            .field("current", &self.current)
            .field("panic_mode", &self.panic_mode)
            .field("error", &self.error)
            .finish()
    }
}

impl<'a> Compiler<'a> {
    #[tracing::instrument()]
    pub fn new(souce: &'a str) -> Self {
        let scanner = Scanner::new(&souce).peekable();
        Self {
            scanner,
            prev: Token::eof(0),
            current: Token::eof(0),
            chunk: Chunk::default(),
            error: None,
            panic_mode: false,
        }
    }
    #[tracing::instrument()]
    pub fn compile(mut self) -> Chunk<'a> {
        self.advance();
        self.expression();
        self.eat(TokenType::Eof);
        self.emit_return();
        if cfg!(feature = "debug") {
            println!("{:#?}", self.chunk);
        }
        self.chunk
    }
    #[tracing::instrument()]
    fn expression(&mut self) {
        self.precedence(Prec::Assignment);
    }
    #[tracing::instrument()]
    fn literal(&mut self) {
        match &self.prev.kind {
            TokenType::False => self.emit_simple_op(OpCode::False, None),
            TokenType::True => self.emit_simple_op(OpCode::True, None),
            TokenType::Nil => self.emit_simple_op(OpCode::Nil, None),
            _ => unreachable!("called literl w/o a literal value"),
        }
    }
    #[tracing::instrument()]
    fn number(&mut self) {
        if let Ok(n) = self.prev.slice.parse() {
            self.emit_constant(Value::Number(n));
        }
    }
    #[tracing::instrument()]
    fn string(&mut self) {
        let idx = self.chunk.add_obj(Obj::string(&self.prev.slice[1..self.prev.slice.len() - 1]));
        self.emit_constant(Value::Obj { idx });
    }
    #[tracing::instrument()]
    fn grouping(&mut self) {
        self.expression();
        if !self.eat(TokenType::RightParen) {
            self.error("Expected closing paren", true);
        }
    }
    #[tracing::instrument()]
    fn unary(&mut self) {
        let op = self.prev.kind;
        self.precedence(Prec::Unary);
        match op {
            TokenType::Minus => self.emit_simple_op(OpCode::Negate, None),
            TokenType::Bang => self.emit_simple_op(OpCode::Not, None),
            _ => (),
        }
    }
    #[tracing::instrument()]
    fn binary(&mut self) {
        use TokenType::*;
        let op = self.prev.kind;
        let prec = Self::determine_precedence(op);
        self.precedence(prec + 1);
        let (first, second) = match op {
            Plus => (OpCode::Add, None),
            Minus => (OpCode::Sub, None),
            Star => (OpCode::Mul, None),
            Slash => (OpCode::Div, None),
            BangEq => (OpCode::Eq, Some(OpCode::Not)),
            EqEq => (OpCode::Eq, None),
            Greater => (OpCode::Gtr, None),
            GreaterEq => (OpCode::Less, Some(OpCode::Not)),
            Less => (OpCode::Less, None),
            LessEq => (OpCode::Gtr, Some(OpCode::Not)),
            _ => return,
        };
        self.emit_simple_op(first, second);
    }
    #[tracing::instrument()]
    fn precedence(&mut self, precedence: Prec) {
        self.advance();
        let prefix = Self::prefix(self.prev.kind);
        if let Some(prefix) = prefix {
            prefix(self);
            while precedence <= Self::determine_precedence(self.current.kind) {
                self.advance();
                let infix = Self::infix(self.prev.kind);
                if let Some(infix) = infix {
                    infix(self);
                } else {
                    self.error("Expect infix expression", false);
                    return;
                }
            }
        } else {
            self.error("Expect prefix expression", false);
        }
    }
    #[tracing::instrument()]
    fn determine_precedence(kind: TokenType) -> Prec {
        use TokenType::*;
        match kind {
            Minus | Plus => Prec::Term,
            Slash | Star => Prec::Factor,
            EqEq | BangEq | LessEq | GreaterEq | Greater | Less => Prec::Equality,
            _ => Prec::None,
        }
    }
    #[tracing::instrument()]
    fn prefix(kind: TokenType) -> MaybeParseFn<'a> {
        use TokenType::*;
        match kind {
            LeftParen => Some(&Self::grouping),
            Minus | Bang => Some(&Self::unary),
            Number => Some(&Self::number),
            String => Some(&Self::string),
            True | False | Nil => Some(&Self::literal),
            _ => None,
        }
    }
    #[tracing::instrument()]
    fn infix(kind: TokenType) -> MaybeParseFn<'a> {
        use TokenType::*;
        match kind {
            Minus | Plus | Slash | Star | Number | EqEq | BangEq | LessEq | GreaterEq | Greater
            | Less => Some(&Self::binary),
            _ => None,
        }
    }
    #[tracing::instrument()]
    fn eat(&mut self, kind: TokenType) -> bool {
        if self.current.kind == kind {
            self.advance();
            true
        } else {
            eprintln!(
                "{:?} didn't match {:?} skipping advance",
                self.current.kind, kind
            );
            false
        }
    }
    #[tracing::instrument()]
    fn emit_return(&mut self) {
        self.emit_simple_op(OpCode::Return, None)
    }
    #[tracing::instrument()]
    fn emit_simple_op(&mut self, code: OpCode, second: Option<OpCode>) {
        self.chunk.write(code, self.current.line);
        if let Some(s) = second {
            self.chunk.write(s, self.current.line);
        }
    }
    #[tracing::instrument()]
    fn emit_constant(&mut self, value: Value) {
        let idx = self.chunk.add_constant(value);
        self.emit_simple_op(OpCode::Constant { idx }, None);
    }
    #[tracing::instrument()]
    fn advance(&mut self) {
        let tok = match self.scanner.next() {
            Some(Ok(tok)) => tok,
            Some(Err(e)) => {
                eprintln!("scanner error {:}", e);
                self.error = Some(e);
                return;
            }
            _ => return,
        };
        self.prev = std::mem::replace(&mut self.current, tok);
    }
    #[tracing::instrument()]
    fn error(&mut self, msg: &str, current: bool) {
        self.panic_mode = true;
        let token = if current { &self.current } else { &self.prev };
        eprint!("[line {}] Error", token.line);
        if token.kind == TokenType::Eof {
            eprint!(" at end");
        } else {
            eprint!(" {:?}", token.slice);
        }
        eprintln!(": {}", msg);
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn strings() {
        let lox = r#""first" + "last""#;
        let c = Compiler::new(lox);
        let chunk = c.compile();
        println!("{:#?}", chunk);
        assert_eq!(chunk.code, vec![
            OpCode::Constant { idx: 0 },
            OpCode::Constant { idx: 1 },
            OpCode::Add,
            OpCode::Return,
        ]);
        assert_eq!(chunk.heap, vec![
            Obj::string("first"),
            Obj::string("last"),
        ]);
    }
}
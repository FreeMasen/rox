mod chunk;
mod compiler;
mod error;
mod op;
mod scanner;
mod value;
mod vm;

pub use chunk::Chunk;
pub use error::Error;
pub use op::OpCode;
pub use value::{Value, Obj};
pub use vm::VM;

pub use scanner::{Token, TokenType};

pub type Result<T> = core::result::Result<T, error::Error>;


#[cfg(test)]
mod test {
    use super::*;
    type TokenList<'a> = Vec<Token<'a>>;
    type TokenListR<'a> = std::result::Result<TokenList<'a>, scanner::ScannerError>;
    #[test]
    fn expressions() {
        let lox = "!(5 - 4 > 3 * 2 == !nil)";
        let scanner = scanner::Scanner::new(lox);
        let tokens = scanner.collect::<TokenListR>().unwrap();
        assert_eq!(tokens, vec![
            Token::new(TokenType::Bang, "!", 1),
            Token::new(TokenType::LeftParen, "(", 1),
            Token::new(TokenType::Number, "5", 1),
            Token::new(TokenType::Minus, "-", 1),
            Token::new(TokenType::Number, "4", 1),
            Token::new(TokenType::Greater, ">", 1),
            Token::new(TokenType::Number, "3", 1),
            Token::new(TokenType::Star, "*", 1),
            Token::new(TokenType::Number, "2", 1),
            Token::new(TokenType::EqEq, "==", 1),
            Token::new(TokenType::Bang, "!", 1),
            Token::new(TokenType::Nil, "nil", 1),
            Token::new(TokenType::RightParen, ")", 1),
            Token::eof(1),
        ]);
        let compiler = compiler::Compiler::new(lox);
        let chunk = compiler.compile();
        assert_eq!(chunk.code, vec![
            OpCode::Constant { idx: 0 }, //5
            OpCode::Constant { idx: 1 }, //4
            OpCode::Sub,
            OpCode::Constant { idx: 2 }, //3
            OpCode::Constant { idx: 3 }, //2
            OpCode::Mul,
            OpCode::Gtr,
            OpCode::Nil,
            OpCode::Not,
            OpCode::Eq,
            OpCode::Not,
            OpCode::Return,
        ]);
    }
}

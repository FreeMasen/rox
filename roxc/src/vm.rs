use crate::{chunk::Chunk, error::Error, op::OpCode, value::Value, Result};
use std::collections::VecDeque;

#[derive(Default)]
pub struct VM {
    pub chunk: Chunk,
    ip: usize,
    stack: VecDeque<Value>,
}

impl VM {
    pub fn interpret(&mut self, bytes: &[u8]) -> Result<()> {
        self.chunk = self.compile(bytes)?;
        self.ip = 0;
        self.run()
    }

    fn compile(&self, _bytes: &[u8]) -> Result<Chunk> {
        Ok(Chunk::default())
    }

    fn run(&mut self) -> Result<()> {
        for i in 0..self.chunk.code[self.ip..].len() {
            let inst = self.chunk.code[i];
            if cfg!(feature = "debug") {
                self.chunk.dissassemble_inst(self.ip, &inst);
                print!("          ");
                println!("{:?}", self.stack);
            }
            self.ip += 1;
            match inst {
                OpCode::Return => {
                    println!("{:?}", self.stack.pop_back());
                    return Ok(());
                }
                OpCode::Constant { idx } => {
                    let c = self.chunk.values[idx];
                    self.stack.push_back(c);
                }
                OpCode::Negate => {
                    if let Some(Value::Number(n)) = self.stack.pop_back() {
                        self.stack.push_back(Value::Number(-n));
                    }
                }
                OpCode::Add => {
                    let lhs = self.pop_operand()?;
                    let rhs = self.pop_operand()?;
                    self.stack.push_back(rhs + lhs)
                }
                OpCode::Sub => {
                    let lhs = self.pop_operand()?;
                    let rhs = self.pop_operand()?;
                    self.stack.push_back(rhs - lhs)
                }
                OpCode::Mul => {
                    let lhs = self.pop_operand()?;
                    let rhs = self.pop_operand()?;
                    self.stack.push_back(rhs * lhs)
                }
                OpCode::Div => {
                    let lhs = self.pop_operand()?;
                    let rhs = self.pop_operand()?;
                    self.stack.push_back(rhs / lhs)
                } // _ => unimplemented!("Unimplemented OpCode: {}", inst),
            }
        }
        Ok(())
    }

    fn pop_operand(&mut self) -> Result<Value> {
        self.stack
            .pop_back()
            .ok_or_else(|| Error::Runtime("Invalid operation, not enough operands".into()))
    }
}

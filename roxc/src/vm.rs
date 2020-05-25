use crate::{chunk::Chunk, compiler::Compiler, error::Error, op::OpCode, value::{Value, Obj}, Result};
use std::{collections::{VecDeque, HashSet}, borrow::Cow};

#[derive(Default)]
pub struct VM<'a> {
    pub chunk: Chunk<'a>,
    ip: usize,
    stack: VecDeque<Value>,
}
impl<'a> std::fmt::Debug for VM<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("VM")
            .field("ip", &self.ip)
            .field("stack", &self.stack)
            .finish()
    }
}

impl<'a> VM<'a> {
    #[tracing::instrument(skip(self, source))]
    pub fn interpret(&mut self, source: &'a str) -> Result<()> {
        self.chunk = self.compile(source)?;
        self.ip = 0;
        self.run()
    }
    #[tracing::instrument(skip(self, source))]
    fn compile(&self, source: &'a str) -> Result<Chunk<'a>> {
        let compiler = Compiler::new(source);
        Ok(compiler.compile())
    }
    #[tracing::instrument(skip(self))]
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
                    let last_value = self.stack.pop_back();
                    self.print_value(last_value);
                    println!();
                    return Ok(());
                }
                OpCode::Constant { idx } => {
                    let c = self.chunk.values[idx];
                    self.stack.push_back(c);
                }
                OpCode::Negate => {
                    if let Some(Value::Number(_)) = self.stack.back() {
                        let n = self.stack.pop_back().unwrap();
                        self.stack.push_back(-n);
                    } else {
                        return self.runtime_err("Operand must be a number.");
                    }
                }
                OpCode::Add => {
                    if !self.operands_match() {
                        return self.runtime_err("Cannot add unmatched operands");
                    }
                    let lhs = self.pop_operand().map_err(|e| {
                        eprintln!("missing lhs in Add");
                        e
                    })?;
                    let rhs = self.pop_operand().map_err(|e| {
                        eprintln!("missing lhs in Add");
                        e
                    })?;
                    let v = match  (lhs, rhs) {
                        (Value::Obj { idx: l_idx }, Value::Obj { idx: r_idx}) => {
                            if let (Obj::String(s1), Obj::String(s2)) = (&self.chunk.heap[l_idx], &self.chunk.heap[r_idx]) {
                                let mut s = s1.to_string();
                                s.push_str(s2);
                                let c = Cow::Owned(s);
                                let idx = self.chunk.add_obj(Obj::String(c));
                                Value::Obj { idx }
                            } else {
                                unimplemented!("")
                            }
                        }
                        _ => lhs + rhs
                    };
                    self.stack.push_back(v)
                }
                OpCode::Sub => {
                    if !self.operands_match() {
                        return self.runtime_err("Cannot subtract unmatched operands");
                    }
                    let lhs = self.pop_operand().map_err(|e| {
                        eprintln!("missing lhs in Sub");
                        e
                    })?;
                    let rhs = self.pop_operand().map_err(|e| {
                        eprintln!("missing rhs in Sub");
                        e
                    })?;
                    self.stack.push_back(rhs - lhs)
                }
                OpCode::Mul => {
                    if !self.operands_match() {
                        return self.runtime_err("Cannot multiply unmatched operands");
                    }
                    let lhs = self.pop_operand().map_err(|e| {
                        eprintln!("missing lhs in Mul");
                        e
                    })?;
                    let rhs = self.pop_operand().map_err(|e| {
                        eprintln!("missing rhs in Mul");
                        e
                    })?;
                    self.stack.push_back(rhs * lhs)
                }
                OpCode::Div => {
                    if !self.operands_match() {
                        return self.runtime_err("Cannot divide unmatched operands");
                    }
                    let lhs = self.pop_operand().map_err(|e| {
                        eprintln!("missing lhs in Div");
                        e
                    })?;
                    let rhs = self.pop_operand().map_err(|e| {
                        eprintln!("missing rhs in Div");
                        e
                    })?;
                    self.stack.push_back(rhs / lhs)
                }
                OpCode::True => {
                    self.stack.push_back(Value::Boolean(true));
                }
                OpCode::False => {
                    self.stack.push_back(Value::Boolean(false));
                }
                OpCode::Nil => {
                    self.stack.push_back(Value::Nil);
                }
                OpCode::Not => {
                    if self.stack.back().is_some() {
                        let b = self.stack.pop_back().unwrap();
                        self.stack.push_back(!b);
                    } else {
                        return self.runtime_err("Cannot negate a non boolean");
                    }
                }
                OpCode::Eq => {
                    let lhs = self.pop_operand().map_err(|e| {
                        eprintln!("missing lhs in Eq");
                        e
                    })?;
                    let rhs = self.pop_operand().map_err(|e| {
                        eprintln!("missing rhs in Eq");
                        e
                    })?;
                    let b = match  (lhs, rhs) {
                        (Value::Obj { idx: l_idx }, Value::Obj { idx: r_idx}) => {
                            self.chunk.heap[l_idx] == self.chunk.heap[r_idx]
                        }
                        _ => lhs == rhs
                    };
                    self.stack.push_back(Value::Boolean(b));
                }
                OpCode::Less => {
                    if !self.operands_match() {
                        return self.runtime_err("Cannot compare unmatched operands");
                    }
                    let lhs = self.pop_operand().map_err(|e| {
                        eprintln!("missing lhs in Less");
                        e
                    })?;
                    let rhs = self.pop_operand().map_err(|e| {
                        eprintln!("missing rhs in Less");
                        e
                    })?;
                    self.stack.push_back(Value::Boolean(lhs < rhs))
                }
                OpCode::Gtr => {
                    if !self.operands_match() {
                        return self.runtime_err("Cannot compare unmatched operands");
                    }
                    let lhs = self.pop_operand().map_err(|e| {
                        eprintln!("missing lhs in Gtr");
                        e
                    })?;
                    let rhs = self.pop_operand().map_err(|e| {
                        eprintln!("missing rhs in Gtr");
                        e
                    })?;
                    self.stack.push_back(Value::Boolean(lhs > rhs))
                }
            }
        }
        Ok(())
    }
    fn print_value(&self, value: Option<Value>) {
        print!("{}", self.format_value(value.as_ref(), 0));
    }

    fn format_value(&self, value: Option<&Value>, indent: usize) -> String {
        let mut ret = String::new();
        if let Some(value) = value {
            match value {
                Value::Number(n) => ret.push_str(&format!("{}", n)),
                Value::Nil => ret.push_str("nil"),
                Value::Boolean(b) => ret.push_str(&format!("{}", b)),
                Value::Obj { idx } => {
                    let heap = &self.chunk.heap[*idx];
                    match heap {
                        Obj::String(s) => ret.push_str(&format!("{}", s)),
                        Obj::HashTable { entries } => {
                            ret.push('{');
                            ret.push('\n');
                            ret.push_str(&"  ".repeat(indent + 1));
                            for (entry, value) in entries {
                                ret.push_str(entry);
                                ret.push_str(": ");
                                ret.push_str(&self.format_value(Some(value), indent + 2))
                            }
                            ret.push('}');
                        }
                    }
                }
            }
        } else {
            ret.push_str("nil");
        }
        ret
    }
    #[tracing::instrument()]
    fn operands_match(&self) -> bool {
        match (self.stack.back(), self.stack.get(self.stack.len() - 2)) {
            (Some(Value::Number(_)), Some(Value::Number(_)))
            | (Some(Value::Boolean(_)), Some(Value::Boolean(_)))
            | (Some(Value::Nil), Some(Value::Nil))
            | (Some(Value::Obj { .. }), Some(Value::Obj { .. })) => true,
            _ => false,
        }
    }
    #[tracing::instrument()]
    fn pop_operand(&mut self) -> Result<Value> {
        self.stack
            .pop_back()
            .ok_or_else(|| self.runtime_error("Invalid operation, not enough operands"))
    }
    #[tracing::instrument()]
    fn runtime_err(&self, msg: &str) -> Result<()> {
        Err(self.runtime_error(msg))
    }
    #[tracing::instrument()]
    fn runtime_error(&self, msg: &str) -> Error {
        eprintln!("{}", msg);
        eprintln!(
            "[line {}] in script",
            self.chunk.lines.get_unchecked(self.ip)
        );
        Error::Runtime(msg.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn strings() {
        
    }
}

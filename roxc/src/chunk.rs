use crate::op::OpCode;
use crate::value::Value;
#[derive(Default)]
pub struct Chunk {
    pub code: Vec<OpCode>,
    pub values: Vec<Value>,
    pub lines: RunList,
}

impl Chunk {
    pub fn write(&mut self, byte: OpCode, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.values.push(value);
        self.values.len() - 1
    }

    pub fn dissassemble_all(&self, name: &str) {
        println!("== {} ==", name);
        for (i, code) in self.code.iter().enumerate() {
            self.dissassemble_inst(i, code)
        }
    }

    pub fn dissassemble_inst_idx(&self, i: usize) {
        self.dissassemble_inst(i, &self.code[i])
    }

    pub fn dissassemble_inst(&self, i: usize, code: &OpCode) {
        print!("{:04} ", i);
        self.dissassemblly_line(i);
        match code {
            OpCode::Return
            | OpCode::Negate
            | OpCode::Add
            | OpCode::Sub
            | OpCode::Mul
            | OpCode::Div => println!("{}", code),
            OpCode::Constant { idx } => {
                println!("{: <16} {:?}", format!("{:}", code), &self.values[*idx])
            }
        }
    }

    fn dissassemblly_line(&self, idx: usize) {
        if idx == 0 {
            print!("{:04} ", self.lines.get_unchecked(0));
        } else {
            let prev = self.lines.get_unchecked(idx - 1);
            let curr = self.lines.get_unchecked(idx);
            if prev == curr {
                print!("   | ");
            } else {
                print!("{:04}", curr);
            }
        }
    }
}

pub struct RunLength {
    value: usize,
    len: usize,
}

#[derive(Default)]
pub struct RunList {
    values: Vec<RunLength>,
}

impl RunList {
    pub fn push(&mut self, i: usize) {
        if let Some(v) = self.values.last_mut() {
            if v.len == i {
                v.len += 1;
                return;
            }
        }
        self.values.push(RunLength { value: i, len: 1 })
    }

    pub fn get_unchecked(&self, idx: usize) -> usize {
        let mut ct = 0;
        for value in &self.values {
            let new_ct = ct + value.len;
            if idx >= ct && idx < new_ct {
                return value.value;
            }
            ct = new_ct;
        }
        panic!("index out of range idx: {}, len: {}", idx, ct);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn run_list() {
        let values = [1usize, 2, 3, 3, 3, 3, 4, 5, 6, 6, 7];
        let mut rl = RunList::default();
        for v in values.iter() {
            rl.push(*v);
        }
        for (i, v) in values.iter().enumerate() {
            assert_eq!(*v, rl.get_unchecked(i));
        }
    }
}

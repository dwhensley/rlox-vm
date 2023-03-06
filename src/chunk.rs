use std::fmt::Debug;

use crate::value::Value;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum OpCode {
    Return,
    Constant,
}

impl TryFrom<u8> for OpCode {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            v if v == OpCode::Return as u8 => Ok(OpCode::Return),
            v if v == OpCode::Constant as u8 => Ok(OpCode::Constant),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Rle<T: Debug + Copy> {
    num: u8,
    value: T,
}

impl<T: Debug + Copy> Rle<T> {
    fn new(value: T) -> Self {
        Self { num: 1, value }
    }
    fn increment(&mut self) {
        self.num += 1;
    }
}

#[derive(Debug, Clone, Default)]
pub struct Chunk {
    code: Vec<u8>,
    lines: Vec<Rle<usize>>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }

    pub fn write_byte(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        match self.lines.last_mut() {
            Some(last_line) if last_line.value == line => {
                last_line.increment();
            }
            _ => self.lines.push(Rle::new(line)),
        }
    }

    pub fn get_line(&self, offset: usize) -> usize {
        let mut start = 0;
        for rle in &self.lines {
            if (start..(start + rle.num)).contains(&(offset as u8)) {
                return rle.value;
            }
            start += rle.num;
        }
        panic!("Offset `{offset}` not associated with any line");
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {name} ==");
        let mut offset = 0;
        while offset < self.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{offset:04} ");
        if offset > 0 && self.get_line(offset) == self.get_line(offset - 1) {
            print!("   | ");
        } else {
            print!("{:4} ", self.get_line(offset));
        }
        if let Ok(instruction) = OpCode::try_from(self.code[offset]) {
            match instruction {
                OpCode::Constant => self.constant_instruction("OP_CONSTANT", offset),
                OpCode::Return => Self::simple_instruction("OP_RETURN", offset),
            }
        } else {
            panic!("Unknown opcode {}", self.code[offset]);
        }
    }

    fn simple_instruction(name: &str, offset: usize) -> usize {
        println!("{name}");
        offset + 1
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant_idx = self.code[offset + 1];
        print!("{name:-16} {constant_idx:4} '");
        print!("{}", self.constants[constant_idx as usize]);
        println!("'");
        offset + 2
    }
}

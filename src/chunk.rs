use std::fmt::Debug;

use crate::value::Value;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum OpCode {
    Return,
    Constant,
    ConstantLong,
}

impl TryFrom<u8> for OpCode {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            v if v == OpCode::Return as u8 => Ok(OpCode::Return),
            v if v == OpCode::Constant as u8 => Ok(OpCode::Constant),
            v if v == OpCode::ConstantLong as u8 => Ok(OpCode::ConstantLong),
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

    pub fn write_constant(&mut self, value: Value, line: usize) {
        if self.constants.len() < u8::MAX as usize {
            self.write_constant_short(value, line);
        } else {
            self.write_constant_long(value, line);
        }
    }

    fn write_constant_short(&mut self, value: Value, line: usize) {
        let constant_idx = self.add_constant(value);
        self.write_byte(OpCode::Constant as u8, line);
        self.write_byte(constant_idx.try_into().expect("Too many constants!"), line);
    }

    fn write_constant_long(&mut self, value: Value, line: usize) {
        let constant_idx = self.add_constant(value);
        self.write_byte(OpCode::ConstantLong as u8, line);
        let [b1, b2] = TryInto::<u16>::try_into(constant_idx)
            .expect("Too many (long) constants!")
            .to_le_bytes();
        self.write_byte(b1, line);
        self.write_byte(b2, line);
    }

    pub fn get_line(&self, offset: usize) -> usize {
        let mut start = 0;
        for rle in &self.lines {
            if (start..(start + rle.num as usize)).contains(&offset) {
                return rle.value;
            }
            start += rle.num as usize;
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
                OpCode::ConstantLong => self.constant_long_instruction("OP_CONSTANT_LONG", offset),
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

    fn constant_long_instruction(&self, name: &str, offset: usize) -> usize {
        let b1 = self.code[offset + 1];
        let b2 = self.code[offset + 2];
        let constant_idx = u16::from_le_bytes([b1, b2]);
        print!("{name:-16} {constant_idx:4} '");
        print!("{}", self.constants[constant_idx as usize]);
        println!("'");
        offset + 3
    }
}

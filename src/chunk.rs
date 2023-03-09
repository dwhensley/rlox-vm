use std::fmt::Debug;

use crate::value::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChunkError {
    #[error("Unrecognized OpCode `{0}`")]
    ParseOpCode(u8),
    #[error("Attempted to construct too many (short) constants (<= 255)")]
    TooManyConstantsShort,
    #[error("Attempted to construct too many (long) constants (<= 65,536)")]
    TooManyConstantsLong,
    #[error("Offset `{0}` not associated with any line")]
    ParseLineForOffset(usize),
}

pub type ChunkResult<T> = Result<T, ChunkError>;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum OpCode {
    Constant = 0,
    ConstantLong,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return,
}

impl OpCode {
    #[inline]
    pub fn from_u8(value: u8) -> Option<Self> {
        use OpCode::*;
        match value {
            0 => Some(Constant),
            1 => Some(ConstantLong),
            2 => Some(Add),
            3 => Some(Subtract),
            4 => Some(Multiply),
            5 => Some(Divide),
            6 => Some(Negate),
            7 => Some(Return),
            _ => None,
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
    pub(crate) code: Vec<u8>,
    pub(crate) lines: Vec<Rle<usize>>,
    pub(crate) constants: Vec<Value>,
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

    pub fn write_constant(&mut self, value: Value, line: usize) -> ChunkResult<()> {
        if self.constants.len() < u8::MAX as usize {
            self.write_constant_short(value, line)
        } else {
            self.write_constant_long(value, line)
        }
    }

    fn write_constant_short(&mut self, value: Value, line: usize) -> ChunkResult<()> {
        let constant_idx = self.add_constant(value);
        self.write_byte(OpCode::Constant as u8, line);
        let trunc_idx = constant_idx
            .try_into()
            .map_err(|_| ChunkError::TooManyConstantsShort)?;
        self.write_byte(trunc_idx, line);
        Ok(())
    }

    fn write_constant_long(&mut self, value: Value, line: usize) -> ChunkResult<()> {
        let constant_idx = self.add_constant(value);
        self.write_byte(OpCode::ConstantLong as u8, line);
        let [b1, b2] = TryInto::<u16>::try_into(constant_idx)
            .map_err(|_| ChunkError::TooManyConstantsLong)?
            .to_le_bytes();
        self.write_byte(b1, line);
        self.write_byte(b2, line);
        Ok(())
    }

    pub fn get_line(&self, offset: usize) -> ChunkResult<usize> {
        let mut start = 0;
        for rle in &self.lines {
            if (start..(start + rle.num as usize)).contains(&offset) {
                return Ok(rle.value);
            }
            start += rle.num as usize;
        }
        Err(ChunkError::ParseLineForOffset(offset))
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn disassemble(&self, name: &str) -> ChunkResult<()> {
        println!("== {name} ==");
        let mut offset = 0;
        while offset < self.len() {
            offset = self.disassemble_instruction(offset)?;
        }
        Ok(())
    }

    pub fn disassemble_instruction(&self, offset: usize) -> ChunkResult<usize> {
        print!("{offset:04} ");
        let line = self.get_line(offset)?;
        if offset > 0 && line == self.get_line(offset - 1)? {
            print!("   | ");
        } else {
            print!("{line:4} ");
        }
        if let Some(instruction) = OpCode::from_u8(self.code[offset]) {
            match instruction {
                OpCode::Constant => Ok(self.constant_instruction("OP_CONSTANT", offset)),
                OpCode::ConstantLong => {
                    Ok(self.constant_long_instruction("OP_CONSTANT_LONG", offset))
                }
                OpCode::Add => Ok(Self::simple_instruction("OP_ADD", offset)),
                OpCode::Subtract => Ok(Self::simple_instruction("OP_SUBTRACT", offset)),
                OpCode::Multiply => Ok(Self::simple_instruction("OP_MULTIPLY", offset)),
                OpCode::Divide => Ok(Self::simple_instruction("OP_DIVIDE", offset)),
                OpCode::Negate => Ok(Self::simple_instruction("OP_NEGATE", offset)),
                OpCode::Return => Ok(Self::simple_instruction("OP_RETURN", offset)),
            }
        } else {
            Err(ChunkError::ParseOpCode(self.code[offset]))
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

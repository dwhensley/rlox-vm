use crate::chunk::{Chunk, OpCode};
use crate::value::Value;

use thiserror::Error;

const STACK_MAX: usize = 256;

macro_rules! binary_op {
    ($self:ident, $op:tt) => {{
        let b = $self.pop();
        // Don't explicitly pop `a` off -- update in place.
        unsafe { *($self.stack_top.sub(1)) = *($self.stack_top.sub(1)) $op b };
    }}
}

#[derive(Error, Debug)]
pub enum InterpretError {
    #[error("Compilation error: {0}")]
    Compilation(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
}

pub type InterpretResult<T> = Result<T, InterpretError>;

pub struct Vm {
    chunk: Chunk,
    ip: *mut u8,
    stack: [Value; STACK_MAX],
    stack_top: *mut Value,
}

impl Vm {
    pub fn new(chunk: Chunk) -> Self {
        let mut vm = Self {
            chunk,
            ip: std::ptr::null::<*const u8>() as *mut u8,
            stack: [0.0; STACK_MAX],
            stack_top: std::ptr::null::<*const Value> as *mut Value,
        };
        vm.ip = std::ptr::addr_of!(vm.chunk.code[0]) as *mut u8;
        vm.stack_top = std::ptr::addr_of!(vm.stack[0]) as *mut Value;
        vm
    }

    pub fn reset_stack(&mut self) {
        self.stack_top = std::ptr::addr_of!(self.stack[0]) as *mut Value;
    }

    pub fn push(&mut self, value: Value) {
        unsafe {
            *self.stack_top = value;
            self.stack_top = self.stack_top.add(1);
        };
    }

    pub fn pop(&mut self) -> Value {
        unsafe {
            self.stack_top = self.stack_top.sub(1);
            *self.stack_top
        }
    }

    pub fn run(&mut self) -> InterpretResult<()> {
        use OpCode::*;
        loop {
            #[cfg(debug_assertions)]
            {
                print!("          ");
                let stack_top_offset = unsafe {
                    self.stack_top
                        .offset_from(std::ptr::addr_of!(self.stack[0]))
                } as usize;
                for slot in &self.stack[0..stack_top_offset] {
                    print!("[ {slot} ]");
                }
                println!();
                let offset =
                    unsafe { self.ip.offset_from(std::ptr::addr_of!(self.chunk.code[0])) } as usize;
                self.chunk
                    .disassemble_instruction(offset)
                    .map_err(|e| InterpretError::Runtime(e.to_string()))?;
            }

            let byte = unsafe { self.read_byte() };
            match OpCode::from_u8(byte) {
                Some(Constant) => {
                    let constant = self.read_constant();
                    self.push(constant);
                }
                Some(ConstantLong) => {
                    let constant = self.read_constant_long();
                    self.push(constant);
                }
                Some(Negate) => {
                    unsafe { *(self.stack_top.sub(1)) = -*(self.stack_top.sub(1)) };
                }
                Some(Add) => binary_op!(self, +),
                Some(Subtract) => binary_op!(self, -),
                Some(Multiply) => binary_op!(self, *),
                Some(Divide) => binary_op!(self, /),
                Some(Return) => {
                    println!("{}", self.pop());
                    return Ok(());
                }
                None => {
                    return Err(InterpretError::Runtime(format!(
                        "Unsupported opcode: `{byte}`"
                    )));
                }
            }
        }
    }

    #[inline]
    unsafe fn read_byte(&mut self) -> u8 {
        let byte = *self.ip;
        self.ip = self.ip.add(1);
        byte
    }

    fn read_constant(&mut self) -> Value {
        let constant_idx = unsafe { self.read_byte() } as usize;
        self.chunk.constants[constant_idx]
    }

    fn read_constant_long(&mut self) -> Value {
        let b1 = unsafe { self.read_byte() };
        let b2 = unsafe { self.read_byte() };
        self.chunk.constants[u16::from_le_bytes([b1, b2]) as usize]
    }
}

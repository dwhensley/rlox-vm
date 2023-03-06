mod chunk;
mod value;

use chunk::{Chunk, OpCode};

fn main() {
    let mut chunk = Chunk::new();

    let constant_idx = chunk.add_constant(1.2);
    chunk.write_byte(OpCode::Constant as u8, 123);
    chunk.write_byte(constant_idx.try_into().expect("Too many constants"), 123);
    chunk.write_byte(OpCode::Return as u8, 123);

    chunk.disassemble("test chunk");
}

mod chunk;
mod value;
mod vm;

use chunk::{Chunk, OpCode};
use vm::Vm;

use anyhow::Result;

fn main() -> Result<()> {
    // Prepare the chunk.
    let mut chunk = Chunk::new();
    chunk.write_constant(1.2, 123)?;
    chunk.write_constant(3.4, 123)?;
    chunk.write_byte(OpCode::Add as u8, 123);
    chunk.write_constant(5.6, 123)?;
    chunk.write_byte(OpCode::Divide as u8, 123);
    chunk.write_byte(OpCode::Negate as u8, 123);
    chunk.write_byte(OpCode::Return as u8, 123);

    // Disassemble the chunk for review.
    chunk.disassemble("test chunk")?;

    // Run the chunk in the VM.
    let mut vm = Vm::new(chunk);
    vm.run()?;

    Ok(())
}

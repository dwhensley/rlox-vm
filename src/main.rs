mod chunk;
mod value;

use chunk::{Chunk, OpCode};

use anyhow::Result;

fn main() -> Result<()> {
    let mut chunk = Chunk::new();

    chunk.write_constant(1.2, 123)?;
    chunk.write_constant(3.4, 123)?;
    chunk.write_constant(5.7, 124)?;
    chunk.write_constant(10.2, 126)?;

    for v in 0..255 {
        chunk.write_constant(v as value::Value, 126 + v)?;
    }

    chunk.write_byte(OpCode::Return as u8, 127);

    chunk.disassemble("test chunk")?;

    Ok(())
}

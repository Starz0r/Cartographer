use std::fs::File;
use std::io::{Result, Write};

use byteorder::{LittleEndian, WriteBytesExt};

pub fn cell_set(mut f: &File, x: i16, y: i16, val: i8) -> Result<()> {
    f.write_u8(0x2)?; // grid cell instruction
    f.write_i16::<LittleEndian>(x)?; // x operand
    f.write_i16::<LittleEndian>(y)?; // y operand
    f.write_i8(val)?; // val operand
    Ok(())
}

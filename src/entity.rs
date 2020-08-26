use std::fs::File;
use std::io::{Result, Write};

use byteorder::{LittleEndian, WriteBytesExt};

pub fn create(mut f: &File, ent: i32, x: i32, y: i32, w: u32, h: u32) -> Result<()> {
    f.write_u8(0x3)?; // entity create instruction
    f.write_i32::<LittleEndian>(ent)?; // entity operand
    f.write_i32::<LittleEndian>(x)?; // x operand
    f.write_i32::<LittleEndian>(y)?; // y operand
    f.write_u32::<LittleEndian>(w)?; // width operand
    f.write_u32::<LittleEndian>(h)?; // height operand
    Ok(())
}

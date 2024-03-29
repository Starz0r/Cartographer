use std::fs::File;
use std::io::Result;

use byteorder::{LittleEndian, WriteBytesExt};

pub fn create(
    mut f: &File,
    ent: i32,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    rot: i16,
    fx: bool,
    fy: bool,
) -> Result<()> {
    f.write_u8(0x3)?; // entity create instruction
    f.write_i32::<LittleEndian>(ent)?; // entity operand
    f.write_i32::<LittleEndian>(x)?; // x operand
    f.write_i32::<LittleEndian>(y)?; // y operand
    f.write_u32::<LittleEndian>(w)?; // width operand
    f.write_u32::<LittleEndian>(h)?; // height operand
    f.write_i16::<LittleEndian>(rot)?; // rotation operand
    f.write_u8(fx as u8)?; // flipped x operand
    f.write_u8(fy as u8)?; // flipped y operand
    Ok(())
}

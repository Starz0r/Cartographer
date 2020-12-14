use std::fs::File;
use std::io::{Result, Write};

use byteorder::{LittleEndian, WriteBytesExt};

pub fn set_width(mut f: &File, w: u32) -> Result<()> {
    f.write_u8(0x1)?; // layer properties instruction
    f.write_u8(0x0)?; // width flag
    f.write_u32::<LittleEndian>(w)?; // width operand
    Ok(())
}
pub fn set_height(mut f: &File, h: u32) -> Result<()> {
    f.write_u8(0x1)?; // height properties instruction
    f.write_u8(0x1)?; // height flag
    f.write_u32::<LittleEndian>(h)?; // width operand
    Ok(())
}

pub fn set_type(mut f: &File, typ: u8) -> Result<()> {
    f.write_u8(0x1)?; // layer properties instruction
    f.write_u8(0x4)?; // set type flag
    f.write_u8(typ)?; // type operand
    Ok(())
}

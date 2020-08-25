use std::fs::File;
use std::io::{Result, Write};

use byteorder::{LittleEndian, WriteBytesExt};

pub fn write_header(mut f: &File) -> Result<()> {
    f.write("LVL Format 0.".as_bytes())?;
    Ok(())
}

pub fn set_width(mut f: &File, w: u64) -> Result<()> {
    f.write_u8(0x0)?; // global properties instruction
    f.write_u8(0x0)?; // width flag
    for char in w.to_le_bytes().iter() {
        f.write_u8(*char)?; // width operand
    }
    Ok(())
}

pub fn set_height(mut f: &File, h: u64) -> Result<()> {
    f.write_u8(0x0)?; // global properties instruction
    f.write_u8(0x1)?; // height flag
    for char in h.to_le_bytes().iter() {
        f.write_u8(*char)?; // height operand
    }
    Ok(())
}

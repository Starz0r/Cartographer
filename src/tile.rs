use std::fs::File;
use std::io::{Result, Write};

use byteorder::{LittleEndian, WriteBytesExt};

pub fn new(mut f: &File, tileset: i32, x: u32, y: u32, tile_x: u16, tile_y: u16) -> Result<()> {
    f.write_u8(0x4)?; // tile add instruction
    f.write_i32::<LittleEndian>(tileset)?; // tileset operand
    f.write_u32::<LittleEndian>(x)?; // x operand
    f.write_u32::<LittleEndian>(y)?; // y operand
    f.write_u16::<LittleEndian>(tile_x)?; // tile_x operand
    f.write_u16::<LittleEndian>(tile_y)?; // tile_y operand
    Ok(())
}

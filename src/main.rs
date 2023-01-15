mod entity;
mod global;
mod grid;
mod layer;
mod tile;

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use {
    anyhow::Result,
    byteorder::{LittleEndian, WriteBytesExt},
    serde::{Deserialize, Serialize},
    structopt::StructOpt,
};

use scopeguard::{defer, defer_on_unwind};

#[derive(StructOpt, Debug)]
#[structopt(name = "cartographer")]
struct Cli {
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    #[structopt(long, parse(from_os_str))]
    info_table: PathBuf,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OgmoMap {
    pub ogmo_version: String,
    pub width: u64,
    pub height: u64,
    pub offset_x: i64,
    pub offset_y: i64,
    pub layers: Vec<OgmoLayer>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OgmoLayer {
    pub name: String,
    #[serde(rename = "_eid")]
    pub eid: String,
    pub offset_x: i64,
    pub offset_y: i64,
    pub grid_cell_width: u32,
    pub grid_cell_height: u32,
    pub grid_cells_x: i64,
    pub grid_cells_y: i64,
    #[serde(default)]
    pub grid: Option<Vec<String>>,
    pub array_mode: Option<i64>,
    #[serde(default, rename = "grid2D")]
    pub grid2d: Option<Vec<Vec<String>>>,
    #[serde(default)]
    pub entities: Vec<OgmoEntity>,
    pub tileset: Option<String>,
    #[serde(default)]
    pub data_coords2_d: Vec<Vec<Vec<i64>>>,
    pub export_mode: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OgmoEntity {
    pub name: String,
    pub id: i64,
    #[serde(rename = "_eid")]
    pub eid: String,
    pub x: i64,
    pub y: i64,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub origin_x: i64,
    pub origin_y: i64,
    pub rotation: Option<i64>,
    pub flipped_x: Option<bool>,
    pub flipped_y: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoTables {
    pub entity_table: Vec<EntityTableEntry>,
    pub tileset_table: Vec<TilesetTableEntry>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityTableEntry {
    pub name: String,
    pub value: i64,
    pub width: i64,
    pub height: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TilesetTableEntry {
    pub name: String,
    pub value: i64,
}

pub fn main() -> Result<()> {
    use std::io::Read;
    use std::io::Write;

    // read cli arguments
    let opt = Cli::from_args();

    // buffered file reading
    let src = File::open(opt.input)?;
    let mut buf = BufReader::new(src);
    let mut con = String::new();

    // marshal into a data structure
    buf.read_to_string(&mut con)?;
    let map: OgmoMap = serde_json::from_str(&con)?;

    // read infotable
    let infotable_file = File::open(opt.info_table)?;
    let mut infotable_buf = BufReader::new(infotable_file);
    let mut infotable_contents = String::new();
    infotable_buf.read_to_string(&mut infotable_contents)?;
    let infotable = serde_json::from_str::<InfoTables>(&infotable_contents)?;

    let ent_table = infotable.entity_table;
    let ts_table = infotable.tileset_table;

    // open a new file
    let dst = File::create(opt.output)?;

    // write file header
    global::write_header(&dst)?;

    // global level properties
    global::set_width(&dst, map.width)?;
    global::set_height(&dst, map.height)?;

    // iterate through layers
    for layer in map.layers.iter() {
        // signal layer type
        if !layer.data_coords2_d.is_empty() {
            layer::set_type(&dst, 0)?;
        }
        if layer.array_mode.is_some() {
            layer::set_type(&dst, 1)?;
        }
        if !layer.entities.is_empty() {
            layer::set_type(&dst, 3)?;
        }

        // set width and height
        layer::set_width(&dst, layer.grid_cell_width)?;
        layer::set_height(&dst, layer.grid_cell_height)?;

        if layer.array_mode.is_some() {
            let array_mode = layer.array_mode.unwrap();

            // keep track of the y axis manually
            if array_mode == 0 || layer.grid.is_some() {
                let (mut cur_x, mut cur_y) = (0, 0);
                let grid = layer.grid.as_ref().unwrap();
                // grid cells
                for cell in grid.iter() {
                    if cur_x == layer.grid_cells_x {
                        cur_x = 0; // reset x cursor
                        cur_y += 1;
                    }

                    grid::cell_set(&dst, cur_x as i16, cur_y as i16, cell.parse::<i8>()?).unwrap();
                }
            } else {
                if layer.grid2d.is_some() {
                    let (mut cur_x, mut cur_y) = (0, 0);
                    let grid = layer.grid2d.as_ref().unwrap();
                    // grid rows and cells
                    for row in grid.iter() {
                        for cell in row.iter() {
                            grid::cell_set(&dst, cur_x as i16, cur_y as i16, cell.parse::<i8>()?)?;
                            cur_x += 1;
                        }
                        cur_x = 0; // reset x cursor
                        cur_y += 1;
                    }
                }
            }
        }

        // entities
        let entities = &layer.entities;
        for entity in entities.iter() {
            for entry in ent_table.iter() {
                if entity.name.eq(entry.name.as_str()) {
                    let w = entity.width.unwrap_or(entry.width);
                    let h = entity.height.unwrap_or(entry.height);
                    let rot = entity.rotation.unwrap_or(0);
                    let flipped_x = entity.flipped_x.unwrap_or(false);
                    let flipped_y = entity.flipped_y.unwrap_or(false);
                    entity::create(
                        &dst,
                        entry.value as i32,
                        entity.x as i32,
                        entity.y as i32,
                        w as u32,
                        h as u32,
                        rot as i16,
                        flipped_x,
                        flipped_y,
                    )?;
                }
            }
        }

        // tiles
        let (mut cur_x, mut cur_y) = (0, 0);
        let tile_rows = &layer.data_coords2_d;
        for tile_row in tile_rows.iter() {
            for tile in tile_row.iter() {
                if tile.len().eq(&1) {
                    cur_x += 1;
                    continue;
                }
                for entry in ts_table.iter() {
                    if entry.name.eq(entry.name.as_str()) {
                        tile::new(
                            &dst,
                            entry.value as i32,
                            cur_x,
                            cur_y,
                            tile[0] as u16,
                            tile[1] as u16,
                        )?;
                    }
                }
                cur_x += 1;
            }
            cur_x = 0; // reset x cursor
            cur_y += 1;
        }
    }

    // sync to disk
    Ok(dst.sync_data()?)
}

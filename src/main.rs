mod entity;
mod global;
mod grid;
mod layer;

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use byteorder::{LittleEndian, WriteBytesExt};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "cartographer")]
struct Cli {
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    entity_table: PathBuf,
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
    pub entities: Option<Vec<OgmoEntity>>,
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
    pub width: i64,
    pub height: i64,
    pub origin_x: i64,
    pub origin_y: i64,
    pub rotation: i64,
    pub flipped_x: bool,
    pub flipped_y: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityTable {
    pub entity_table: Vec<EntityTableEntry>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityTableEntry {
    pub name: String,
    pub value: i64,
}

fn main() {
    use std::io::Read;
    use std::io::Write;

    // read cli arguments
    let opt = Cli::from_args();

    // buffered file reading
    let src = File::open(opt.input).unwrap();
    let mut buf = BufReader::new(src);
    let mut con = String::new();

    // marshal into a data structure
    buf.read_to_string(&mut con).unwrap();
    let map: OgmoMap = serde_json::from_str(&con).unwrap();

    // entity table
    let entity_table_file = File::open(opt.entity_table).unwrap();
    let mut entity_table_buf = BufReader::new(entity_table_file);
    let mut entity_table_contents = String::new();
    entity_table_buf
        .read_to_string(&mut entity_table_contents)
        .unwrap();
    let ent_table = serde_json::from_str::<EntityTable>(&entity_table_contents)
        .unwrap()
        .entity_table;

    // open a new file
    let dst = File::create(opt.output).unwrap();

    // write file header
    global::write_header(&dst).unwrap();

    // global level properties
    global::set_width(&dst, map.width).unwrap();
    global::set_height(&dst, map.height).unwrap();

    // iterate through layers
    for layer in map.layers.iter() {
        layer::set_width(&dst, layer.grid_cell_width).unwrap();
        layer::set_height(&dst, layer.grid_cell_height).unwrap();

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

                    grid::cell_set(
                        &dst,
                        cur_x as i16,
                        cur_y as i16,
                        cell.parse::<i8>().unwrap(),
                    )
                    .unwrap();
                }
            } else {
                let (mut cur_x, mut cur_y) = (0, 0);
                let grid = layer.grid2d.as_ref().unwrap();
                // grid rows and cells
                for row in grid.iter() {
                    for cell in row.iter() {
                        grid::cell_set(
                            &dst,
                            cur_x as i16,
                            cur_y as i16,
                            cell.parse::<i8>().unwrap(),
                        )
                        .unwrap();
                        cur_x += 1;
                    }
                    cur_x = 0; // reset x cursor
                    cur_y += 1;
                }
            }
        }

        // entities
        if layer.entities.is_none() {
            continue;
        }
        let entities = layer.entities.as_ref().unwrap();
        for entity in entities.iter() {
            for entry in ent_table.iter() {
                if entity.name.eq(entry.name.as_str()) {
                    entity::create(
                        &dst,
                        entry.value as i32,
                        entity.x as i32,
                        entity.y as i32,
                        entity.width as u32,
                        entity.height as u32,
                        entity.rotation as i16,
                        entity.flipped_x,
                        entity.flipped_y,
                    )
                    .unwrap();
                }
            }
        }
    }

    // sync to disk
    dst.sync_data().unwrap();
}

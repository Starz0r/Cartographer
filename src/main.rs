mod entity;
mod global;
mod grid;
mod layer;
mod ldtk;
mod tile;

use std::{fs::File, io::BufReader, path::PathBuf};

use {
    anyhow::Result,
    hashbrown::HashMap,
    serde::{Deserialize, Serialize},
    structopt::StructOpt,
};

use scopeguard::{defer, defer_on_unwind};

use crate::ldtk::{LdtkJson, Level};

#[derive(StructOpt, Debug)]
#[structopt(name = "cartographer")]
struct Cli {
    #[structopt(short, long, parse(from_os_str))]
    project: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    level: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    #[structopt(long, parse(from_os_str))]
    info_table: PathBuf,
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

    // read cli arguments
    let opt = Cli::from_args();

    // buffered file reading
    let src = File::open(opt.project)?;
    let mut buf = BufReader::new(src);
    let mut con = String::new();

    // marshal into a data structure
    buf.read_to_string(&mut con)?;
    let project: LdtkJson = serde_json::from_str(&con)?;

    let src = File::open(opt.level)?;
    let mut buf = BufReader::new(src);
    let mut con = String::new();
    buf.read_to_string(&mut con)?;
    let map: Level = serde_json::from_str(&con)?;

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
    use std::convert::TryInto;
    global::set_width(
        &dst,
        project
            .default_level_width
            .unwrap_or_else(|| 0)
            .try_into()?,
    )?;
    global::set_height(
        &dst,
        project
            .default_level_height
            .unwrap_or_else(|| 0)
            .try_into()?,
    )?;

    // iterate through layers
    let layers = map.layer_instances.unwrap();
    for layer in layers.iter() {
        // signal layer type
        match layer.layer_instance_type.as_ref() {
            "IntGrid" => {
                layer::set_type(&dst, 1)?;
            }

            "Entities" => {
                layer::set_type(&dst, 3)?;
            }

            &_ => {
                panic!("unsupported layer type")
            }
        }

        // set width and height
        layer::set_width(&dst, layer.grid_size.try_into()?)?;
        layer::set_height(&dst, layer.grid_size.try_into()?)?;

        if layer.layer_instance_type == String::from("IntGrid") {
            // keep track of the y axis manually
            let (mut cur_x, mut cur_y) = (0, 0);
            let grid = &layer.int_grid_csv;
            // grid cells
            for cell in grid.iter() {
                if cur_x == layer.c_wid {
                    cur_x = 0; // reset x cursor
                    cur_y += 1;
                }

                grid::cell_set(&dst, cur_x as i16, cur_y as i16, *cell as i8).unwrap();
            }
        }

        // entities
        let entities = &layer.entity_instances;
        for entity in entities.iter() {
            for entry in ent_table.iter() {
                if entity.name.eq(entry.name.as_str()) {
                    // DANGER: px should always be populated with two entries
                    // NOTE: [x,y] are effected by optional layer offsets
                    // which just aren't taken into account here at all
                    let x = unsafe { entity.px.get_unchecked(0) };
                    let y = unsafe { entity.px.get_unchecked(1) };
                    let w = entity.width;
                    let h = entity.height;
                    let rot = 0;
                    let flipped_x = false;
                    let flipped_y = false;
                    entity::create(
                        &dst,
                        entry.value as i32,
                        *x as i32,
                        *y as i32,
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
        for tile in layer.grid_tiles.iter() {
            for entry in ts_table.iter() {
                // TODO: use tileset_def_uid & override_tileset_uid instead
                let tileset_name = layer.tileset_rel_path.clone().unwrap_or_else(|| String::new());
                if tileset_name.eq(entry.name.as_str()) {
                    // NOTE: same as above, optional layer offsets
                    // aren't taken into account here
                    let x = unsafe { tile.px.get_unchecked(0) };
                    let y = unsafe { tile.px.get_unchecked(1) };
                    let src_x = unsafe { tile.src.get_unchecked(0) };
                    let src_y = unsafe { tile.src.get_unchecked(1) };
                    tile::new(
                        &dst,
                        entry.value as i32,
                        *x as u32,
                        *y as u32,
                        *src_x as u16,
                        *src_y as u16,
                    )?;
                }
            }
        }
    }

    // sync to disk
    Ok(dst.sync_data()?)
}

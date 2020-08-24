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
    pub grid_cell_width: i64,
    pub grid_cell_height: i64,
    pub grid_cells_x: i64,
    pub grid_cells_y: i64,
    #[serde(default)]
    pub grid: Option<Vec<String>>,
    pub array_mode: i64,
    #[serde(default, rename = "grid2D")]
    pub grid2d: Option<Vec<Vec<String>>>,
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

    // open a new file
    let mut dst = File::create(opt.output).unwrap();

    // write file header
    dst.write("LVL Format 0.".as_bytes()).unwrap();

    // sync to disk
    dst.sync_data().unwrap();
}

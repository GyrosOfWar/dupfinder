extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate twox_hash;
extern crate img_hash;
extern crate image;
extern crate rayon;
extern crate serde;
extern crate serde_json;
extern crate crossbeam;
extern crate walkdir;
#[macro_use]
extern crate failure;
extern crate parking_lot;
extern crate byteorder;

use std::io::prelude::*;
use std::fs::File;
use std::path::{PathBuf, Path};

use structopt::StructOpt;

use dupfinder::*;

mod filecmp;
mod dupfinder;

fn normal_output(duplicates: &[Vec<PathBuf>]) -> String {
    let mut t = String::new();
    for set in duplicates {
        let paths: Vec<_> = set.iter().map(|s| s.file_name().unwrap().to_string_lossy()).collect();
        let paths = paths.join(", ");
        t.push_str(&format!("[{}]\n", paths));
    }
    t.pop();
    t
}

// TODO add option for long/short filename output
fn json_output(duplicates: &[Vec<PathBuf>]) -> String {
    let paths: Vec<Vec<_>> = duplicates
        .iter()
        .map(|lst| {
            lst.iter()
                .map(|p| p.file_name().unwrap().to_str().unwrap())
                .collect()
        })
        .collect();

    serde_json::to_string(&paths).unwrap()
}

fn run(config: Config) -> Result<()> {
    let mut df = DuplicateFinder::new(config.clone());
    let path = Path::new(&config.path);
    let duplicates = df.find_duplicates(path)?;
    let output = if config.json {
        json_output(&duplicates)
    } else {
        normal_output(&duplicates)
    };
    if let Some(path) = config.out_path {
        let mut file = File::create(path)?;
        write!(file, "{}", output)?;
    } else {
        println!("{}", output);
    }
    Ok(())
}

fn main() {
    let config = Config::from_args();
    if let Err(e) = run(config) {
        eprintln!("Error: {}", e);
    }
}

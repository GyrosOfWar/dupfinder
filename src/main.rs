#![feature(conservative_impl_trait)]

extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate murmurhash3;
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
    let path = Path::new(&config.path);
    // let hasher = config.method.get_comparer();

    // let duplicates = find_duplicates(&config)?;

    // let output = if config.json {
    //     json_output(&duplicates)
    // } else {
    //     normal_output(&duplicates)
    // };

    // if !config.out_path.is_empty() {
    //     let mut f = File::create(config.out_path).unwrap();
    //     write!(f, "{}", output).unwrap();
    // } else {
    //     println!("{}", output);
    // }
    Ok(())
}

fn main() {
    let config = Config::from_args();
    if let Err(e) = run(config) {
        eprintln!("Error: {}", e);
    }
}

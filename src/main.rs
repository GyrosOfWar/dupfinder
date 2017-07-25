extern crate argparse;
extern crate murmurhash3;
extern crate img_hash;
extern crate image;
extern crate rayon;
extern crate serde;
extern crate serde_json;
extern crate crossbeam;
extern crate walkdir;

use dupfinder::*;
use argparse::{ArgumentParser, Store, StoreTrue};
use std::path::PathBuf;

mod filecmp;
mod dupfinder;

use std::io::prelude::*;
use std::fs::File;

pub fn parse_args() -> Config {
    let mut config = Config {
        verbose: false,
        json: false,
        path: ".".into(),
        method: "mur".into(),
        progressbar: false,
        out_path: "".into(),
        recursive: false,
    };
    {
        let mut parser = ArgumentParser::new();
        parser.set_description("Counts duplicate files in a directory.");
        parser.refer(&mut config.method).add_option(
            &["-m", "--method"],
            Store,
            "Hashing method to be used. Defaults to MurmurHash3. (img, mur, head)",
        );
        parser
            .refer(&mut config.path)
            .add_argument("path", Store, "Path to search");
        parser
            .refer(&mut config.verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "A lot of console output");
        parser
            .refer(&mut config.json)
            .add_option(&["--json"], StoreTrue, "Output as JSON");
        parser
            .refer(&mut config.out_path)
            .add_option(&["-o", "--out"], Store, "Output path");
        parser.refer(&mut config.recursive).add_option(
            &["-r", "--recursive"],
            StoreTrue,
            "Recurse into subdirectories",
        );

        parser.parse_args_or_exit();
    }

    config
}

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
// TODO add option for a tolerance when using image hashing
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

fn main() {
    let config = parse_args();
    let duplicates = find_duplicates(&config).unwrap();

    let output = if config.json {
        json_output(&duplicates)
    } else {
        normal_output(&duplicates)
    };

    if !config.out_path.is_empty() {
        let mut f = File::create(config.out_path).unwrap();
        write!(f, "{}", output).unwrap();
    } else {
        println!("{}", output);
    }
}

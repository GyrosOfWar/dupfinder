#![feature(plugin)]
#![plugin(clippy)]

extern crate pbr;
extern crate argparse;
extern crate murmurhash3;
extern crate img_hash;
extern crate image;
extern crate rayon;
extern crate serde;
extern crate serde_json;

use dupfinder::*;
use argparse::{ArgumentParser, Store, StoreTrue};
use std::path::PathBuf;

mod filecmp;
mod dupfinder;

pub fn parse_args() -> Config {
    let mut config = Config {
        quiet: false,
        verbose: false,
        json: false,
        path: ".".to_owned(),
        method: "md5".to_owned(),
        progressbar: false,
        filter: vec![]
    }; 
    {
        let mut parser = ArgumentParser::new();
        parser.set_description("Counts duplicate files in a directory.");
        parser.refer(&mut config.method)
            .add_option(&["-h", "--hasher"], Store,
                        "Specify hasher to be used. Options are md5, img and head.");
        parser.refer(&mut config.path)
            .add_argument("path", Store, "Path to search");
        parser.refer(&mut config.quiet)
            .add_option(&["-q", "--quiet"], StoreTrue, "No console output");
        parser.refer(&mut config.verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "A lot of console output");
        parser.refer(&mut config.json)
            .add_option(&["--json"], StoreTrue, "Output as JSON");
        // parser.refer(&mut )
        parser.parse_args_or_exit();
    }
    
    config
}

fn normal_output(duplicates: &[Vec<PathBuf>]) -> String {
    let mut t = String::new();
    for set in duplicates {
        let mut s = String::new();
        s.push('[');
        for path in set {
            s.push_str(&format!("{}, ", path.to_string_lossy()));
        }
        s.pop();
        s.pop();
        s.push(']');
        
        t.push_str(&s);
        t.push('\n');
    }
    
    t
}

fn json_output(duplicates: &[Vec<PathBuf>]) -> String {
    serde_json::to_string(&duplicates).unwrap()
}

fn main() {
    let config = parse_args();
    let duplicates = find_duplicates(config.clone()).unwrap();

    if duplicates.is_empty() {
        println!("No duplicates found!");
    }
    else if !config.quiet {
        println!("Found duplicates:");
        if config.json {
            println!("{}", json_output(&duplicates));
        } else {
            println!("{}", normal_output(&duplicates));
        }
    }
}

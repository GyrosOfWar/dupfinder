#![feature(plugin)]
#![plugin(clippy)]

extern crate pbr;
extern crate argparse;
extern crate crypto;
extern crate img_hash;
extern crate image;
extern crate rayon;
extern crate serde;
extern crate serde_json;

use std::path::Path;
use filecmp::*;
use dupfinder::*;
use crypto::md5::Md5;
use img_hash::HashType;
use argparse::{ArgumentParser, Store};

mod filecmp;
mod dupfinder;

pub fn parse_args() -> Config {
    let mut path = ".".to_owned();
    let mut hasher = "md5".to_owned();
    let mut quiet = false;
    let mut verbose = false;
    {
        let mut parser = ArgumentParser::new();
        parser.set_description("Counts duplicate files in a directory.");
        parser.refer(&mut hasher)
            .add_option(&["-h", "--hasher"], Store,
                        "Specify hasher to be used. Options are md5, img and head.");
        parser.refer(&mut path)
            .add_argument("path", Store, "Path to search");
        parser.refer(&mut quiet)
            .add_option(&["-q", "--quiet"], Store, "No console output");
        parser.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], Store, "A lot of console output");
        
        parser.parse_args_or_exit();
    }
    Config {
        quiet: quiet,
        verbose: verbose,
        progressbar: true,
        json: false,
        path: path,
        method: hasher, 
    }
}

fn main() {
    let config = parse_args();
    let duplicates = find_duplicates(config).unwrap();

    if duplicates.is_empty() {
        println!("No duplicates found!");
    }
    else {
        for set in duplicates {
            let mut s = String::new();
            s.push('[');
            for path in set {
                s.push_str(&format!("{}, ", path.to_string_lossy()));
            }
            s.pop();
            s.pop();
            s.push(']');

            println!("{}", s);
        }
    }
}

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

use std::path::{Path, PathBuf};
use filecmp::*;
use dupfinder::*;
use crypto::md5::Md5;
use img_hash::HashType;
use argparse::{ArgumentParser, Store};

mod filecmp;
mod dupfinder;

fn main() {
    let mut path = ".".to_owned();
    let mut hasher = "md5".to_owned();
    {
        let mut parser = ArgumentParser::new();
        parser.set_description("Counts duplicate files in a directory.");
        parser.refer(&mut hasher)
            .add_option(&["-h", "--hasher"], Store,
                        "Specify hasher to be used. Options are md5, img and head.");
        parser.refer(&mut path)
            .add_argument("path", Store, "Path to search");

        parser.parse_args_or_exit();
    }
    let p = Path::new(&path);
    let tmp: &str = &hasher;
    let duplicates = match tmp {
        "md5" => {
            let hasher = DigestFileComparer::new(Md5::new());
            let mut dupfinder = DuplicateFinder::new(hasher);
            dupfinder.find_duplicates(&p)
        },
        "img" => {
            let hasher = ImgHashFileComparer::new(8, HashType::Gradient);
            let mut dupfinder = DuplicateFinder::new(hasher);
            dupfinder.find_duplicates(&p)
        },
        "head" => {
            let hasher = FileHeadComparer::new(16);
            let mut dupfinder = DuplicateFinder::new(hasher);
            dupfinder.find_duplicates(&p)
        }
        _ => panic!()
    }.unwrap();

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

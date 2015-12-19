extern crate pbr;
extern crate argparse;
extern crate crypto;
extern crate img_hash;
extern crate image;

use std::path::Path;
use filehasher::*;
use dupfinder::*;
use crypto::md5::Md5;
use img_hash::HashType;
use argparse::{ArgumentParser, Store};

pub mod filehasher;
pub mod dupfinder;

fn main() {
    let mut path = ".".to_string();
    let mut hasher = "md5".to_string();
    {
        let mut parser = ArgumentParser::new();
        parser.set_description("Counts duplicate files in a directory.");
        parser.refer(&mut hasher)
            .add_option(&["-h", "--hasher"], Store,
                        "Specify hasher to be used. Options are md5, img and head.");

        parser.refer(&mut path).add_argument("path", Store, "Path to search");
        parser.parse_args_or_exit();
    }
    let p = Path::new(&path);
    let tmp: &str = &hasher;
    let duplicates = match tmp {
        "md5" => {
            let hasher = DigestFileHasher::new(Md5::new());
            let mut dupfinder = DuplicateFinder::new(hasher);
            dupfinder.find_duplicates(&p)
        },
        "img" => {
            let hasher = ImgFileHasher::new(8, HashType::Gradient);
            let mut dupfinder = DuplicateFinder::new(hasher);
            dupfinder.find_duplicates(&p)
        },
        "head" => {
            let hasher = FileHeadHasher::new(16);
            let mut dupfinder = DuplicateFinder::new(hasher);
            dupfinder.find_duplicates(&p)
        }
        _ => panic!()
    }.unwrap();

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

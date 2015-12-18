extern crate pbr;
extern crate argparse;
extern crate crypto;
extern crate img_hash;
extern crate image;

use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use std::io;
use std::fs::File;
use std::collections::HashMap;
use crypto::md5::Md5;
use crypto::sha2::Sha256;
use crypto::digest::Digest;
use img_hash::{ImageHash, HashType};

pub trait DuplicateFinder {
    fn find_duplicates(&mut self, folder: &Path) -> Vec<Vec<PathBuf>>;
}

fn hash_file<D: Digest>(p: &Path, h: &mut D) -> io::Result<Vec<u8>> {
    let mut f = try!(File::open(p));
    let mut content = vec![];
    f.read_to_end(&mut content).expect("Error reading file");
    h.input(&content);
    let result = h.result_str();
    h.reset();
    Ok(result.into_bytes())
}

pub struct HashDuplicateFinder<D> {
    hashes: HashMap<Vec<u8>, PathBuf>,
    hash_maker: D
}

impl<D> HashDuplicateFinder<D> {
    pub fn new(h: D) -> HashDuplicateFinder<D> {
        HashDuplicateFinder {
            hashes: HashMap::new(),
            hash_maker: h
        }
    }
}

impl<D> DuplicateFinder for HashDuplicateFinder<D> where D: Digest {
    fn find_duplicates(&mut self, folder: &Path) -> Vec<Vec<PathBuf>> {
        let mut duplicates = vec![];

        for file in fs::read_dir(folder).unwrap() {
            let file = file.unwrap();
            let path = file.path();

            if path.is_dir() {
                continue;
            }

            let hash = match hash_file(&path, &mut self.hash_maker) {
                Ok(r) => r,
                Err(why) => {
                    // TODO add silent/verbose flags
                    println!("Error reading file: {:?}: {}", file.path(), why);
                    continue;
                }
            };
            if self.hashes.contains_key(&hash) {
                duplicates.push(vec![path, self.hashes[&hash].clone()]);
            } else {
                self.hashes.insert(hash, path);
            }
        }

        duplicates
    }
}

fn read_file_head(p: &Path, len: usize) -> io::Result<Vec<u8>> {
    let f = try!(File::open(p));
    Ok(f.bytes()
        .map(|r| r.unwrap())
        .take(len)
        .collect())
}

pub struct FileHeadDuplicateFinder {
    head_len: usize,
    file_heads: HashMap<Vec<u8>, PathBuf>
}

impl FileHeadDuplicateFinder {
    pub fn new(head_len: usize) -> FileHeadDuplicateFinder {
        FileHeadDuplicateFinder {
             head_len: head_len,
             file_heads: HashMap::new()
        }
    }
}

// Lots of duplicate code here
impl DuplicateFinder for FileHeadDuplicateFinder {
    fn find_duplicates(&mut self, folder: &Path) -> Vec<Vec<PathBuf>> {
        let mut results = vec![];
        // TODO add progress bar
        for file in fs::read_dir(folder).unwrap() {
            let file = file.unwrap();
            let path = file.path();

            if path.is_dir() {
                continue;
            }
            let head = match read_file_head(&path, self.head_len) {
                Ok(r) => r,
                Err(why) => {
                    println!("Error reading file {:?}: {}", file.path(), why);
                    continue;
                }
            };

            if self.file_heads.contains_key(&head) {
                results.push(vec![path, self.file_heads[&head].clone()]);
            } else {
                self.file_heads.insert(head, path);
            }
        }

        results
    }
}

pub struct ImgHashDuplicateFinder {
    hashes: HashMap<ImageHash, PathBuf>,
    hash_size: u32,
    hash_type: HashType,
}

impl DuplicateFinder for ImgHashDuplicateFinder {
    fn find_duplicates(&mut self, folder: &Path) -> Vec<Vec<PathBuf>> {
        let mut results = vec![];
        for file in fs::read_dir(folder).unwrap() {
            let file = file.unwrap();
            let path = file.path();

            if path.is_dir() {
                continue;
            }
            let image = match image::open(&path) {
                Ok(r) => r,
                Err(why) => {
                    println!("Error reading file: {:?}: {}", path, why);
                    continue;
                }
            };
            let hash = ImageHash::hash(&image, self.hash_size, self.hash_type);
            if self.hashes.contains_key(&hash) {
                 results.push(vec![path, self.hashes[&hash].clone()]);
            } else {
                self.hashes.insert(hash, path);
            }
        }
        results
    }
}

pub fn find_duplicates_md5(folder: &Path) -> Vec<Vec<PathBuf>> {
    let mut dup_finder = HashDuplicateFinder::new(Md5::new());
    dup_finder.find_duplicates(folder)
}

pub fn find_duplicates_sha256(folder: &Path) -> Vec<Vec<PathBuf>> {
    let mut dup_finder = HashDuplicateFinder::new(Sha256::new());
    dup_finder.find_duplicates(folder)
}

pub fn find_duplicates_img_hash(folder: &Path) -> Vec<Vec<PathBuf>> {
    let mut dup_finder = ImgHashDuplicateFinder {
        hashes: HashMap::new(),
        hash_size: 8,
        hash_type: HashType::Gradient
    };

    dup_finder.find_duplicates(folder)
}

fn main() {
    let arg = env::args().nth(1).expect("Expected commandline argument (folder)");
    let directory = Path::new(&arg);
    let duplicates = find_duplicates_img_hash(directory);
    for set in duplicates {
        println!("{:?}", set);
    }
}

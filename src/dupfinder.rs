use std::{fs, io};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::str::FromStr;

use img_hash::HashType;
use rayon::prelude::*;
use walkdir::WalkDir;
use failure::{Error};
use parking_lot::Mutex;

use filecmp::*;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum HashAlgorithm {
    MurmurHash,
    ImageHash,
    FileHead
}

impl FromStr for HashAlgorithm {
    type Err = Error;

    fn from_str(input: &str) -> Result<HashAlgorithm> {
        match input {
            "mur" => Ok(HashAlgorithm::MurmurHash),
            "img" => Ok(HashAlgorithm::ImageHash),
            "head" => Ok(HashAlgorithm::FileHead),
            other => bail!("Unknown error type: {}", other)
        }
    }
}

#[derive(Debug, Clone, StructOpt)]
pub struct Config {
    #[structopt(
        short = "v",
        long = "verbose",
        help = "More verbose output."
    )]
    pub verbose: bool,
    #[structopt(
        short = "p",
        long = "progress",
        help = "Show a progress bar."
    )]
    pub progressbar: bool,
    #[structopt(
        long = "json",
        help = "Output data as JSON."
    )]
    pub json: bool,
    #[structopt(
        name = "INPUT",
        help = "Input path."
    )]
    pub path: String,
    #[structopt(
        short = "m",
        long = "method",
        help = "Hashing algorithm to use"
    )]
    pub method: HashAlgorithm,
    pub out_path: Option<String>,
    pub recursive: bool,
}

pub struct DuplicateFinder<H> {
    hasher: H,
    config: Config,
}

fn collect_files(folder: &Path, recursive: bool) -> io::Result<Vec<PathBuf>> {
    if recursive {
        let wd = WalkDir::new(folder);
        let mut files = vec![];

        for f in wd {
            files.push(try!(f).path().into());
        }

        Ok(files)
    } else {
        let files = try!(fs::read_dir(folder));
        files.map(|f| f.and_then(|g| Ok(g.path()))).collect()
    }
}

impl<H, K> DuplicateFinder<H>
where
    H: FileComparer<V = K>,
    K: Hash + Eq + Send + Sync,
{
    pub fn new(hasher: H, config: Config) -> DuplicateFinder<H> {
        DuplicateFinder {
            hasher: hasher,
            config: config,
        }
    }

    pub fn find_duplicates(&mut self, folder: &Path) -> Result<Vec<Vec<PathBuf>>> {
        let mut dup_vec = vec![];
        let files = try!(collect_files(folder, self.config.recursive));
        let file_hashes = files.into_par_iter().map(|path| {
            let mut h = self.hasher.clone();
            if self.config.verbose {
                println!("Hashing file {:?}", path.file_name());
            }
            h.hash_file(path.clone()).and_then(|h| Ok((path, h)))
        });
        let data = HashMap::new();
        let duplicates = Arc::new(Mutex::new(data));
        file_hashes.for_each(|res| if let Ok((path, hash)) = res {
            let mut map = duplicates.lock();
            match map.entry(hash) {
                Entry::Occupied(ref mut e) => {
                    let p: &mut Vec<PathBuf> = e.get_mut();
                    p.push(path);
                }
                Entry::Vacant(e) => {
                    e.insert(vec![path]);
                }
            }
        });

        let dups = duplicates.lock();
        for (_, paths) in dups.iter() {
            if paths.len() > 1 {
                dup_vec.push(paths.clone());
            }
        }

        Ok(dup_vec)
    }
}

// pub fn find_duplicates(config: &Config) -> Result {
//     let path = Path::new(&config.path);
//     let method: &str = &config.method;
//     match method {
//         "mur" => {
//             let hasher = HashComparer;
//             let mut df = DuplicateFinder::new(hasher, config.clone());
//             df.find_duplicates(path)
//         }        
//         "img" => {
//             let hasher = ImgHashFileComparer::new(8, HashType::Gradient);
//             let mut df = DuplicateFinder::new(hasher, config.clone());
//             df.find_duplicates(path)
//         }
//         "head" => {
//             let hasher = FileHeadComparer::new(16);
//             let mut df = DuplicateFinder::new(hasher, config.clone());
//             df.find_duplicates(path)
//         }
//         _ => Err(format!("No such hashing method: {}", method).into()),
//     }
// }

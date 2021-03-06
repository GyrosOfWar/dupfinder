use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::{fs, io};

use crate::error::Result;
use crate::filecmp::*;
use failure::Error;
use img_hash::HashType;
use parking_lot::Mutex;
use rayon::prelude::*;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub enum HashAlgorithm {
    XxHash(HashComparer),
    ImageHash(ImgHashFileComparer),
}

impl FileComparer for HashAlgorithm {
    fn hash_file<P>(&mut self, path: P, buf: &mut Vec<u8>) -> Result<()>
    where
        P: AsRef<Path>,
    {
        match *self {
            HashAlgorithm::XxHash(ref mut hasher) => hasher.hash_file(path, buf),
            HashAlgorithm::ImageHash(ref mut hasher) => hasher.hash_file(path, buf),
        }
    }
}

impl FromStr for HashAlgorithm {
    type Err = Error;

    fn from_str(input: &str) -> Result<HashAlgorithm> {
        match input {
            "xxh" => Ok(HashAlgorithm::XxHash(HashComparer)),
            "img" => Ok(HashAlgorithm::ImageHash(ImgHashFileComparer::new(
                8,
                HashType::Gradient,
            ))),
            other => bail!("Unknown error type: {}", other),
        }
    }
}

#[derive(Debug, Clone, StructOpt)]
pub struct Config {
    #[structopt(short = "v", long = "verbose", help = "More verbose output.")]
    pub verbose: bool,

    #[structopt(short = "p", long = "progress", help = "Show a progress bar.")]
    pub progressbar: bool,

    #[structopt(long = "json", help = "Output data as JSON.")]
    pub json: bool,

    #[structopt(name = "INPUT", help = "Input path.")]
    pub path: String,

    #[structopt(short = "m", long = "method", help = "Hashing algorithm to use")]
    pub method: HashAlgorithm,

    pub out_path: Option<String>,

    pub recursive: bool,
}

pub struct DuplicateFinder {
    config: Config,
}

fn collect_files(folder: &Path, recursive: bool) -> io::Result<Vec<PathBuf>> {
    if recursive {
        let wd = WalkDir::new(folder);
        let mut files = vec![];

        for f in wd {
            files.push(r#try!(f).path().into());
        }

        Ok(files)
    } else {
        let files = r#try!(fs::read_dir(folder));
        files.map(|f| f.and_then(|g| Ok(g.path()))).collect()
    }
}

impl DuplicateFinder {
    pub fn new(config: Config) -> Self {
        DuplicateFinder { config }
    }

    pub fn find_duplicates(&mut self, folder: &Path) -> Result<Vec<Vec<PathBuf>>> {
        let mut dup_vec = vec![];
        let files = r#try!(collect_files(folder, self.config.recursive));
        let file_hashes = files.into_par_iter().map(|path| {
            let mut h = self.config.method.clone();
            let mut buf = vec![];
            if self.config.verbose {
                println!("Hashing file {:?}", path.file_name());
            }
            h.hash_file(path.clone(), &mut buf)
                .and_then(|_| Ok((path, buf)))
        });
        let data = HashMap::new();
        let duplicates = Arc::new(Mutex::new(data));
        file_hashes.for_each(|res| {
            if let Ok((path, hash)) = res {
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

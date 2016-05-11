use std::hash::Hash;
use filecmp::*;
use std::collections::HashMap;
use std::{io, fs};
use std::path::{PathBuf, Path};
use std::collections::hash_map::Entry;
use img_hash::HashType;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

pub type FinderResult = Result<Vec<Vec<PathBuf>>, Error>;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    UnknownMethod,
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub verbose: bool,
    pub progressbar: bool,
    pub json: bool,
    pub path: String,
    pub method: String,
    pub out_path: String,
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
    where H: FileComparer<V = K>,
          K: Hash + Eq + Send + Sync
{
    pub fn new(hasher: H, config: Config) -> DuplicateFinder<H> {
        DuplicateFinder {
            hasher: hasher,
            config: config,
        }
    }

    pub fn find_duplicates(&mut self, folder: &Path) -> FinderResult {
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
        file_hashes.for_each(|res| {
            if let Ok((path, hash)) = res {
                let mut map = duplicates.lock().unwrap();
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

        let dups = duplicates.lock().unwrap();
        for (_, paths) in dups.iter() {
            if paths.len() > 1 {
                dup_vec.push(paths.clone());
            }
        }

        Ok(dup_vec)
    }
}

pub fn find_duplicates(config: Config) -> FinderResult {
    let path = Path::new(&config.path);
    let method: &str = &config.method;
    match method {
        "mur" => {
            let hasher = HashComparer;
            let mut df = DuplicateFinder::new(hasher, config.clone());
            df.find_duplicates(path)
        }        
        "img" => {
            let hasher = ImgHashFileComparer::new(8, HashType::Gradient);
            let mut df = DuplicateFinder::new(hasher, config.clone());
            df.find_duplicates(path)
        }
        "head" => {
            let hasher = FileHeadComparer::new(16);
            let mut df = DuplicateFinder::new(hasher, config.clone());
            df.find_duplicates(path)
        }
        _ => Err(Error::UnknownMethod),
    }
}

pub enum DeletionStrategy {
    Oldest,
    Newest,
    Biggest,
    Smallest,
}


pub fn delete_duplicates(duplicates: FinderResult,
                         strategy: DeletionStrategy)
                         -> io::Result<Vec<PathBuf>> {
    unimplemented!()
}

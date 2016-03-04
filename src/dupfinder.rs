use std::hash::Hash;
use filecmp::*;
use std::collections::HashMap;
use std::io;
use std::fs;
use std::path::{PathBuf, Path};
use std::collections::hash_map::Entry;
use pbr::ProgressBar;
use crypto::md5::Md5;
use img_hash::HashType;

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

pub struct Config {
    quiet: bool,
    verbose: bool,
    progressbar: bool,
    json: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            quiet: false,
            verbose: false,
            progressbar: true,
            json: false
        }
    }
}

pub struct DuplicateFinder<H, K: Hash + Eq> {
    hasher: H,
    hashes: HashMap<K, PathBuf>,
    config: Config,
    progressbar: Option<ProgressBar>
}

impl<H, K> DuplicateFinder<H, K> where H: FileComparer<V = K>, K: Hash + Eq {
    pub fn new(hasher: H) -> DuplicateFinder<H, K> {
        DuplicateFinder {
            hasher: hasher,
            hashes: HashMap::new(),
            config: Default::default(),
            progressbar: None
        }
    }

    pub fn find_duplicates<P>(&mut self, folder: P) -> FinderResult where P: AsRef<Path> + Clone {
        let mut duplicates = vec![];
        if self.config.progressbar {
            let count = try!(fs::read_dir(folder.clone())).count();
            self.progressbar = Some(ProgressBar::new(count));
        }
        
        for file in try!(fs::read_dir(folder)) {
            let file = try!(file);
            let path = file.path();

            // TODO implement FileFilter or something
            if path.is_dir() {
                continue;
            }

            let hash = match self.hasher.hash_file(&path) {
                Ok(h) => h,
                Err(why) => {
                    if !self.config.quiet {
                        println!("Error reading file: {:?}: {}", file.path(), why);
                    }
                    continue;
                }
            };

            match self.hashes.entry(hash) {
                Entry::Occupied(e) => {
                    duplicates.push(vec![path, e.get().clone()]);
                }
                Entry::Vacant(e) => {
                    e.insert(path);
                }
            }
            
            if let Some(ref mut pb) = self.progressbar {
                pb.inc();
            }
        }

        Ok(duplicates)
    }
}

pub fn find_duplicates<P>(path: P, method: &str) -> FinderResult where P: AsRef<Path> {
    match method {
        "md5" => {
            let hasher = DigestFileComparer::new(Md5::new());
            let mut df = DuplicateFinder::new(hasher);
            df.find_duplicates(path)
        },        
        "img" => {
            let hasher = ImgHashFileComparer::new(8, HashType::Gradient);
            let mut dupfinder = DuplicateFinder::new(hasher);
            dupfinder.find_duplicates(path)
        },
        "head" => {
            let hasher = FileHeadComparer::new(16);
            let mut dupfinder = DuplicateFinder::new(hasher);
            dupfinder.find_duplicates(path)
        }
        _ => Err(Error::UnknownMethod)
    }
}
use std::hash::Hash;
use filecmp::*;
use std::collections::HashMap;
use std::{io, fs, thread};
use std::path::{PathBuf, Path};
use std::collections::hash_map::Entry;
use pbr::ProgressBar;
use img_hash::HashType;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use crossbeam::sync::MsQueue;
use crossbeam::scope;

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
    pub quiet: bool,
    pub verbose: bool,
    pub progressbar: bool,
    pub json: bool,
    pub path: String,
    pub method: String,
    pub filter: Vec<String>,
}

pub struct DuplicateFinder<H> {
    hasher: H,
    config: Config,
}

fn collect_files(folder: &Path) -> io::Result<Vec<PathBuf>> {
    let files = try!(fs::read_dir(folder));
    files.map(|f| f.and_then(|g| Ok(g.path()))).collect()
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
        let mut ddupp = vec![];
        let files = try!(collect_files(folder));
        let count = files.len();
        let queue = MsQueue::new();
        scope(|s| {
            s.spawn(|| {
                let mut pb = ProgressBar::new(count as u64);
                loop {
                    println!("Thread!");
                    if queue.pop() {
                        pb.inc();
                    } else {
                        break;
                    }
                }
            })
        });
        let file_hashes = files.into_par_iter().map(|path| {
            let mut h = self.hasher.clone();
            h.hash_file(path.clone()).and_then(|h| Ok((path, h)))
        });
        let data = HashMap::new();
        let duplicates = Arc::new(Mutex::new(data));
        file_hashes.for_each(|res| {
            let (path, hash) = res.unwrap();
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

            queue.push(true);
        });

        let dups = duplicates.lock().unwrap();
        for (_, paths) in dups.iter() {
            if paths.len() > 1 {
                ddupp.push(paths.clone());
            }
        }
        queue.push(false);

        Ok(ddupp)
    }
}

pub fn find_duplicates(config: Config) -> FinderResult {
    let path = Path::new(&config.path);
    let method: &str = &config.method;
    match method {
        "mr3" => {
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

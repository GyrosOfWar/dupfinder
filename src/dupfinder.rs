use std::hash::Hash;
use filehasher::FileHasher;
use std::collections::HashMap;
use std::io;
use std::fs;
use std::path::{PathBuf, Path};
use std::collections::hash_map::Entry;
use pbr::ProgressBar;

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

impl<H, K> DuplicateFinder<H, K> where H: FileHasher<V = K>, K: Hash + Eq {
    pub fn new(hasher: H) -> DuplicateFinder<H, K> {
        DuplicateFinder {
            hasher: hasher,
            hashes: HashMap::new(),
            config: Default::default(),
            progressbar: None
        }
    }

    pub fn find_duplicates(&mut self, folder: &Path) -> io::Result<Vec<Vec<PathBuf>>> {
        let mut duplicates = vec![];
        if self.config.progressbar {
            let count = try!(fs::read_dir(folder)).count();
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

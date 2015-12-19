use std::hash::Hash;
use filehasher::FileHasher;
use std::collections::HashMap;
use std::io;
use std::fs;
use std::path::{PathBuf, Path};

pub struct DuplicateFinder<H, K: Hash + Eq> {
	hasher: H,
	hashes: HashMap<K, PathBuf>
}

impl<H, K> DuplicateFinder<H, K> where H: FileHasher<V = K>, K: Hash + Eq {
	pub fn new(hasher: H) -> DuplicateFinder<H, K> {
		DuplicateFinder {
			hasher: hasher,
			hashes: HashMap::new()
		}
	}

	pub fn find_duplicates(&mut self, folder: &Path) -> io::Result<Vec<Vec<PathBuf>>> {
		let mut duplicates = vec![];

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

		Ok(duplicates)
	}
}

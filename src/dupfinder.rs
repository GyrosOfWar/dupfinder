use std::hash::Hash;
use filehasher::FileHasher;
use std::collections::HashMap;
use std::path::{PathBuf, Path};

pub struct DuplicateFinder<H, K: Hash+Eq> {
	hasher: H, 
	hashes: HashMap<K, PathBuf>
}

impl<H, K> DuplicateFinder<H, K> where H: FileHasher, K: Hash + Eq {
	pub fn new(hasher: H) -> DuplicateFinder<H, K> {
		DuplicateFinder { 
			hasher: hasher,
			hashes: HashMap::new()
		}
	}
	
	pub fn find_duplicates(&mut self, folder: &Path) -> Vec<Vec<PathBuf>> {
		let mut duplicates = vec![];
		
		
		
		duplicates
	}
	
}
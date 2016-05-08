use std::path::Path;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use image;
use img_hash::{ImageHash, HashType};
use std::hash::Hash;
use murmurhash3::murmurhash3_x64_128;

pub trait FileComparer: Sync + Clone {
    type V: Hash + Eq;
    fn hash_file<P>(&mut self, path: P) -> io::Result<Self::V> where P: AsRef<Path>;
}

#[derive(Clone)]
pub struct HashComparer;

impl FileComparer for HashComparer {
    type V = (u64, u64);

    fn hash_file<P>(&mut self, path: P) -> io::Result<Self::V>
        where P: AsRef<Path>
    {
        const SEED: u64 = 0x12345678;

        let mut f = try!(File::open(path));
        let mut content = vec![];
        try!(f.read_to_end(&mut content));
        Ok(murmurhash3_x64_128(&content, SEED))
    }
}


#[derive(Clone)]
pub struct FileHeadComparer {
    head_len: usize,
}

impl FileHeadComparer {
    pub fn new(head_len: usize) -> FileHeadComparer {
        FileHeadComparer { head_len: head_len }
    }
}

impl FileComparer for FileHeadComparer {
    type V = Vec<u8>;

    fn hash_file<P>(&mut self, path: P) -> io::Result<Vec<u8>>
        where P: AsRef<Path>
    {
        let f = try!(File::open(path));
        let mut result = vec![];

        for r in f.bytes().take(self.head_len) {
            let b = try!(r);
            result.push(b);
        }

        Ok(result)
    }
}

#[derive(Clone)]
pub struct ImgHashFileComparer {
    hash_size: u32,
    hash_type: HashType,
}

impl ImgHashFileComparer {
    pub fn new(hash_size: u32, hash_type: HashType) -> ImgHashFileComparer {
        ImgHashFileComparer {
            hash_size: hash_size,
            hash_type: hash_type,
        }
    }
}

impl FileComparer for ImgHashFileComparer {
    type V = ImageHash;

    fn hash_file<P>(&mut self, path: P) -> io::Result<ImageHash>
        where P: AsRef<Path>
    {
        // FIXME use map_err
        let image = image::open(path).unwrap();
        let hash = ImageHash::hash(&image, self.hash_size, self.hash_type);
        Ok(hash)
    }
}

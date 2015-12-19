use std::path::Path;
use crypto::digest::Digest;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use image;
use img_hash::{ImageHash, HashType};
use std::hash::Hash;

pub trait FileHasher {
    type V: Hash + Eq;
    fn hash_file(&mut self, path: &Path) -> io::Result<Self::V>;
}

pub struct DigestFileHasher<D> {
    digest: D
}

impl<D> DigestFileHasher<D> where D: Digest {
    pub fn new(digest: D) -> DigestFileHasher<D> {
        DigestFileHasher { digest: digest }
    }
}

impl<D> FileHasher for DigestFileHasher<D> where D: Digest {
    type V = Vec<u8>;

    fn hash_file(&mut self, path: &Path) -> io::Result<Vec<u8>> {
        let mut f = try!(File::open(path));
        let mut content = vec![];
        try!(f.read_to_end(&mut content));
        self.digest.input(&content);
        let result = self.digest.result_str();
        self.digest.reset();
        Ok(result.into_bytes())
    }
}

pub struct FileHeadHasher {
    head_len: usize
}

impl FileHeadHasher {
    pub fn new(head_len: usize) -> FileHeadHasher {
        FileHeadHasher { head_len: head_len }
    }
}

impl FileHasher for FileHeadHasher {
    type V = Vec<u8>;

    fn hash_file(&mut self, path: &Path) -> io::Result<Vec<u8>> {
        let f = try!(File::open(path));
        let mut result = vec![];

        for r in f.bytes().take(self.head_len) {
            let b = try!(r);
            result.push(b);
        }

        Ok(result)
    }
}

pub struct ImgFileHasher {
    hash_size: u32,
    hash_type: HashType
}

impl ImgFileHasher {
    pub fn new(hash_size: u32, hash_type: HashType) -> ImgFileHasher {
        ImgFileHasher {
            hash_size: hash_size,
            hash_type: hash_type
        }
    }
}

impl FileHasher for ImgFileHasher {
    type V = ImageHash;

    fn hash_file(&mut self, path: &Path) -> io::Result<ImageHash> {
        // FIXME
        let image = image::open(path).unwrap();
        let hash = ImageHash::hash(&image, self.hash_size, self.hash_type);

        Ok(hash)
    }
}

use std::path::Path;
use crypto::digest::Digest;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use image;
use img_hash::{ImageHash, HashType};
use std::hash::Hash;

pub trait FileComparer {
    type V: Hash + Eq;
    fn hash_file<P>(&mut self, path: P) -> io::Result<Self::V> where P: AsRef<Path>;
}

pub struct DigestFileComparer<D> {
    digest: D
}

impl<D> DigestFileComparer<D> where D: Digest {
    pub fn new(digest: D) -> DigestFileComparer<D> {
        DigestFileComparer { digest: digest }
    }
}

impl<D> FileComparer for DigestFileComparer<D> where D: Digest {
    type V = Vec<u8>;

    fn hash_file<P>(&mut self, path: P) -> io::Result<Vec<u8>> where P: AsRef<Path> {
        let mut f = try!(File::open(path));
        let mut content = vec![];
        try!(f.read_to_end(&mut content));
        self.digest.input(&content);
        let result = self.digest.result_str();
        self.digest.reset();
        Ok(result.into_bytes())
    }
}

pub struct FileHeadComparer {
    head_len: usize
}

impl FileHeadComparer {
    pub fn new(head_len: usize) -> FileHeadComparer {
        FileHeadComparer { head_len: head_len }
    }
}

impl FileComparer for FileHeadComparer {
    type V = Vec<u8>;

    fn hash_file<P>(&mut self, path: P) -> io::Result<Vec<u8>> where P: AsRef<Path> {
        let f = try!(File::open(path));
        let mut result = vec![];

        for r in f.bytes().take(self.head_len) {
            let b = try!(r);
            result.push(b);
        }

        Ok(result)
    }
}

pub struct ImgHashFileComparer {
    hash_size: u32,
    hash_type: HashType
}

impl ImgHashFileComparer {
    pub fn new(hash_size: u32, hash_type: HashType) -> ImgHashFileComparer {
        ImgHashFileComparer {
            hash_size: hash_size,
            hash_type: hash_type
        }
    }
}

impl FileComparer for ImgHashFileComparer {
    type V = ImageHash;

    fn hash_file<P>(&mut self, path: P) -> io::Result<ImageHash> where P: AsRef<Path> {
        // FIXME use map_err 
        let image = image::open(path).unwrap();
        let hash = ImageHash::hash(&image, self.hash_size, self.hash_type);
        Ok(hash)
    }
}

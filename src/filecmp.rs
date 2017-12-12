use std::path::Path;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::hash::Hasher;

use byteorder::{WriteBytesExt, LE};
use image;
use img_hash::{HashType, ImageHash};
use twox_hash;

pub trait FileComparer: Sync + Clone {
    fn hash_file<P>(&mut self, path: P, buf: &mut Vec<u8>) -> io::Result<()>
    where
        P: AsRef<Path>;
}

#[derive(Clone, Debug)]
pub struct HashComparer;

impl FileComparer for HashComparer {
    fn hash_file<P>(&mut self, path: P, buf: &mut Vec<u8>) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        const SEED: u64 = 0x12345678;

        let mut hasher = twox_hash::XxHash::with_seed(SEED);

        let mut file = io::BufReader::new(File::open(path)?);
        let mut file_buf = [0; 1024 * 8];
        loop {
            match file.read(&mut file_buf) {
                Ok(0) => break,
                Ok(amt) => hasher.write(&file_buf[0..amt]),
                Err(e) => return Err(e)
            }
        }
        let result = hasher.finish();
        buf.write_u64::<LE>(result)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ImgHashFileComparer {
    hash_size: u32,
    hash_type: HashType,
}

impl ImgHashFileComparer {
    pub fn new(hash_size: u32, hash_type: HashType) -> ImgHashFileComparer {
        ImgHashFileComparer {
            hash_size,
            hash_type,
        }
    }
}

impl FileComparer for ImgHashFileComparer {
    fn hash_file<P>(&mut self, path: P, buf: &mut Vec<u8>) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        // FIXME use map_err
        let image = image::open(path).unwrap();
        let hash = ImageHash::hash(&image, self.hash_size, self.hash_type);
        let bits = hash.to_bytes();
        buf.write_all(&bits)?;
        Ok(())
    }
}

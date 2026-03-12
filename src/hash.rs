use anyhow::Result;
use log::debug;
use md5hash::MD5Hasher;
use std::{fs::File, io::Read, path::PathBuf};

const HASH_BYTES: usize = 10 * 1024 * 1024; // 10MB

pub fn calculate(movie_file: &PathBuf) -> Result<String> {
    let mut f = File::open(movie_file)?;
    let mut buf = vec![0u8; HASH_BYTES];
    let bytes_read = f.read(&mut buf)?;
    debug!("bytes read: {}", bytes_read);

    let mut h = MD5Hasher::new();
    h.digest(&buf);
    Ok(format!("{:x}", h.finish()))
}

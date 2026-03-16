use anyhow::{Context, Result, bail};
use log::debug;
use md5hash::MD5Hasher;
use std::{fs::File, io::Read, path::PathBuf};

const HASH_BYTES: usize = 10 * 1024 * 1024; // 10MB

#[derive(Debug, Clone)]
pub struct Md5Hex(String);

impl Md5Hex {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for Md5Hex {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        if value.len() != 32 {
            bail!("md5 hex must be 32 characters");
        }
        if !value.chars().all(|c| c.is_ascii_hexdigit()) {
            bail!("md5 hex contains non-hex characters");
        }
        Ok(Md5Hex(value.to_string()))
    }
}

pub fn compute_md5(movie_file: &PathBuf) -> Result<Md5Hex> {
    let mut f = File::open(movie_file)?;
    let mut buf = vec![0u8; HASH_BYTES];
    let bytes_read = f.read(&mut buf)?;
    debug!("bytes read: {}", bytes_read);

    let mut h = MD5Hasher::new();
    h.digest(&buf);
    Md5Hex::try_from(format!("{:x}", h.finish()).as_str())
}

pub fn compute_token(z: &Md5Hex) -> Result<String> {
    let add: [usize; 5] = [0, 0xd, 0x10, 0xb, 0x5];
    let mul: [u32; 5] = [2, 2, 5, 4, 3];
    let idx: [usize; 5] = [0xe, 0x3, 0x6, 0x8, 0x2];

    let mut out = String::with_capacity(idx.len());
    for i in 0..idx.len() {
        let a = add[i];
        let m = mul[i];
        let i = idx[i];

        let zi = z
            .as_str()
            .get(i..i + 1)
            .and_then(|s| s.chars().next())
            .context("hash has unexpected length or encoding")?;
        let t = a + zi.to_digit(16).context("hash contains non-hex characters")? as usize;

        let hex = z
            .as_str()
            .get(t..t + 2)
            .context("hash has unexpected length for token derivation")?;
        let v = u32::from_str_radix(hex, 16).context("hash contains non-hex characters")?;
        let digit = (v * m) & 0xf;

        let ch = std::char::from_digit(digit, 16).context("failed to format token digit")?;
        out.push(ch);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    // use super::*;

    use crate::hash::{Md5Hex, compute_token};

    #[test]
    fn test_known_token() {
        let h1 = Md5Hex::try_from("e17ef434e816db49e58b062b45e3e258").unwrap();
        let h2 = Md5Hex::try_from("4b3d32b7700b3588531dd81db058eba9").unwrap();
        assert_eq!(compute_token(&h1).unwrap(), "8c081");
        assert_eq!(compute_token(&h2).unwrap(), "00640");
    }
}

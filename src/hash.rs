use anyhow::{Context, Result, bail};
use log::debug;
use md5hash::MD5Hasher;
use std::{fs::File, io::Read, path::PathBuf};

const HASH_BYTES: usize = 10 * 1024 * 1024; // 10MB

pub fn compute_md5(movie_file: &PathBuf) -> Result<String> {
    let mut f = File::open(movie_file)?;
    let mut buf = vec![0u8; HASH_BYTES];
    let bytes_read = f.read(&mut buf)?;
    debug!("bytes read: {}", bytes_read);

    let mut h = MD5Hasher::new();
    h.digest(&buf);
    Ok(format!("{:x}", h.finish()))
}

pub fn compute_token(z: &str) -> Result<String> {
    if z.len() < 32 {
        bail!("hash too short for token generation");
    }

    let add: [usize; 5] = [0, 0xd, 0x10, 0xb, 0x5];
    let mul: [u32; 5] = [2, 2, 5, 4, 3];
    let idx: [usize; 5] = [0xe, 0x3, 0x6, 0x8, 0x2];

    let mut out = String::with_capacity(idx.len());
    for i in 0..idx.len() {
        let a = add[i];
        let m = mul[i];
        let i = idx[i];

        let zi = z
            .get(i..i + 1)
            .and_then(|s| s.chars().next())
            .context("hash has unexpected length or encoding")?;
        let t = a + zi.to_digit(16).context("hash contains non-hex characters")? as usize;

        let hex = z
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

    use crate::hash::compute_token;

    #[test]
    fn test_known_token() {
        assert_eq!(compute_token("e17ef434e816db49e58b062b45e3e258").unwrap(), "8c081");
        assert_eq!(compute_token("4b3d32b7700b3588531dd81db058eba9").unwrap(), "00640");
    }
}

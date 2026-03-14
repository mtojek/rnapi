use std::io::{BufReader, Cursor};

use anyhow::{Result, bail};
use log::debug;
use sevenz_rust2::{ArchiveReader, Password};
use ureq::Error;

pub fn download(checksum: &str, token: &str) -> Result<Vec<u8>> {
    let url = format!(
        "https://napiprojekt.pl/unit_napisy/dl.php?l=PL&f={checksum}&t={token}&v=other&kolejka=false&nick=&pass=&napios=posix"
    );
    debug!("URL: {}", url);

    match ureq::get(url).call() {
        Ok(mut response) => {
            let buf = response.body_mut().read_to_vec()?;
            if buf.starts_with(b"NPc0") {
                bail!("Subtitles not found (NPc0).")
            }
            Ok(buf)
        }
        Err(Error::StatusCode(code)) => {
            if code == 404 {
                bail!("Subtitles not found.")
            }
            bail!("Unexpected status code: {}", code);
        }
        Err(err) => {
            bail!("Unexpected error: {}", err);
        }
    }
}

const PASSWORD: &str = "iBlm8NTigvru0Jr0";

pub fn decompress(data: Vec<u8>) -> Result<Vec<u8>> {
    let cursor = Cursor::new(data);
    let reader = BufReader::new(cursor);
    let mut archive = ArchiveReader::new(reader, PASSWORD.into())?;

    let mut decompressed = vec![];
    archive
        .for_each_entries(|_entry, reader| {
            reader.read_to_end(&mut decompressed).unwrap();
            Ok(true)
        })
        .unwrap();
    Ok(decompressed)
}

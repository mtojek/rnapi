use std::{
    fs,
    io::{BufReader, Cursor},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use log::{debug, info};
use sevenz_rust2::ArchiveReader;
use ureq::Error;

const PASSWORD: &str = "iBlm8NTigvru0Jr0";
const PREVIEW_LINES: usize = 10;

pub fn download(checksum: &str, token: &str) -> Result<Vec<u8>> {
    let url = format!(
        "https://napiprojekt.pl/unit_napisy/dl.php?l=PL&f={checksum}&t={token}&v=other&kolejka=false&nick=&pass=&napios=posix"
    );
    info!("Download subtitles file: {}", url);

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

pub fn decompress(data: Vec<u8>) -> Result<Vec<u8>> {
    let cursor = Cursor::new(data);
    let reader = BufReader::new(cursor);
    let mut archive = ArchiveReader::new(reader, PASSWORD.into())?;

    let mut decompressed = vec![];
    archive.for_each_entries(|_entry, reader| {
        reader.read_to_end(&mut decompressed)?;
        Ok(true)
    })?;
    Ok(decompressed)
}

pub fn preview(data: &[u8]) {
    data.splitn(PREVIEW_LINES + 1, |b| *b == b'\n')
        .take(PREVIEW_LINES)
        .for_each(|line| {
            debug!("{}", String::from_utf8_lossy(&line));
        });
}

pub fn write_out<P: AsRef<Path>>(movie_file: P, content: &[u8]) -> Result<PathBuf> {
    let mut sub_path = movie_file.as_ref().to_path_buf();
    sub_path.set_extension("srt");

    fs::write(&sub_path, content)
        .with_context(|| format!("can't write subtitles file: {:?}", sub_path))?;

    info!("Subtitles file written: {}", sub_path.display());
    Ok(sub_path)
}

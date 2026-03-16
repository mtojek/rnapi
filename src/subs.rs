use std::{
    ffi::OsStr,
    io::{BufReader, Cursor},
};

use anyhow::{Result, bail};
use log::debug;
use sevenz_rust2::ArchiveReader;
use subparse::{SrtFile, SubtitleFileInterface, SubtitleFormat, get_subtitle_format, parse_str};
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

const PREVIEW_LINES: usize = 10;

pub fn preview(data: &[u8]) {
    data.splitn(PREVIEW_LINES + 1, |b| *b == b'\n')
        .take(PREVIEW_LINES)
        .for_each(|line| {
            debug!("{}", String::from_utf8_lossy(&line));
        });
}

pub fn to_srt(content: &[u8], fps: f64) -> Vec<u8> {
    let format = get_subtitle_format(Some(OsStr::new("sub")), content) // FIXME sub
        .expect("unrecognized subtitle format");
    debug!("Subtitle extension detected: {:?}", format);

    if format == SubtitleFormat::SubRip {
        return content.to_vec();
    }

    debug!("Parse subtitles");
    let subtitle_file =
        parse_str(format, str::from_utf8(content).unwrap(), fps).expect("can't parse subtitles");
    let entries = subtitle_file
        .get_subtitle_entries()
        .expect("can't read subtitle entries");

    debug!("Write SubRip file");
    let lines = entries
        .into_iter()
        .map(|entry| (entry.timespan, entry.line.unwrap_or_default()))
        .collect();
    let subrip = SrtFile::create(lines).expect("can't build SubRip file");
    subrip.to_data().expect("can't serialize SubRip file")
}

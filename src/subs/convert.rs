use anyhow::{Context, Result};
use log::{debug, info};
use subparse::{SrtFile, SubtitleFileInterface, SubtitleFormat, parse_str};

pub fn to_srt(content: &[u8], fps: f64) -> Result<Vec<u8>> {
    let format = detect_subtitle_format(content)
        .context("unrecognized subtitle format")?;
    debug!("Subtitle format detected: {:?}", format);

    if format == SubtitleFormat::SubRip {
        info!("Already a SubRip file");
        return Ok(content.to_vec());
    }

    debug!("Parse subtitles");
    let text = std::str::from_utf8(content).context("subtitle content is not valid UTF-8")?;
    let subtitle_file = parse_str(format, text, fps)
        .map_err(|e| anyhow::anyhow!("can't parse subtitles: {e}"))?;
    let entries = subtitle_file
        .get_subtitle_entries()
        .map_err(|e| anyhow::anyhow!("can't read subtitle entries: {e}"))?;

    debug!("Serialize SubRip file");
    let lines = entries
        .into_iter()
        .map(|entry| (entry.timespan, entry.line.unwrap_or_default()))
        .collect();
    let subrip = SrtFile::create(lines)
        .map_err(|e| anyhow::anyhow!("can't build SubRip file: {e}"))?;
    let serialized = subrip
        .to_data()
        .map_err(|e| anyhow::anyhow!("can't serialize SubRip file: {e}"))?;

    info!("Converted to SubRip");
    Ok(serialized)
}

fn detect_subtitle_format(content: &[u8]) -> Option<SubtitleFormat> {
    let text = match std::str::from_utf8(content) {
        Ok(s) => s,
        Err(_) => return None,
    };

    if looks_like_srt(text) {
        return Some(SubtitleFormat::SubRip);
    }
    if looks_like_ass(text) {
        return Some(SubtitleFormat::SubStationAlpha);
    }
    if looks_like_idx(text) {
        return Some(SubtitleFormat::VobSubIdx);
    }
    if looks_like_microdvd(text) {
        return Some(SubtitleFormat::MicroDVD);
    }
    None
}

fn looks_like_srt(text: &str) -> bool {
    text.lines()
        .any(|line| line.contains(" --> ") && line.contains(','))
}

fn looks_like_ass(text: &str) -> bool {
    text.contains("[Script Info]") || text.lines().any(|line| line.starts_with("Dialogue:"))
}

fn looks_like_idx(text: &str) -> bool {
    text.lines().any(|line| line.contains("timestamp:") && line.contains("filepos:"))
        || text.lines().any(|line| line.starts_with("VobSub index file"))
        || text.lines().any(|line| line.starts_with("id: "))
}

fn looks_like_microdvd(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim_start();
        line.starts_with('{')
            && line.contains("}{")
            && line
                .chars()
                .take_while(|c| *c != '}')
                .skip(1)
                .all(|c| c.is_ascii_digit())
    })
}

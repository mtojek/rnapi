use anyhow::{Context, Result};
use log::{debug, info};
use subparse::{
    SrtFile,
    SubtitleFileInterface,
    SubtitleFormat,
    parse_str,
    timetypes::{TimePoint, TimeSpan},
};

pub fn to_srt(content: &[u8], fps: f64) -> Result<Vec<u8>> {
    let text = std::str::from_utf8(content).context("subtitle content is not valid UTF-8")?;
    if looks_like_mpl2(text) {
        return parse_mpl2_to_srt(text);
    }

    let format = detect_subtitle_format(content)
        .context("unrecognized subtitle format")?;
    debug!("Subtitle format detected: {:?}", format);

    if format == SubtitleFormat::SubRip {
        info!("Already a SubRip file");
        return Ok(content.to_vec());
    }

    debug!("Parse subtitles");
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

fn looks_like_mpl2(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim_start();
        line.starts_with('[')
            && line.contains("][")
            && line
                .chars()
                .take_while(|c| *c != ']')
                .skip(1)
                .all(|c| c.is_ascii_digit())
    })
}

fn parse_mpl2_to_srt(text: &str) -> Result<Vec<u8>> {
    let mut lines = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || !line.starts_with('[') {
            continue;
        }

        let (start, end, payload) = parse_mpl2_line(line)
            .map_err(|e| anyhow::anyhow!("invalid MPL2 line '{line}': {e}"))?;

        let start_ms = start * 100;
        let end_ms = end * 100;
        let text = payload.replace('|', "\n");
        lines.push((
            TimeSpan::new(
                TimePoint::from_msecs(start_ms as i64),
                TimePoint::from_msecs(end_ms as i64),
            ),
            text,
        ));
    }

    let subrip = SrtFile::create(lines)
        .map_err(|e| anyhow::anyhow!("can't build SubRip file: {e}"))?;
    let serialized = subrip
        .to_data()
        .map_err(|e| anyhow::anyhow!("can't serialize SubRip file: {e}"))?;
    Ok(serialized)
}

fn parse_mpl2_line(line: &str) -> Result<(i64, i64, &str)> {
    let mut i = 0usize;
    let bytes = line.as_bytes();

    if bytes.get(i) != Some(&b'[') {
        anyhow::bail!("missing '['");
    }
    i += 1;
    let end1 = line[i..].find(']').context("missing ']' for start")? + i;
    let start_str = &line[i..end1];
    let start = start_str.parse::<i64>().context("invalid start time")?;

    i = end1 + 1;
    if bytes.get(i) != Some(&b'[') {
        anyhow::bail!("missing second '['");
    }
    i += 1;
    let end2 = line[i..].find(']').context("missing ']' for end")? + i;
    let end_str = &line[i..end2];
    let end = end_str.parse::<i64>().context("invalid end time")?;

    let payload = line.get(end2 + 1..).unwrap_or("");
    Ok((start, end, payload))
}

#[cfg(test)]
mod tests {
    use super::{
        parse_mpl2_to_srt,
        detect_subtitle_format,
    };
    use subparse::SubtitleFormat;

    #[test]
    fn detect_microdvd_curly_format() {
        let input = "{497}{536}{y:b}..:: ZABOJCZA BRON ::..";
        let format = detect_subtitle_format(input.as_bytes());
        assert_eq!(format, Some(SubtitleFormat::MicroDVD));
    }

    #[test]
    fn parse_mpl2_to_srt_converts_times() {
        let input = "[497][536]Hello";
        let srt = String::from_utf8(parse_mpl2_to_srt(input).unwrap()).unwrap();
        assert!(srt.contains("00:00:49,700 --> 00:00:53,600"));
    }

    #[test]
    fn parse_mpl2_to_srt_preserves_pipe_newlines() {
        let input = "[1][2]A|B";
        let srt = String::from_utf8(parse_mpl2_to_srt(input).unwrap()).unwrap();
        assert!(srt.contains("A\nB"));
    }
}

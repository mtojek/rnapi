# rnapi

CLI that downloads Napiprojekt subtitles for a given movie file, converts them to SubRip (`.srt`), and writes the result next to the movie.

## Features

- Computes movie hash and token for Napiprojekt
- Downloads and extracts subtitles (7z)
- Converts common subtitle formats to SRT
- Writes `<movie_name>.srt` in the same directory

## Requirements

- Rust toolchain (stable)
- Network access (Napiprojekt)
- A local movie file

## Usage

```bash
cargo run -- /path/to/movie.mkv
```

For MicroDVD (`.sub`) inputs, FPS is required for correct timing conversion. Default is `23.976`:

```bash
cargo run -- /path/to/movie.mkv --fps 25
```

The output file will be written as:

```
/path/to/movie.srt
```

## Logging

Default log level is `INFO`. Use `RUST_LOG` to change it:

```bash
RUST_LOG=debug cargo run -- /path/to/movie.mkv
```

## Notes / Limitations

- Subtitle encoding is currently assumed to be Windows-1250 and is converted to UTF-8.
- Format detection is based on content heuristics and supports SubRip, ASS/SSA, MicroDVD, and VobSub IDX text formats.
- If multiple files are present in the downloaded archive, all entries are currently concatenated during decompression.

## License

See `LICENSE`.

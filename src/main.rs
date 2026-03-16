use anyhow::{Result, bail};
use clap::Parser;
use env_logger::Env;
use log::{debug, error};
use std::path::PathBuf;

mod encoding;
mod hash;
mod subs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 23.976)]
    fps: f64,

    movie_file: PathBuf,
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    if let Err(err) = run(&args) {
        error!("{err}");
        std::process::exit(1);
    }
}

fn run(args: &Args) -> Result<()> {
    validate(&args)?;

    let h = hash::compute_md5(&args.movie_file)?;
    debug!("movie file hash = {}", &h);

    let t = hash::compute_token(&h);
    debug!("token = {}", &t);

    let f = subs::download(&h, &t)?;
    debug!("subtitle archive size = {}", f.len());

    let s = subs::decompress(f)?;
    debug!("Original:");
    subs::preview(&s);

    let encoded = encoding::to_utf8(&s);
    debug!("UTF-8 encoded:");
    subs::preview(&encoded);

    let converted = subs::to_srt(&encoded, args.fps);

    debug!("SubRip format:");
    subs::preview(&converted);

    subs::write_out(&args.movie_file, &converted);
    Ok(())
}

fn validate(args: &Args) -> Result<()> {
    if args.fps <= 0.0 {
        bail!("fps must be greater than 0");
    }

    if !args.movie_file.exists() {
        bail!("movie file does not exist");
    }

    debug!("Validation passed");
    Ok(())
}

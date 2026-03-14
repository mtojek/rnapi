use anyhow::{Result, bail};
use clap::Parser;
use log::{debug, error};
use std::path::PathBuf;

mod hash;
mod subs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 23.976)]
    fps: f32,

    movie_file: PathBuf,
}

fn main() {
    env_logger::init(); // RUST_LOG=debug

    let args = Args::parse();
    if let Err(err) = run(&args) {
        error!("{err}");
        std::process::exit(1);
    }
}

fn run(args: &Args) -> Result<()> {
    validate(&args)?;
    debug!("Validation passed");

    let h = hash::compute_md5(&args.movie_file)?;
    debug!("movie file hash = {}", &h);

    let t = hash::compute_token(&h);
    debug!("token = {}", &t);

    let f = subs::download(&h, &t)?;
    debug!("subtitle archive size = {}", f.len());

    let s = subs::decompress(f)?;
    debug!("{}", String::from_utf8_lossy(&s));

    Ok(())
}

fn validate(args: &Args) -> Result<()> {
    if args.fps <= 0.0 {
        bail!("fps must be greater than 0");
    }

    if !args.movie_file.exists() {
        bail!("movie file does not exist");
    }
    Ok(())
}

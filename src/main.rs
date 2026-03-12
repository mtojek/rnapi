use anyhow::{Result, bail};
use clap::Parser;
use log::error;
use std::path::PathBuf;

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

    println!("Hello, {}", args.movie_file.display());
}

fn run(args: &Args) -> Result<()> {
    validate(&args)?;
    println!("Hello, {}", args.movie_file.display());
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

use std::path::PathBuf;
use clap::Parser;

/// Nesium TUI Frontend
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the NES ROM file
    #[arg(required = true)]
    pub rom: PathBuf,

    /// Disable audio
    #[arg(long)]
    pub no_audio: bool,

    /// Force integer FPS (60Hz) pacing
    #[arg(long)]
    pub integer_fps: bool,
}

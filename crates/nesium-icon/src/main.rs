use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Icon generation utility for Nesium.
#[derive(Parser, Debug)]
#[command(name = "nesium-icon")]
#[command(about = "Generate composite and layered (bg/fg) icons", long_about = None)]
struct Cli {
    /// Output path for the composite icon (used when no subcommand is specified)
    #[arg(short, long)]
    out: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Output background + foreground layer PNGs (useful for adaptive/layered icons)
    Layers {
        /// Background output path
        #[arg(long, default_value = "icon_bg_1024.png")]
        bg: PathBuf,

        /// Foreground output path
        #[arg(long, default_value = "icon_fg_1024.png")]
        fg: PathBuf,

        /// Output size (e.g. 512). Defaults to the crate's DEFAULT_ICON_SIZE.
        #[arg(long)]
        size: Option<u32>,
    },
    /// Output the icon as an SVG file
    Svg {
        /// Output path
        #[arg(long, default_value = "icon.svg")]
        out: PathBuf,
    },
}

fn main() -> Result<(), String> {
    // Examples:
    //   cargo run -p nesium-icon --
    //   cargo run -p nesium-icon -- --out foo.png
    //   cargo run -p nesium-icon -- layers
    //   cargo run -p nesium-icon -- layers --size 512 --bg bg.png --fg fg.png

    let cli = Cli::parse();

    match cli.command {
        Some(Command::Layers { bg, fg, size }) => {
            if let Some(s) = size {
                nesium_icon::render_layer_pngs_sized(
                    bg.to_string_lossy().as_ref(),
                    fg.to_string_lossy().as_ref(),
                    s,
                )
            } else {
                nesium_icon::render_layer_pngs(
                    bg.to_string_lossy().as_ref(),
                    fg.to_string_lossy().as_ref(),
                )
            }
        }
        Some(Command::Svg { out }) => nesium_icon::render_svg(out.to_string_lossy().as_ref()),
        None => {
            let out_path = cli.out.unwrap_or_else(|| PathBuf::from("icon_1024.png"));
            nesium_icon::render_png(out_path.to_string_lossy().as_ref())
        }
    }
}

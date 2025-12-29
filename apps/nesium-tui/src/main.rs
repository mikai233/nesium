mod app;
mod args;
mod input;
mod ui;

use anyhow::Result;
use clap::Parser;

use crate::{app::App, args::Args};

fn main() -> Result<()> {
    let args = Args::parse();
    let mut app = App::new(args)?;
    app.run()
}

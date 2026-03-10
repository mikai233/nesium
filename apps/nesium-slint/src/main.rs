mod app;
mod runtime;
mod video;

slint::include_modules!();

fn main() -> anyhow::Result<()> {
    app::run()
}

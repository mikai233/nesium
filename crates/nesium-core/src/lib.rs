pub mod apu;
pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod ppu;

#[cfg(test)]
mod tests {
    use ctor::ctor;
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    pub(crate) const TEST_COUNT: usize = 1000;

    #[ctor]
    fn init_tracing() {
        let subscriber = FmtSubscriber::builder()
            .with_file(true)
            .with_line_number(true)
            .with_max_level(Level::INFO)
            .pretty()
            .finish();
        tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
    }
}

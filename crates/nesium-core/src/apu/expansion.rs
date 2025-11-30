use core::fmt::Debug;

/// Optional expansion audio interface implemented by certain cartridge boards.
///
/// Boards such as VRC6/VRC7, Sunsoft 5B, MMC5, Namco 163, or FDS provide extra
/// sound generators that are mixed alongside the core APU channels.
///
/// By default these methods are no-ops / silent so that mappers can opt-in to
/// expansion audio simply by providing an empty `impl ExpansionAudio` block.
///
/// TODO: Revisit the exact clocking source used for `clock_audio`. At the
/// moment expansion audio is driven once per CPU bus access via
/// `Cartridge::cpu_clock`, but some boards may more closely track the APU
/// frame sequencer or a raw CPU-cycle (M2) clock. Once timing is validated
/// against dedicated expansion-audio test ROMs, this trait may grow more
/// precise documentation or additional helpers.
pub trait ExpansionAudio: Debug + Send {
    /// Advance the expansion audio state by one CPU cycle.
    fn clock_audio(&mut self) {}

    /// Current expansion audio sample in linear amplitude space.
    ///
    /// The value is expected to be in a reasonable range (e.g. `0.0..=1.0`);
    /// the mixer may apply additional scaling when combining it with the core
    /// APU output.
    fn sample(&self) -> f32 {
        0.0
    }
}

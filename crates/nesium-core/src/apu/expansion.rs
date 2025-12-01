use core::fmt::Debug;

/// Per-chip expansion audio samples produced by cartridge mappers.
///
/// Each field represents the instantaneous linear amplitude for a given
/// expansion audio source. Most boards will only ever drive a single field.
#[derive(Debug, Default, Clone, Copy)]
pub struct ExpansionSamples {
    pub fds: f32,
    pub mmc5: f32,
    pub namco163: f32,
    pub sunsoft5b: f32,
    pub vrc6: f32,
    pub vrc7: f32,
}

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

    /// Current expansion audio samples in linear amplitude space.
    ///
    /// Each implementation should populate the field corresponding to the
    /// chip it models (e.g. `namco163` for Namco 163); the mixer applies
    /// appropriate per-chip scaling when combining with the core APU output.
    fn samples(&self) -> ExpansionSamples {
        ExpansionSamples::default()
    }
}

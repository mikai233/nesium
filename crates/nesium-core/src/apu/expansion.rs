use core::fmt::Debug;

/// Optional expansion audio interface implemented by certain cartridge boards.
///
/// Boards such as VRC6/VRC7, Sunsoft 5B, MMC5, or FDS provide extra sound
/// generators that are mixed alongside the core APU channels. Implementors are
/// expected to advance their internal state once per CPU cycle and expose a
/// linear sample that the mixer can combine with the base APU output.
pub trait ExpansionAudio: Debug + Send {
    /// Advance the expansion audio state by one CPU cycle.
    fn clock_audio(&mut self);

    /// Current expansion audio sample in linear amplitude space.
    ///
    /// The value is expected to be in a reasonable range (e.g. `0.0..=1.0`);
    /// the mixer may apply additional scaling when combining it with the core
    /// APU output.
    fn sample(&self) -> f32;
}

use core::fmt::Debug;

use crate::audio::{AudioChannel, NesSoundMixer};

/// Immutable timing information for one expansion-audio CPU clock tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExpansionAudioClockContext {
    /// CPU cycle counter in the same clock domain used by the mixer.
    pub cpu_cycle: u64,
    /// Mesen-aligned APU mixer timestamp for this expansion-audio tick.
    ///
    /// Mesen clocks mapper expansion audio before the APU advances its own
    /// cycle counter, so expansion deltas are tagged with the APU's
    /// pre-step cycle index rather than the just-begun CPU bus cycle.
    pub apu_cycle: u64,
    /// Master clock value (12 master clocks per CPU cycle on NTSC).
    pub master_clock: u64,
}

/// Debug/inspection snapshot of current expansion-audio levels.
#[derive(Debug, Default, Clone, Copy)]
pub struct ExpansionAudioSnapshot {
    pub fds: f32,
    pub mmc5: f32,
    pub namco163: f32,
    pub sunsoft5b: f32,
    pub vrc6: f32,
    pub vrc7: f32,
}

/// Delta sink used by mapper-side expansion chips to feed the core mixer.
pub trait ExpansionAudioSink {
    fn push_delta(&mut self, channel: AudioChannel, cpu_cycle: u64, delta: f32);
}

impl ExpansionAudioSink for NesSoundMixer {
    #[inline]
    fn push_delta(&mut self, channel: AudioChannel, cpu_cycle: u64, delta: f32) {
        self.add_delta(channel, cpu_cycle as i64, delta);
    }
}

/// No-op sink for stepping without audio emission.
#[derive(Debug, Default)]
pub struct NullExpansionAudioSink;

impl ExpansionAudioSink for NullExpansionAudioSink {
    #[inline]
    fn push_delta(&mut self, _channel: AudioChannel, _cpu_cycle: u64, _delta: f32) {}
}

/// Mapper-provided expansion audio backend.
///
/// Implementations own chip-specific state and emit per-channel deltas to the
/// mixer via [`ExpansionAudioSink`].
pub trait ExpansionAudio: Debug + Send {
    /// Advance chip state by one CPU cycle and emit any output deltas.
    fn clock_cpu(&mut self, _ctx: ExpansionAudioClockContext, _sink: &mut dyn ExpansionAudioSink) {}

    /// Optional state snapshot used by debug/introspection APIs.
    fn snapshot(&self) -> ExpansionAudioSnapshot {
        ExpansionAudioSnapshot::default()
    }
}

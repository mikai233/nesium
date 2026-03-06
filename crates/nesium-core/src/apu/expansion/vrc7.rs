use crate::apu::expansion::{
    ExpansionAudio, ExpansionAudioClockContext, ExpansionAudioSink, ExpansionAudioSnapshot,
};

/// Phase 1 scaffold for the reusable VRC7 audio chip.
///
/// The full OPLL implementation is intentionally deferred to the later mapper85
/// phase. This type exists now so the expansion-audio module layout is stable
/// before the Konami-family mapper refactor begins.
#[derive(Debug, Clone)]
pub struct Vrc7Audio {
    register_select: u8,
    registers: [u8; 0x40],
    muted: bool,
}

impl Vrc7Audio {
    pub fn new() -> Self {
        Self {
            register_select: 0,
            registers: [0; 0x40],
            muted: false,
        }
    }

    pub fn reset(&mut self) {
        self.register_select = 0;
        self.registers.fill(0);
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    pub fn write_register_select(&mut self, value: u8) {
        if self.muted {
            return;
        }
        self.register_select = value & 0x3F;
    }

    pub fn write_register_data(&mut self, value: u8) {
        if self.muted {
            return;
        }
        self.registers[self.register_select as usize] = value;
    }
}

impl Default for Vrc7Audio {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpansionAudio for Vrc7Audio {
    fn clock_cpu(&mut self, _ctx: ExpansionAudioClockContext, _sink: &mut dyn ExpansionAudioSink) {}

    fn snapshot(&self) -> ExpansionAudioSnapshot {
        ExpansionAudioSnapshot::default()
    }
}

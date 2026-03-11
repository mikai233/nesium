use crate::{
    apu::expansion::{
        ExpansionAudio, ExpansionAudioClockContext, ExpansionAudioSink, ExpansionAudioSnapshot,
    },
    audio::{AudioChannel, CPU_CLOCK_NTSC},
};

const VRC7_OPLL_SAMPLE_RATE: u32 = 49_716;
const VRC7_OPLL_CLOCK_RATE: u32 = VRC7_OPLL_SAMPLE_RATE * 72;
const VRC7_PATCH_TYPE: u8 = 1;
const VRC7_CHIP_TYPE: u8 = 1;
const VRC7_REGISTER_COUNT: usize = 0x40;

fn is_vrc7_mirrored_alias(reg: u8) -> bool {
    matches!(reg, 0x19..=0x1F | 0x29..=0x2F | 0x39..=0x3F)
}

fn canonicalize_vrc7_reg(reg: u8) -> Option<u8> {
    if reg >= VRC7_REGISTER_COUNT as u8 {
        return None;
    }

    Some(if is_vrc7_mirrored_alias(reg) {
        reg - 9
    } else {
        reg
    })
}

#[cfg(nesium_has_vrc7_native)]
mod native {
    use super::{
        AudioChannel, CPU_CLOCK_NTSC, ExpansionAudio, ExpansionAudioClockContext,
        ExpansionAudioSink, ExpansionAudioSnapshot, VRC7_CHIP_TYPE, VRC7_OPLL_CLOCK_RATE,
        VRC7_OPLL_SAMPLE_RATE, VRC7_PATCH_TYPE, VRC7_REGISTER_COUNT, canonicalize_vrc7_reg,
        is_vrc7_mirrored_alias,
    };
    use core::ffi::c_void;

    #[repr(C)]
    struct OpllOpaque(c_void);

    unsafe extern "C" {
        fn nesium_vrc7_opll_new(clk: u32, rate: u32) -> *mut OpllOpaque;
        fn nesium_vrc7_opll_delete(opll: *mut OpllOpaque);
        fn nesium_vrc7_opll_reset(opll: *mut OpllOpaque);
        fn nesium_vrc7_opll_reset_patch(opll: *mut OpllOpaque, patch_type: u8);
        fn nesium_vrc7_opll_set_chip_type(opll: *mut OpllOpaque, chip_type: u8);
        fn nesium_vrc7_opll_write_reg(opll: *mut OpllOpaque, reg: u32, value: u8);
        fn nesium_vrc7_opll_calc(opll: *mut OpllOpaque) -> i16;
    }

    #[derive(Debug)]
    struct OpllHandle(*mut OpllOpaque);

    unsafe impl Send for OpllHandle {}

    impl OpllHandle {
        fn new() -> Self {
            let opll = unsafe { nesium_vrc7_opll_new(VRC7_OPLL_CLOCK_RATE, VRC7_OPLL_SAMPLE_RATE) };
            assert!(!opll.is_null(), "failed to create VRC7 OPLL");
            unsafe {
                nesium_vrc7_opll_set_chip_type(opll, VRC7_CHIP_TYPE);
                nesium_vrc7_opll_reset_patch(opll, VRC7_PATCH_TYPE);
                nesium_vrc7_opll_reset(opll);
            }
            Self(opll)
        }

        fn reset(&mut self) {
            unsafe {
                nesium_vrc7_opll_reset(self.0);
            }
        }

        fn write_reg(&mut self, reg: u8, value: u8) {
            unsafe {
                nesium_vrc7_opll_write_reg(self.0, reg as u32, value);
            }
        }

        fn calc(&mut self) -> i16 {
            unsafe { nesium_vrc7_opll_calc(self.0) }
        }
    }

    impl Drop for OpllHandle {
        fn drop(&mut self) {
            unsafe {
                nesium_vrc7_opll_delete(self.0);
            }
        }
    }

    #[derive(Debug)]
    pub struct Vrc7Audio {
        opll: OpllHandle,
        register_select: u8,
        registers: [u8; VRC7_REGISTER_COUNT],
        previous_output: i16,
        clock_timer: f64,
        muted: bool,
    }

    impl Clone for Vrc7Audio {
        fn clone(&self) -> Self {
            let mut cloned = Self::new();
            cloned.register_select = self.register_select;
            cloned.registers = self.registers;
            cloned.previous_output = self.previous_output;
            cloned.clock_timer = self.clock_timer;
            cloned.muted = self.muted;
            for (reg, value) in self.registers.iter().copied().enumerate() {
                if !is_vrc7_mirrored_alias(reg as u8) {
                    cloned.opll.write_reg(reg as u8, value);
                }
            }
            cloned
        }
    }

    impl Vrc7Audio {
        pub fn new() -> Self {
            Self {
                opll: OpllHandle::new(),
                register_select: 0,
                registers: [0; VRC7_REGISTER_COUNT],
                previous_output: 0,
                clock_timer: 0.0,
                muted: false,
            }
        }

        pub fn reset(&mut self) {
            self.opll.reset();
            self.register_select = 0;
            self.registers.fill(0);
            self.previous_output = 0;
            self.clock_timer = 0.0;
        }

        pub fn set_muted(&mut self, muted: bool) {
            self.muted = muted;
        }

        pub fn write_register_select(&mut self, value: u8) {
            if self.muted {
                return;
            }
            self.register_select = value;
        }

        pub fn write_register_data(&mut self, value: u8) {
            if self.muted {
                return;
            }
            if let Some(reg) = canonicalize_vrc7_reg(self.register_select) {
                self.registers[reg as usize] = value;
            }
            self.opll.write_reg(self.register_select, value);
        }
    }

    impl Default for Vrc7Audio {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ExpansionAudio for Vrc7Audio {
        fn clock_cpu(
            &mut self,
            ctx: ExpansionAudioClockContext,
            sink: &mut dyn ExpansionAudioSink,
        ) {
            if self.clock_timer == 0.0 {
                self.clock_timer = (CPU_CLOCK_NTSC as f64) / (VRC7_OPLL_SAMPLE_RATE as f64);
            }

            self.clock_timer -= 1.0;
            if self.clock_timer <= 0.0 {
                let output = self.opll.calc();
                if !self.muted {
                    let delta = (output - self.previous_output) as f32;
                    if delta != 0.0 {
                        sink.push_delta(AudioChannel::Vrc7, ctx.apu_cycle, delta);
                    }
                }
                self.previous_output = output;
                self.clock_timer = (CPU_CLOCK_NTSC as f64) / (VRC7_OPLL_SAMPLE_RATE as f64);
            }
        }

        fn snapshot(&self) -> ExpansionAudioSnapshot {
            ExpansionAudioSnapshot {
                vrc7: if self.muted {
                    0.0
                } else {
                    self.previous_output as f32
                },
                ..ExpansionAudioSnapshot::default()
            }
        }
    }
}

#[cfg(not(nesium_has_vrc7_native))]
mod fallback {
    use super::{
        ExpansionAudio, ExpansionAudioClockContext, ExpansionAudioSink, ExpansionAudioSnapshot,
        VRC7_REGISTER_COUNT, canonicalize_vrc7_reg,
    };

    #[derive(Debug, Clone)]
    pub struct Vrc7Audio {
        register_select: u8,
        registers: [u8; VRC7_REGISTER_COUNT],
        muted: bool,
    }

    impl Vrc7Audio {
        pub fn new() -> Self {
            Self {
                register_select: 0,
                registers: [0; VRC7_REGISTER_COUNT],
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
            self.register_select = value;
        }

        pub fn write_register_data(&mut self, value: u8) {
            if self.muted {
                return;
            }
            if let Some(reg) = canonicalize_vrc7_reg(self.register_select) {
                self.registers[reg as usize] = value;
            }
        }
    }

    impl Default for Vrc7Audio {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ExpansionAudio for Vrc7Audio {
        fn clock_cpu(
            &mut self,
            _ctx: ExpansionAudioClockContext,
            _sink: &mut dyn ExpansionAudioSink,
        ) {
        }

        fn snapshot(&self) -> ExpansionAudioSnapshot {
            ExpansionAudioSnapshot::default()
        }
    }
}

#[cfg(not(nesium_has_vrc7_native))]
pub use fallback::Vrc7Audio;
#[cfg(nesium_has_vrc7_native)]
pub use native::Vrc7Audio;

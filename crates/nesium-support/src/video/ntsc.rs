use core::ffi::{c_int, c_long, c_uchar, c_ulong, c_void};
use std::mem::MaybeUninit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NesNtscPreset {
    Composite,
    SVideo,
    Rgb,
    Monochrome,
}

impl NesNtscPreset {
    pub(crate) fn setup(self) -> &'static nes_ntsc_setup_t {
        unsafe {
            match self {
                NesNtscPreset::Composite => &nes_ntsc_composite,
                NesNtscPreset::SVideo => &nes_ntsc_svideo,
                NesNtscPreset::Rgb => &nes_ntsc_rgb,
                NesNtscPreset::Monochrome => &nes_ntsc_monochrome,
            }
        }
    }
}

pub const fn nes_ntsc_out_width(in_width: usize) -> usize {
    // Macro from nes_ntsc.h:
    // ((((in_width) - 1) / 3 + 1) * 7)
    if in_width == 0 {
        return 0;
    }
    (((in_width - 1) / 3) + 1) * 7
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct nes_ntsc_setup_t {
    pub hue: f64,
    pub saturation: f64,
    pub contrast: f64,
    pub brightness: f64,
    pub sharpness: f64,
    pub gamma: f64,
    pub resolution: f64,
    pub artifacts: f64,
    pub fringing: f64,
    pub bleed: f64,
    pub merge_fields: c_int,
    pub decoder_matrix: *const f32,
    pub palette_out: *mut c_uchar,
    pub palette: *const c_uchar,
    pub base_palette: *const c_uchar,
}

// Vendor config sets NES_NTSC_EMPHASIS=1 → 64*8 entries.
const NES_NTSC_PALETTE_SIZE: usize = 64 * 8;
const NES_NTSC_ENTRY_SIZE: usize = 128;

#[repr(C)]
pub struct nes_ntsc_t {
    table: [[c_ulong; NES_NTSC_ENTRY_SIZE]; NES_NTSC_PALETTE_SIZE],
}

pub struct NesNtsc {
    data: Box<nes_ntsc_t>,
    setup: nes_ntsc_setup_t,
}

impl core::fmt::Debug for NesNtsc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NesNtsc")
            .field("setup", &"nes_ntsc_setup_t")
            .finish()
    }
}

// SAFETY: `NesNtsc` is moved to the emulation thread and used there only.
// Any pointers stored in `setup` are required to remain valid for the lifetime
// of the owning post-processor.
unsafe impl Send for NesNtsc {}

impl NesNtsc {
    pub fn new(preset: NesNtscPreset) -> Self {
        // SAFETY: `nes_ntsc_init` writes the whole table.
        let mut data = Box::new(MaybeUninit::<nes_ntsc_t>::zeroed());
        let setup = *preset.setup();
        unsafe {
            nes_ntsc_init(data.as_mut_ptr(), &setup);
            let ptr = Box::into_raw(data) as *mut nes_ntsc_t;
            Self {
                data: Box::from_raw(ptr),
                setup,
            }
        }
    }

    pub fn set_setup(&mut self, setup: nes_ntsc_setup_t) {
        self.setup = setup;
        unsafe {
            nes_ntsc_init(self.data.as_mut(), &self.setup);
        }
    }

    pub fn blit(
        &self,
        nes_in: *const u16,
        in_row_width: usize,
        burst_phase: i32,
        in_width: usize,
        in_height: usize,
        rgb_out: *mut c_void,
        out_pitch_bytes: usize,
    ) {
        unsafe {
            nes_ntsc_blit(
                self.data.as_ref(),
                nes_in,
                in_row_width as c_long,
                burst_phase as c_int,
                in_width as c_int,
                in_height as c_int,
                rgb_out,
                out_pitch_bytes as c_long,
            );
        }
    }

    pub fn setup_mut(&mut self) -> &mut nes_ntsc_setup_t {
        &mut self.setup
    }

    pub fn setup(&self) -> &nes_ntsc_setup_t {
        &self.setup
    }
}

unsafe extern "C" {
    static nes_ntsc_composite: nes_ntsc_setup_t;
    static nes_ntsc_svideo: nes_ntsc_setup_t;
    static nes_ntsc_rgb: nes_ntsc_setup_t;
    static nes_ntsc_monochrome: nes_ntsc_setup_t;

    fn nes_ntsc_init(ntsc: *mut nes_ntsc_t, setup: *const nes_ntsc_setup_t);
    fn nes_ntsc_blit(
        ntsc: *const nes_ntsc_t,
        nes_in: *const u16,
        in_row_width: c_long,
        burst_phase: c_int,
        in_width: c_int,
        in_height: c_int,
        rgb_out: *mut c_void,
        out_pitch: c_long,
    );
}

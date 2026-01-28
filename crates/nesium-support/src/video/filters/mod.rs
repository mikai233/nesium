#[cfg(feature = "hqx-cpp")]
pub mod hqx;

#[cfg(feature = "hqx-cpp")]
pub use hqx::HqxPostProcessor;

#[cfg(feature = "ntsc-cpp")]
pub mod ntsc;

#[cfg(feature = "ntsc-cpp")]
pub use ntsc::NesNtscPostProcessor;

#[cfg(feature = "ntsc-cpp")]
pub use crate::video::ntsc::NesNtscPreset;

#[cfg(feature = "ntsc-cpp")]
pub use ntsc::NesNtscTuning;

pub mod sai;

pub use sai::{SaiPostProcessor, SaiVariant};

pub mod lcd_grid;

pub use lcd_grid::LcdGridPostProcessor;

pub mod scanline;

pub use scanline::ScanlinePostProcessor;

pub mod xbrz;

pub use xbrz::XbrzPostProcessor;

#[cfg(feature = "ntsc-bisqwit-cpp")]
pub mod ntsc_bisqwit;

#[cfg(feature = "ntsc-bisqwit-cpp")]
pub use ntsc_bisqwit::{NtscBisqwitOptions, NtscBisqwitPostProcessor};

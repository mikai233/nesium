#[cfg(feature = "hqx")]
pub mod hqx;

#[cfg(feature = "hqx")]
pub use hqx::HqxPostProcessor;

#[cfg(feature = "ntsc")]
pub mod ntsc;

#[cfg(feature = "ntsc")]
pub use ntsc::NesNtscPostProcessor;

#[cfg(feature = "ntsc")]
pub use crate::video::ntsc::NesNtscPreset;

#[cfg(feature = "ntsc")]
pub use ntsc::NesNtscTuning;

pub mod sai;

pub use sai::{SaiPostProcessor, SaiVariant};

pub mod lcd_grid;

pub use lcd_grid::LcdGridPostProcessor;

pub mod scanline;

pub use scanline::ScanlinePostProcessor;

pub mod xbrz;

pub use xbrz::XbrzPostProcessor;

#[cfg(all(feature = "ntsc-bisqwit", not(target_arch = "wasm32")))]
pub mod ntsc_bisqwit;

#[cfg(all(feature = "ntsc-bisqwit", not(target_arch = "wasm32")))]
pub use ntsc_bisqwit::{NtscBisqwitOptions, NtscBisqwitPostProcessor};

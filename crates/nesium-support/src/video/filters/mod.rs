#[cfg(all(feature = "hqx-cpp", not(target_arch = "wasm32")))]
pub mod hqx;

#[cfg(all(feature = "hqx-cpp", not(target_arch = "wasm32")))]
pub use hqx::HqxPostProcessor;

#[cfg(all(feature = "ntsc-cpp", not(target_arch = "wasm32")))]
pub mod ntsc;

#[cfg(all(feature = "ntsc-cpp", not(target_arch = "wasm32")))]
pub use ntsc::NesNtscPostProcessor;

#[cfg(all(feature = "ntsc-cpp", not(target_arch = "wasm32")))]
pub use crate::video::ntsc::NesNtscPreset;

#[cfg(all(feature = "ntsc-cpp", not(target_arch = "wasm32")))]
pub use ntsc::NesNtscTuning;

pub mod sai;

pub use sai::{SaiPostProcessor, SaiVariant};

pub mod lcd_grid;

pub use lcd_grid::LcdGridPostProcessor;

pub mod scanline;

pub use scanline::ScanlinePostProcessor;

pub mod xbrz;

pub use xbrz::XbrzPostProcessor;

#[cfg(all(feature = "ntsc-bisqwit-cpp", not(target_arch = "wasm32")))]
pub mod ntsc_bisqwit;

#[cfg(all(feature = "ntsc-bisqwit-cpp", not(target_arch = "wasm32")))]
pub use ntsc_bisqwit::{NtscBisqwitOptions, NtscBisqwitPostProcessor};

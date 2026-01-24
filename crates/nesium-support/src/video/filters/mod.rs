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

#[cfg(feature = "sai")]
pub mod sai;

#[cfg(feature = "sai")]
pub use sai::{SaiPostProcessor, SaiVariant};

#[cfg(all(feature = "lcd-grid", not(target_arch = "wasm32")))]
pub mod lcd_grid;

#[cfg(all(feature = "lcd-grid", not(target_arch = "wasm32")))]
pub use lcd_grid::LcdGridPostProcessor;

#[cfg(all(feature = "scanline", not(target_arch = "wasm32")))]
pub mod scanline;

#[cfg(all(feature = "scanline", not(target_arch = "wasm32")))]
pub use scanline::ScanlinePostProcessor;

#[cfg(all(feature = "xbrz", not(target_arch = "wasm32")))]
pub mod xbrz;

#[cfg(all(feature = "xbrz", not(target_arch = "wasm32")))]
pub use xbrz::XbrzPostProcessor;

#[cfg(all(feature = "ntsc-bisqwit", not(target_arch = "wasm32")))]
pub mod ntsc_bisqwit;

#[cfg(all(feature = "ntsc-bisqwit", not(target_arch = "wasm32")))]
pub use ntsc_bisqwit::{NtscBisqwitOptions, NtscBisqwitPostProcessor};

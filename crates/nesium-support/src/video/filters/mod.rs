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

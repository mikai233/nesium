pub mod bus;
pub mod channel;
pub mod filters;
pub mod mixer;
pub mod settings;

pub use bus::SoundMixerBus;
pub use channel::AudioChannel;
pub use filters::StereoFilterType;
pub use mixer::NesSoundMixer;
pub use settings::MixerSettings;

/// NTSC CPU clock rate (also used to drive the APU).
pub const CPU_CLOCK_NTSC: f64 = 1_789_773.0;

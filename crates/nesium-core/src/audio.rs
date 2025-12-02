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

use crate::mem_block::MemBlock;

/// Generic per-channel array covering all NES+expansion audio channels.
pub type ChannelArray<T> = MemBlock<T, { AudioChannel::COUNT }>;
/// Per-channel linear output level state.
pub type ChannelLevels = ChannelArray<f32>;
/// Per-channel mixer volume.
pub type ChannelVolumes = ChannelArray<f32>;
/// Per-channel stereo panning.
pub type ChannelPanning = ChannelArray<f32>;

/// NTSC CPU clock rate (also used to drive the APU).
pub const CPU_CLOCK_NTSC: f64 = 1_789_773.0;

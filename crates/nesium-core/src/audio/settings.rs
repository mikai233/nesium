use crate::audio::{ChannelPanning, ChannelVolumes};

use super::filters::StereoFilterType;

/// User-facing mixer settings that control per-channel volume and panning.
///
/// Frontends can construct and apply this to keep balances and stereo
/// placement consistent with their own configuration UI or with Mesen2's
/// `ChannelVolumes` / `ChannelPanning` defaults.
#[derive(Debug, Clone)]
pub struct MixerSettings {
    /// Per-channel volume in `[0.0, 1.0]` (0 = muted, 1 = full).
    pub volume: ChannelVolumes,
    /// Per-channel panning in `[-1.0, 1.0]` (-1 = hard left, 0 = center, 1 = hard right).
    pub panning: ChannelPanning,
    /// Optional stereo post-filter applied after mixing.
    pub stereo_filter: StereoFilterType,
    /// Delay (ms) for [`StereoFilterType::Delay`] and [`StereoFilterType::Comb`].
    pub stereo_delay_ms: f32,
    /// Global stereo panning angle in degrees for [`StereoFilterType::Panning`].
    pub stereo_panning_angle_deg: f32,
    /// Comb filter delay (ms) for [`StereoFilterType::Comb`].
    pub stereo_comb_delay_ms: f32,
    /// Comb filter strength in `[0.0, 1.0]` for [`StereoFilterType::Comb`].
    pub stereo_comb_strength: f32,
}

impl Default for MixerSettings {
    fn default() -> Self {
        let mut volume = ChannelVolumes::new();
        volume.fill(1.0);
        Self {
            volume,
            panning: ChannelPanning::new(),
            stereo_filter: StereoFilterType::None,
            stereo_delay_ms: 0.0,
            stereo_panning_angle_deg: 0.0,
            stereo_comb_delay_ms: 0.0,
            stereo_comb_strength: 0.0,
        }
    }
}

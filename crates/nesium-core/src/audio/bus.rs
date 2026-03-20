//! Global audio bus that accepts PCM from one or more [`NesSoundMixer`]
//! instances and produces stereo PCM at the host sample rate.
//!
//! This mirrors Mesen2's `Core/Shared/Audio/SoundMixer` at a high level but
//! starts with a minimal feature set: fixed input rate (typically 96 kHz),
//! configurable output rate, simple master volume/background attenuation, a
//! Hermite resampler, and basic EQ/reverb/crossfeed support.

/// Minimal audio bus configuration inspired by Mesen2's `AudioConfig`.
///
/// All fields are in host-facing units:
/// - `master_volume` in `[0.0, 1.0]` (0 = muted, 1 = full scale).
/// - `volume_reduction` in `[0.0, 1.0]` (0.75 ≈ "reduce by 75%").
/// - `mute_in_background` / `reduce_in_background` control attenuation when
///   `in_background` is true.
/// - `reduce_in_fast_forward` controls attenuation when `is_fast_forward`
///   is true.
#[derive(Debug, Clone, Copy)]
pub struct AudioBusConfig {
    pub master_volume: f32,
    pub mute_in_background: bool,
    pub reduce_in_background: bool,
    pub reduce_in_fast_forward: bool,
    pub volume_reduction: f32,
    pub in_background: bool,
    pub is_fast_forward: bool,
    /// Enable the EQ stage (see `eq_band_gains`).
    pub enable_equalizer: bool,
    /// Per-band EQ gains in dB (20 bands, loosely mirroring Mesen2).
    pub eq_band_gains: [f32; 20],
    /// Enable the reverb stage.
    pub reverb_enabled: bool,
    /// Reverb strength in `[0.0, 1.0]` (0 = off, 1 = strong).
    pub reverb_strength: f32,
    /// Reverb base delay in milliseconds.
    pub reverb_delay_ms: f32,
    /// Enable the crossfeed stage.
    pub crossfeed_enabled: bool,
    /// Crossfeed ratio in `[0.0, 1.0]` (0 = none, 1 = strong).
    pub crossfeed_ratio: f32,
}

impl Default for AudioBusConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            mute_in_background: false,
            reduce_in_background: true,
            reduce_in_fast_forward: false,
            // Match Mesen2's default of 75% reduction when enabled.
            volume_reduction: 0.75,
            in_background: false,
            is_fast_forward: false,
            enable_equalizer: false,
            eq_band_gains: [0.0; 20],
            reverb_enabled: false,
            reverb_strength: 0.0,
            reverb_delay_ms: 0.0,
            crossfeed_enabled: false,
            crossfeed_ratio: 0.0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Equalizer {
    bands_db: [f32; 20],
    sample_rate: u32,
}

impl Equalizer {
    fn update(&mut self, bands_db: &[f32; 20], sample_rate: u32) {
        self.bands_db = *bands_db;
        self.sample_rate = sample_rate;
    }

    fn apply(&mut self, samples: &mut [f32]) {
        // Neutral when all gains are near 0 dB.
        if self.bands_db.iter().all(|g| g.abs() < 0.001) {
            return;
        }

        // Minimal implementation: approximate the multi-band EQ with a single
        // global gain based on the average requested band gain.
        let sum: f32 = self.bands_db.iter().copied().sum();
        let avg_db = sum / self.bands_db.len() as f32;
        let gain = 10.0_f32.powf(avg_db / 20.0);
        if (gain - 1.0).abs() < 0.001 {
            return;
        }

        for s in samples {
            *s *= gain;
        }
    }
}

#[derive(Debug, Default, Clone)]
struct ReverbFilter {
    left: Vec<f32>,
    right: Vec<f32>,
    index: usize,
    delay_samples: usize,
    decay: f32,
}

impl ReverbFilter {
    fn reset(&mut self) {
        self.left.clear();
        self.right.clear();
        self.index = 0;
        self.delay_samples = 0;
        self.decay = 0.0;
    }

    fn configure(&mut self, sample_rate: u32, strength: f32, delay_ms: f32) {
        if sample_rate == 0 {
            self.reset();
            return;
        }

        let delay_samples = ((delay_ms / 1000.0) * sample_rate as f32).round().max(1.0) as usize;
        let decay = strength.clamp(0.0, 1.0);

        if delay_samples != self.delay_samples {
            self.left.clear();
            self.right.clear();
            self.left.resize(delay_samples, 0.0);
            self.right.resize(delay_samples, 0.0);
            self.index = 0;
        }

        self.delay_samples = delay_samples;
        self.decay = decay;
    }

    fn apply(&mut self, samples: &mut [f32], sample_rate: u32, strength: f32, delay_ms: f32) {
        if strength <= 0.0 || delay_ms <= 0.0 {
            // When disabled, keep any existing delay line but do not add new
            // reverb until re-enabled.
            return;
        }

        let frames = samples.len() / 2;
        if frames == 0 {
            return;
        }

        self.configure(sample_rate, strength, delay_ms);
        if self.delay_samples == 0 || self.left.is_empty() {
            return;
        }

        let delay_len = self.delay_samples;
        for i in 0..frames {
            let idx = self.index % delay_len;

            let l = samples[2 * i];
            let r = samples[2 * i + 1];

            let dl = self.left[idx];
            let dr = self.right[idx];

            let out_l = l + dl * self.decay;
            let out_r = r + dr * self.decay;

            samples[2 * i] = out_l;
            samples[2 * i + 1] = out_r;

            // Simple feedback: feed the wet signal back into the delay line.
            self.left[idx] = out_l;
            self.right[idx] = out_r;

            self.index = (self.index + 1) % delay_len;
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct CrossFeedFilter;

impl CrossFeedFilter {
    fn apply(&mut self, samples: &mut [f32], ratio: f32) {
        let r = ratio.clamp(0.0, 1.0);
        if r <= 0.0 {
            return;
        }
        let frames = samples.len() / 2;
        for i in 0..frames {
            let idx = i * 2;
            let l = samples[idx];
            let r_sample = samples[idx + 1];
            samples[idx] = l + r_sample * r;
            samples[idx + 1] = r_sample + l * r;
        }
    }
}

#[derive(Debug, Clone)]
struct HermiteResamplerStereo {
    prev_left: [f64; 4],
    prev_right: [f64; 4],
    volume: i32,
    rate_ratio: f64,
    fraction: f64,
    left: i16,
    right: i16,
}

impl Default for HermiteResamplerStereo {
    fn default() -> Self {
        Self {
            prev_left: [0.0; 4],
            prev_right: [0.0; 4],
            volume: 256,
            rate_ratio: 1.0,
            fraction: 0.0,
            left: 0,
            right: 0,
        }
    }
}

impl HermiteResamplerStereo {
    fn reset(&mut self) {
        self.prev_left = [0.0; 4];
        self.prev_right = [0.0; 4];
        self.fraction = 0.0;
        self.left = 0;
        self.right = 0;
    }

    fn set_volume(&mut self, volume: f64) {
        self.volume = (volume * 256.0).round() as i32;
    }

    fn set_sample_rates(&mut self, src_rate: f64, dst_rate: f64) {
        if src_rate > 0.0 && dst_rate > 0.0 {
            self.rate_ratio = src_rate / dst_rate;
        } else {
            self.rate_ratio = 1.0;
        }
    }

    fn resample(&mut self, input: &[i16], out: &mut Vec<i16>) {
        if input.is_empty() {
            return;
        }

        if (self.rate_ratio - 1.0).abs() <= f64::EPSILON {
            out.reserve(input.len());
            for frame in input.chunks_exact(2) {
                let left = frame[0];
                let right = frame[1];
                self.left = left;
                self.right = right;
                self.write_sample(out, left, right);
            }
            return;
        }

        out.reserve(input.len());
        for frame in input.chunks_exact(2) {
            while self.fraction <= 1.0 {
                self.left = self.hermite_interpolate(&self.prev_left, self.fraction);
                self.right = self.hermite_interpolate(&self.prev_right, self.fraction);
                self.write_sample(out, self.left, self.right);
                self.fraction += self.rate_ratio;
            }

            Self::push_sample(&mut self.prev_left, frame[0]);
            Self::push_sample(&mut self.prev_right, frame[1]);
            self.fraction -= 1.0;
        }
    }

    fn write_sample(&self, out: &mut Vec<i16>, left: i16, right: i16) {
        let l = (((left as i32) * self.volume) >> 8).clamp(i16::MIN as i32, i16::MAX as i32);
        let r = (((right as i32) * self.volume) >> 8).clamp(i16::MIN as i32, i16::MAX as i32);
        out.push(l as i16);
        out.push(r as i16);
    }

    fn hermite_interpolate(&self, values: &[f64; 4], mu: f64) -> i16 {
        let mu2 = mu * mu;
        let mu3 = mu2 * mu;
        let m0 = (values[1] - values[0]) / 2.0 + (values[2] - values[1]) / 2.0;
        let m1 = (values[2] - values[1]) / 2.0 + (values[3] - values[2]) / 2.0;
        let a0 = 2.0 * mu3 - 3.0 * mu2 + 1.0;
        let a1 = mu3 - 2.0 * mu2 + mu;
        let a2 = mu3 - mu2;
        let a3 = -2.0 * mu3 + 3.0 * mu2;
        let output = a0 * values[1] + a1 * m0 + a2 * m1 + a3 * values[2];
        output.clamp(i16::MIN as f64, i16::MAX as f64) as i16
    }

    fn push_sample(prev_values: &mut [f64; 4], sample: i16) {
        prev_values[0] = prev_values[1];
        prev_values[1] = prev_values[2];
        prev_values[2] = prev_values[3];
        prev_values[3] = sample as f64;
    }
}

#[derive(Debug, Clone)]
pub struct SoundMixerBus {
    /// Base input sample rate used by the per-console mixer (e.g. 96 kHz).
    base_input_rate: u32,
    /// Effective input sample rate used by the bus resampler.
    ///
    /// This usually matches `base_input_rate`, but can be adjusted to time-stretch
    /// audio when the frontend runs the emulator at an integer display FPS (e.g. 60Hz)
    /// instead of the console's exact FPS (e.g. 60.0988Hz on NTSC).
    input_rate: u32,
    /// Host/device output sample rate.
    output_rate: u32,
    /// Master volume and attenuation configuration.
    config: AudioBusConfig,
    /// Optional EQ applied at the bus level.
    eq: Equalizer,
    /// Simple stereo reverb.
    reverb: ReverbFilter,
    /// Simple stereo crossfeed.
    crossfeed: CrossFeedFilter,
    /// Scratch buffer used when summing multiple sources before resampling.
    mix_scratch: Vec<f32>,
    /// PCM16 scratch mirror of `mix_scratch`.
    mix_scratch_i16: Vec<i16>,
    /// Resampler output scratch (PCM16 stereo interleaved).
    resample_scratch_i16: Vec<i16>,
    /// Mesen-style Hermite resampler state.
    resampler: HermiteResamplerStereo,
}

impl SoundMixerBus {
    /// Constructs a new bus that converts from `input_rate` to `output_rate`.
    pub fn new(input_rate: u32, output_rate: u32) -> Self {
        let input_rate = input_rate.max(1);
        Self {
            base_input_rate: input_rate,
            input_rate,
            output_rate: output_rate.max(1),
            config: AudioBusConfig::default(),
            eq: Equalizer::default(),
            reverb: ReverbFilter::default(),
            crossfeed: CrossFeedFilter,
            mix_scratch: Vec::new(),
            mix_scratch_i16: Vec::new(),
            resample_scratch_i16: Vec::new(),
            resampler: {
                let mut r = HermiteResamplerStereo::default();
                r.set_volume(1.0);
                r.set_sample_rates(input_rate as f64, output_rate as f64);
                r
            },
        }
    }

    /// Clears any internal state. The current rate configuration is preserved.
    pub fn reset(&mut self) {
        self.mix_scratch.clear();
        self.mix_scratch_i16.clear();
        self.resample_scratch_i16.clear();
        self.reverb.reset();
        self.resampler.reset();
        self.resampler
            .set_sample_rates(self.input_rate as f64, self.output_rate as f64);
    }

    /// Adjusts the effective input sample rate used by the bus resampler.
    ///
    /// This is a "time-stretch" knob. The actual input samples are still produced at
    /// `base_input_rate`, but changing this value alters how many samples the resampler
    /// outputs for a given input chunk.
    pub fn set_resample_input_rate(&mut self, input_rate: u32) {
        self.input_rate = input_rate.max(1);
        self.resampler
            .set_sample_rates(self.input_rate as f64, self.output_rate as f64);
    }

    /// Restores the resampler input rate back to the original mixer input rate.
    pub fn reset_resample_input_rate(&mut self) {
        self.input_rate = self.base_input_rate;
        self.resampler
            .set_sample_rates(self.input_rate as f64, self.output_rate as f64);
    }

    /// Updates the output sample rate while keeping the input rate fixed.
    pub fn set_output_rate(&mut self, output_rate: u32) {
        self.output_rate = output_rate.max(1);
        self.resampler
            .set_sample_rates(self.input_rate as f64, self.output_rate as f64);
    }

    /// Updates the bus configuration (master volume and attenuation flags).
    pub fn set_config(&mut self, config: AudioBusConfig) {
        self.config = config;
    }

    /// Current bus configuration.
    pub fn config(&self) -> AudioBusConfig {
        self.config
    }

    /// Current output sample rate.
    pub fn output_rate(&self) -> u32 {
        self.output_rate
    }

    /// Mixes one or more interleaved stereo sources and resamples them into
    /// `out` at the configured output rate.
    ///
    /// - All sources are expected to share the same input rate (`input_rate`)
    ///   and length in frames; extra samples in longer sources are ignored.
    /// - Samples are summed in linear amplitude space before resampling.
    /// - Resampling follows Mesen2's Hermite algorithm and preserves state
    ///   across chunks.
    pub fn mix_frame(&mut self, sources: &[&[f32]], out: &mut Vec<f32>) {
        if sources.is_empty() {
            return;
        }

        let min_len = sources.iter().map(|s| s.len()).min().unwrap_or(0);
        let frames_in = min_len / 2;
        if frames_in == 0 {
            return;
        }

        // Sum all sources into the scratch buffer.
        self.mix_scratch.clear();
        self.mix_scratch.resize(frames_in * 2, 0.0);

        for src in sources {
            let frames = (src.len() / 2).min(frames_in);
            for i in 0..frames * 2 {
                self.mix_scratch[i] += src[i];
            }
        }

        self.mix_scratch_i16.clear();
        self.mix_scratch_i16.reserve(self.mix_scratch.len());
        for &sample in &self.mix_scratch {
            self.mix_scratch_i16.push(f32_to_i16_sample(sample));
        }

        self.resample_scratch_i16.clear();
        self.resampler
            .resample(&self.mix_scratch_i16, &mut self.resample_scratch_i16);

        let out_start = out.len();
        out.reserve(self.resample_scratch_i16.len());
        for &sample in &self.resample_scratch_i16 {
            out.push(i16_to_f32_sample(sample));
        }

        let slice = &mut out[out_start..];

        // Apply EQ, reverb and crossfeed in the bus, mirroring Mesen2's
        // SoundMixer ordering (EQ → reverb → crossfeed → master volume).
        if self.config.enable_equalizer {
            self.eq.update(&self.config.eq_band_gains, self.output_rate);
            self.eq.apply(slice);
        }

        if self.config.reverb_enabled {
            self.reverb.apply(
                slice,
                self.output_rate,
                self.config.reverb_strength,
                self.config.reverb_delay_ms,
            );
        }

        if self.config.crossfeed_enabled {
            self.crossfeed.apply(slice, self.config.crossfeed_ratio);
        }

        // Apply master volume / attenuation at the very end of the bus path,
        // mirroring Mesen2's `AudioConfig`-driven scaling.
        let gain = effective_gain(self.config);
        if gain < 1.0 - f32::EPSILON {
            for s in &mut out[out_start..] {
                *s *= gain;
            }
        }
    }
}

fn effective_gain(config: AudioBusConfig) -> f32 {
    let mut gain = config.master_volume.clamp(0.0, 1.0);

    if config.in_background {
        if config.mute_in_background {
            gain = 0.0;
        } else if config.reduce_in_background {
            let factor = 1.0 - config.volume_reduction.clamp(0.0, 1.0);
            gain *= factor;
        }
    }

    if config.is_fast_forward && config.reduce_in_fast_forward {
        let factor = 1.0 - config.volume_reduction.clamp(0.0, 1.0);
        gain *= factor;
    }

    gain
}

#[inline]
fn f32_to_i16_sample(sample: f32) -> i16 {
    let s = if sample.is_finite() { sample } else { 0.0 };
    let scaled = (s * 32768.0).round() as i32;
    scaled.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

#[inline]
fn i16_to_f32_sample(sample: i16) -> f32 {
    sample as f32 / 32768.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_identity_copies_input() {
        let mut bus = SoundMixerBus::new(48_000, 48_000);
        let src = vec![0.1f32, -0.1, 0.2, -0.2, 0.3, -0.3, 0.4, -0.4];
        let mut out = Vec::new();
        bus.mix_frame(&[&src], &mut out);
        assert_eq!(src.len(), out.len());
        for (expected, actual) in src.iter().zip(out.iter()) {
            assert!((expected - actual).abs() <= (1.0 / 32768.0));
        }
    }

    #[test]
    fn resample_down_96k_to_48k_halves_frame_count() {
        let mut bus = SoundMixerBus::new(96_000, 48_000);
        // 1600 frames of a simple ramp at 96 kHz (~1/60th second).
        let frames_in = 1600usize;
        let mut src = Vec::with_capacity(frames_in * 2);
        for i in 0..frames_in {
            let v = i as f32 / frames_in as f32;
            src.push(v);
            src.push(-v);
        }

        let mut out = Vec::new();
        bus.mix_frame(&[&src], &mut out);

        let frames_out = out.len() / 2;
        // Mesen's Hermite resampler emits one extra startup sample when
        // downsampling a cold stream in a single chunk.
        assert_eq!(frames_out, 801, "expected Hermite startup sample at 48 kHz");

        // Check endpoints preserve polarity and approximate amplitude.
        let l_first = out[0];
        let r_first = out[1];
        let l_last = out[out.len() - 2];
        let r_last = out[out.len() - 1];

        assert!(l_first.abs() < 1e-6 && r_first.abs() < 1e-6);
        assert!(l_last > 0.9 && r_last < -0.9);
    }

    #[test]
    fn resample_down_96k_to_44100_matches_expected_frames_per_frame() {
        let mut bus = SoundMixerBus::new(96_000, 44_100);
        let frames_in = 1600usize; // 96k / 60
        let mut src = Vec::with_capacity(frames_in * 2);
        for _ in 0..frames_in {
            src.push(0.0);
            src.push(0.0);
        }

        let mut out = Vec::new();
        bus.mix_frame(&[&src], &mut out);
        let frames_out = out.len() / 2;

        // 44_100 / 60 = 735 frames per NTSC frame.
        assert_eq!(frames_out, 735);
    }

    #[test]
    fn master_volume_scales_output() {
        let mut bus = SoundMixerBus::new(48_000, 48_000);
        let cfg = AudioBusConfig {
            master_volume: 0.5,
            ..Default::default()
        };
        bus.set_config(cfg);

        let src = vec![0.8f32, -0.8, 0.2, -0.2];
        let mut out = Vec::new();
        bus.mix_frame(&[&src], &mut out);

        assert_eq!(out.len(), src.len());
        assert!((out[0] - 0.4).abs() < 5e-4);
        assert!((out[1] + 0.4).abs() < 5e-4);
    }

    #[test]
    fn background_and_fast_forward_attenuation_match_config() {
        let mut bus = SoundMixerBus::new(48_000, 48_000);

        // Start with unity config.
        let mut cfg = AudioBusConfig {
            master_volume: 1.0,
            // default: keep 25% when reducing
            volume_reduction: 0.75,
            // Background reduction only.
            in_background: true,
            mute_in_background: false,
            reduce_in_background: true,
            is_fast_forward: false,
            reduce_in_fast_forward: false,
            ..Default::default()
        };

        bus.set_config(cfg);

        let src = vec![1.0f32, 1.0];
        let mut out = Vec::new();
        bus.mix_frame(&[&src], &mut out);
        assert_eq!(out.len(), 2);
        // 1.0 * (1.0 - 0.75) = 0.25
        assert!((out[0] - 0.25).abs() < 5e-4);

        // Fast-forward + background reduction compounded.
        out.clear();
        cfg.is_fast_forward = true;
        cfg.reduce_in_fast_forward = true;
        bus.set_config(cfg);
        bus.mix_frame(&[&src], &mut out);
        assert_eq!(out.len(), 2);
        // 1.0 * 0.25 (background) * 0.25 (fast-forward) = 0.0625
        assert!((out[0] - 0.0625).abs() < 5e-4);
    }

    #[test]
    fn equalizer_applies_global_gain_when_enabled() {
        let mut bus = SoundMixerBus::new(48_000, 48_000);
        let cfg = AudioBusConfig {
            enable_equalizer: true,
            // Request a modest 6 dB boost across all bands.
            eq_band_gains: [6.0; 20],
            ..Default::default()
        };
        bus.set_config(cfg);

        let src = vec![0.5f32, -0.5];
        let mut out = Vec::new();
        bus.mix_frame(&[&src], &mut out);

        // 6 dB ≈ *2.0 global gain.
        assert_eq!(out.len(), 2);
        assert!(out[0] > 0.9 && out[0] < 1.1);
        assert!(out[1] < -0.9 && out[1] > -1.1);
    }

    #[test]
    fn crossfeed_blends_channels_when_enabled() {
        let mut bus = SoundMixerBus::new(48_000, 48_000);
        let cfg = AudioBusConfig {
            crossfeed_enabled: true,
            crossfeed_ratio: 0.5,
            ..Default::default()
        };
        bus.set_config(cfg);

        // Hard-panned left/right.
        let src = vec![1.0f32, 0.0];
        let mut out = Vec::new();
        bus.mix_frame(&[&src], &mut out);

        assert_eq!(out.len(), 2);
        // Left remains mostly left, right receives some bleed.
        assert!((out[0] - 1.0).abs() < 5e-4);
        assert!((out[1] - 0.5).abs() < 5e-4);
    }

    #[test]
    fn reverb_adds_delayed_energy_over_time() {
        let mut bus = SoundMixerBus::new(48_000, 48_000);
        let cfg = AudioBusConfig {
            reverb_enabled: true,
            reverb_strength: 0.5,
            reverb_delay_ms: 10.0,
            ..Default::default()
        };
        bus.set_config(cfg);

        let frames = 100usize;
        let mut out = Vec::new();

        // First frame: impulse, fills the delay line but produces no reverb yet.
        let mut src = vec![0.0f32; frames * 2];
        src[0] = 1.0;
        bus.mix_frame(&[&src], &mut out);
        let first_frame = out.clone();
        assert!(first_frame.iter().any(|&v| v > 0.0));

        // Subsequent frames: silent input; after enough frames to exceed the
        // delay, expect some reverb energy due to feedback from the delay line.
        let silent = vec![0.0f32; frames * 2];
        let mut found_energy = false;
        for _ in 0..10 {
            out.clear();
            bus.mix_frame(&[&silent], &mut out);
            if out.iter().any(|&v| v.abs() > 0.0) {
                found_energy = true;
                break;
            }
        }
        assert!(found_energy);
    }
}

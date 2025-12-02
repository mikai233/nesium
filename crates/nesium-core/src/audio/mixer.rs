use std::f32::consts::PI;

use nesium_blip::BlipBuf;

use crate::audio::{
    AudioChannel, ChannelLevels, ChannelPanning, ChannelVolumes,
    filters::{StereoCombState, StereoDelayState, StereoFilterType, StereoPanningState},
    settings::MixerSettings,
};

/// Band-limited NES mixer that accepts per-channel amplitude deltas tagged
/// with the CPU/APU clock time and produces filtered PCM at the host sample
/// rate.
//
// TODO(mesen2): Integrate mixer state into save-states (similar to NesSoundMixer::Serialize),
// so loading a state restores filter history and per-channel levels without
// transient pops or phase jumps.
// TODO(mesen2): Model VS DualSystem audio mixing (ProcessVsDualSystemAudio) once
// multi-console support exists, so main/sub console audio routing matches Mesen2.
// TODO(mesen2): Audit clocking/rate updates against Mesen2's UpdateRates() path
// (dynamic region/clock changes, MaxSamplesPerFrame, overclock scenarios) to
// ensure blip_buf never truncates samples under extreme settings.
#[derive(Debug)]
pub struct NesSoundMixer {
    blip_left: BlipBuf,
    blip_right: BlipBuf,
    clock_rate: f64,
    sample_rate: f32,
    last_frame_clock: i64,
    /// Last linear output level per channel (pulse1/2, triangle, noise, DMC, plus per-chip expansion).
    channel_levels: ChannelLevels,
    /// Per-channel volume and panning (0.0 = hard left, 1.0 = center, 2.0 = hard right).
    volumes: ChannelVolumes,
    panning: ChannelPanning,
    /// Cached mixed amplitude after the non-linear APU combiner (left/right).
    mixed_left: f32,
    mixed_right: f32,
    /// DC-block filter state (per channel).
    dc_last_input_l: f32,
    dc_last_output_l: f32,
    dc_last_input_r: f32,
    dc_last_output_r: f32,
    /// Rumble high-pass state (~90 Hz, per channel).
    rumble_last_input_l: f32,
    rumble_state_l: f32,
    rumble_last_input_r: f32,
    rumble_state_r: f32,
    /// Soft low-pass to tame aliasing and harsh edges (~12 kHz, per channel).
    lowpass_state_l: f32,
    lowpass_state_r: f32,
    dc_coeff: f32,
    rumble_coeff: f32,
    lowpass_alpha: f32,
    master_gain: f32,
    has_panning: bool,

    stereo_filter: StereoFilterType,
    stereo_delay_ms: f32,
    stereo_panning_angle_deg: f32,
    stereo_comb_delay_ms: f32,
    stereo_comb_strength: f32,
    stereo_delay_state: StereoDelayState,
    stereo_panning_state: StereoPanningState,
    stereo_comb_state: StereoCombState,
}

impl NesSoundMixer {
    /// Construct a mixer for the given CPU/APU clock and host sample rate.
    pub fn new(clock_rate: f64, sample_rate: u32) -> Self {
        let sr = sample_rate as f32;
        // Approximate the NES analog output filters (per NESdev): two
        // high-pass stages around 90 Hz and 440 Hz plus a low-pass around
        // 14 kHz. The DC/rumble filters are implemented as simple one-pole
        // IIRs; the exact poles are close but not cycle-perfect.
        let dc_cut = 90.0_f32;
        let rumble_cut = 440.0_f32;
        let lowpass_cut = 14_000.0_f32;

        let dc_coeff = pole_coeff(sr, dc_cut);
        let rumble_coeff = pole_coeff(sr, rumble_cut);
        let lowpass_alpha = pole_alpha(sr, lowpass_cut);

        let mut volumes = ChannelVolumes::new();
        volumes.fill(1.0);
        let mut panning = ChannelPanning::new();
        panning.fill(1.0);

        Self {
            blip_left: BlipBuf::new(clock_rate, sample_rate as f64, 24),
            blip_right: BlipBuf::new(clock_rate, sample_rate as f64, 24),
            clock_rate,
            sample_rate: sr,
            last_frame_clock: 0,
            channel_levels: ChannelLevels::new(),
            volumes,
            panning,
            mixed_left: 0.0,
            mixed_right: 0.0,
            dc_last_input_l: 0.0,
            dc_last_output_l: 0.0,
            dc_last_input_r: 0.0,
            dc_last_output_r: 0.0,
            rumble_last_input_l: 0.0,
            rumble_state_l: 0.0,
            rumble_last_input_r: 0.0,
            rumble_state_r: 0.0,
            lowpass_state_l: 0.0,
            lowpass_state_r: 0.0,
            dc_coeff,
            rumble_coeff,
            lowpass_alpha,
            // Keep some headroom; the non-linear mixer and soft clipper will
            // do the rest. Use a neutral master gain so overall loudness
            // tracks Mesen2's `GetOutputVolume() * 4` path; headroom is
            // primarily controlled by the scaling in `mix_channels_stereo`.
            master_gain: 1.0,
            has_panning: false,
            stereo_filter: StereoFilterType::None,
            stereo_delay_ms: 0.0,
            stereo_panning_angle_deg: 0.0,
            stereo_comb_delay_ms: 0.0,
            stereo_comb_strength: 0.0,
            stereo_delay_state: StereoDelayState::default(),
            stereo_panning_state: StereoPanningState::default(),
            stereo_comb_state: StereoCombState::default(),
        }
    }

    /// Reset all accumulated state while keeping configuration.
    pub fn reset(&mut self) {
        self.blip_left.clear();
        self.blip_right.clear();
        self.last_frame_clock = 0;
        self.channel_levels.fill(0.0);
        self.mixed_left = 0.0;
        self.mixed_right = 0.0;
        self.dc_last_input_l = 0.0;
        self.dc_last_output_l = 0.0;
        self.dc_last_input_r = 0.0;
        self.dc_last_output_r = 0.0;
        self.rumble_last_input_l = 0.0;
        self.rumble_state_l = 0.0;
        self.rumble_last_input_r = 0.0;
        self.rumble_state_r = 0.0;
        self.lowpass_state_l = 0.0;
        self.lowpass_state_r = 0.0;
        self.stereo_delay_state = StereoDelayState::default();
        self.stereo_panning_state = StereoPanningState::default();
        self.stereo_comb_state = StereoCombState::default();
    }

    /// Update internal clock/sample rates, mirroring Mesen2's
    /// `NesSoundMixer::UpdateRates`.
    ///
    /// This adjusts blip_buf's notion of input clock and output sample
    /// rate and recomputes the analog-style filter coefficients without
    /// otherwise disturbing mixer state. Call this whenever the CPU/APU
    /// clock or host sample rate changes (for example, on region switch
    /// or when the audio device's sample rate is reconfigured).
    pub fn update_rates(&mut self, clock_rate: f64, sample_rate: u32) {
        let sr = sample_rate as f32;
        if (clock_rate - self.clock_rate).abs() < f64::EPSILON
            && (sr - self.sample_rate).abs() < f32::EPSILON
        {
            return;
        }

        self.clock_rate = clock_rate;
        self.sample_rate = sr;
        self.blip_left.set_rates(clock_rate, sample_rate as f64);
        self.blip_right.set_rates(clock_rate, sample_rate as f64);

        // Recompute filter coefficients so the DC/rumble/low-pass behaviour
        // stays approximately aligned with NES analog characteristics at the
        // new sample rate.
        let dc_cut = 90.0_f32;
        let rumble_cut = 440.0_f32;
        let lowpass_cut = 14_000.0_f32;
        self.dc_coeff = pole_coeff(sr, dc_cut);
        self.rumble_coeff = pole_coeff(sr, rumble_cut);
        self.lowpass_alpha = pole_alpha(sr, lowpass_cut);
    }

    /// Apply per-channel volume and panning settings coming from the host.
    ///
    /// - `volume[i]` is expected in `[0.0, 1.0]` (0 = muted, 1 = full).
    /// - `panning[i]` is expected in `[-1.0, 1.0]` (-1 = hard left, 0 = center, 1 = hard right).
    pub fn apply_mixer_settings(&mut self, settings: &MixerSettings) {
        let mut has_panning = false;
        for (idx, (&vol, &pan)) in settings
            .volume
            .iter()
            .zip(settings.panning.iter())
            .enumerate()
        {
            self.volumes[idx] = vol.clamp(0.0, 1.0);
            // Map [-1, 1] to [0, 2] like Mesen2's (ChannelPanning + 100) / 100.
            self.panning[idx] = (pan.clamp(-1.0, 1.0) + 1.0).clamp(0.0, 2.0);
            if self.panning[idx] != 1.0 {
                // Match Mesen2's behaviour: when transitioning from "all
                // channels centered" to "per-channel panning", clear both
                // blip buffers so the stereo configuration change does not
                // cause oddities with in-flight samples.
                if !self.has_panning {
                    self.blip_left.clear();
                    self.blip_right.clear();
                }
                has_panning = true;
            }
        }
        self.has_panning = has_panning;

        self.stereo_filter = settings.stereo_filter;
        self.stereo_delay_ms = settings.stereo_delay_ms.max(0.0);
        self.stereo_panning_angle_deg = settings.stereo_panning_angle_deg;
        self.stereo_comb_delay_ms = settings.stereo_comb_delay_ms.max(0.0);
        self.stereo_comb_strength = settings.stereo_comb_strength.clamp(0.0, 1.0);
    }

    /// Directly apply a channel delta at the given CPU/APU clock.
    ///
    /// `clock_time` is an absolute CPU/APU cycle count (typically
    /// [`Nes::apu_cycles`](crate::Nes::apu_cycles)) that must be monotonically
    /// non-decreasing within a frame. Internally this is converted to a
    /// frame-local timestamp by subtracting the last frame's end clock before
    /// feeding it to blip_buf, mirroring Mesen2's `NesSoundMixer::AddDelta`.
    pub fn add_delta(&mut self, channel: AudioChannel, clock_time: i64, delta: f32) {
        if delta == 0.0 {
            return;
        }
        let idx = channel.idx();
        self.channel_levels[idx] += delta;

        let (left, right) = self.mix_channels_stereo();
        let delta_left = left - self.mixed_left;
        let delta_right = right - self.mixed_right;
        if delta_left != 0.0 || delta_right != 0.0 {
            // BlipBuf expects clock times relative to the start of the
            // current frame, so convert from the absolute CPU/APU clock.
            let rel_clock = clock_time - self.last_frame_clock;
            debug_assert!(
                rel_clock >= 0,
                "NesSoundMixer::add_delta must be called with non-decreasing clock times within a frame"
            );
            if rel_clock >= 0 {
                // Map our roughly [-1.0, 1.0] mixed amplitude into the
                // integer domain expected by the C blip_buf implementation.
                // Mesen2 computes an int16 GetOutputVolume() and then scales
                // it by 4 before feeding it to blip_add_delta(), so a factor
                // of ~32768 gives similar resolution.
                const BLIP_SCALE: f32 = 32_768.0;
                if delta_left != 0.0 {
                    self.blip_left.add_delta(rel_clock, delta_left * BLIP_SCALE);
                }
                if delta_right != 0.0 {
                    self.blip_right
                        .add_delta(rel_clock, delta_right * BLIP_SCALE);
                }
            }
            self.mixed_left = left;
            self.mixed_right = right;
        }
    }

    /// Convenience helper that computes the delta against the last channel level.
    pub fn set_channel_level(&mut self, channel: AudioChannel, clock_time: i64, value: f32) {
        let idx = channel.idx();
        let delta = value - self.channel_levels[idx];
        self.add_delta(channel, clock_time, delta);
    }

    /// Finalize all samples up to `frame_end_clock` and push filtered PCM into `out`.
    ///
    /// `frame_end_clock` is the absolute CPU/APU cycle at the end of the
    /// current audio frame (for example, the current APU cycle counter when
    /// the PPU reports a frame boundary). The duration passed to blip_buf is
    /// computed relative to the previous `frame_end_clock`, matching
    /// Mesen2's `NesSoundMixer::EndFrame` behaviour.
    pub fn end_frame(&mut self, frame_end_clock: i64, out: &mut Vec<f32>) {
        // Convert absolute clock into a frame-relative duration for BlipBuf.
        let duration = frame_end_clock - self.last_frame_clock;
        debug_assert!(
            duration >= 0,
            "NesSoundMixer::end_frame must be called with non-decreasing frame_end_clock"
        );
        if duration < 0 {
            return;
        }

        self.blip_left.end_frame(duration);
        self.blip_right.end_frame(duration);
        self.last_frame_clock = frame_end_clock;

        let avail_left = self.blip_left.samples_avail();
        let avail_right = self.blip_right.samples_avail();
        let avail = avail_left.min(avail_right);
        if avail == 0 {
            return;
        }

        let mut left = vec![0.0f32; avail];
        let mut right = vec![0.0f32; avail];
        let got_left = self.blip_left.read_samples(&mut left[..]);
        let got_right = self.blip_right.read_samples(&mut right[..]);
        let got = got_left.min(got_right);
        let mut stereo = Vec::with_capacity(got * 2);

        for i in 0..got {
            let l = left[i];
            let r = right[i];

            // For closer parity with Mesen2, bypass the additional DC and
            // rumble high-pass filters here and rely primarily on the APU's
            // own non-linear mixer characteristics plus the final low-pass.
            let l_smoothed = low_pass(l, &mut self.lowpass_state_l, self.lowpass_alpha);
            let r_smoothed = low_pass(r, &mut self.lowpass_state_r, self.lowpass_alpha);

            let l_scaled = soft_clip(l_smoothed * self.master_gain);
            let r_scaled = soft_clip(r_smoothed * self.master_gain);

            stereo.push(l_scaled);
            stereo.push(r_scaled);
        }

        self.apply_stereo_post_filters(&mut stereo);
        out.extend_from_slice(&stereo);
    }

    fn mix_channels_stereo(&self) -> (f32, f32) {
        let idx = |ch: AudioChannel| ch.idx();
        let base = |ch: AudioChannel| self.channel_levels[idx(ch)] as f64;
        let vol = |ch: AudioChannel| self.volumes[idx(ch)] as f64;
        let pan = |ch: AudioChannel| self.panning[idx(ch)] as f64;

        // Helper to compute left/right contribution for a single channel
        // given its base linear output, per-channel volume, and panning.
        let lr = |ch: AudioChannel| {
            let v = base(ch) * vol(ch);
            let p = pan(ch);
            let left = v * (2.0 - p);
            let right = v * p;
            (left, right)
        };

        let (p1_l, p1_r) = lr(AudioChannel::Pulse1);
        let (p2_l, p2_r) = lr(AudioChannel::Pulse2);
        let (t_l, t_r) = lr(AudioChannel::Triangle);
        let (n_l, n_r) = lr(AudioChannel::Noise);
        let (d_l, d_r) = lr(AudioChannel::Dmc);
        let (fds_l, fds_r) = lr(AudioChannel::Fds);
        let (mmc5_l, mmc5_r) = lr(AudioChannel::Mmc5);
        let (n163_l, n163_r) = lr(AudioChannel::Namco163);
        let (s5b_l, s5b_r) = lr(AudioChannel::Sunsoft5B);
        let (vrc6_l, vrc6_r) = lr(AudioChannel::Vrc6);
        let (vrc7_l, vrc7_r) = lr(AudioChannel::Vrc7);

        // Square contribution (two pulse channels).
        let square_l = p1_l + p2_l;
        let square_r = p1_r + p2_r;

        let square_vol_l = if square_l > 0.0 {
            (95.88 * 5000.0) / (8128.0 / square_l + 100.0)
        } else {
            0.0
        };
        let square_vol_r = if square_r > 0.0 {
            (95.88 * 5000.0) / (8128.0 / square_r + 100.0)
        } else {
            0.0
        };

        // TND (triangle/noise/DMC) contribution.
        let tnd_lin_l = d_l + 2.751_671_326_1 * t_l + 1.849_358_712_5 * n_l;
        let tnd_lin_r = d_r + 2.751_671_326_1 * t_r + 1.849_358_712_5 * n_r;

        let tnd_vol_l = if tnd_lin_l > 0.0 {
            (159.79 * 5000.0) / (22638.0 / tnd_lin_l + 100.0)
        } else {
            0.0
        };
        let tnd_vol_r = if tnd_lin_r > 0.0 {
            (159.79 * 5000.0) / (22638.0 / tnd_lin_r + 100.0)
        } else {
            0.0
        };

        // Expansion audio contribution, matching Mesen2's GetOutputVolume()
        // per-chip scalings.
        let exp_l =
            fds_l * 20.0 + mmc5_l * 43.0 + n163_l * 20.0 + s5b_l * 15.0 + vrc6_l * 5.0 + vrc7_l;
        let exp_r =
            fds_r * 20.0 + mmc5_r * 43.0 + n163_r * 20.0 + s5b_r * 15.0 + vrc6_r * 5.0 + vrc7_r;

        let mixed_l = square_vol_l + tnd_vol_l + exp_l;
        let mixed_r = square_vol_r + tnd_vol_r + exp_r;

        // Mesen2 feeds `GetOutputVolume() * 4` (int16) into blip_buf. Map that
        // same domain into our [-1.0, 1.0] float space by scaling with
        // `4 / 32768`, which keeps typical non-expansion peaks (~5000) around
        // 0.6 and leaves ample headroom for louder content and post-filters.
        const OUTPUT_SCALE: f64 = 4.0 / 32_768.0;
        (
            (mixed_l * OUTPUT_SCALE) as f32,
            (mixed_r * OUTPUT_SCALE) as f32,
        )
    }

    fn apply_stereo_post_filters(&mut self, samples: &mut [f32]) {
        match self.stereo_filter {
            StereoFilterType::None => {}
            StereoFilterType::Delay => {
                self.stereo_delay_state
                    .apply(samples, self.sample_rate, self.stereo_delay_ms);
            }
            StereoFilterType::Panning => {
                self.stereo_panning_state
                    .apply(samples, self.stereo_panning_angle_deg);
            }
            StereoFilterType::Comb => {
                self.stereo_comb_state.apply(
                    samples,
                    self.sample_rate,
                    self.stereo_comb_delay_ms,
                    self.stereo_comb_strength,
                );
            }
        }
    }
}

fn pole_coeff(sample_rate: f32, cutoff_hz: f32) -> f32 {
    (-2.0 * PI * cutoff_hz / sample_rate).exp()
}

fn pole_alpha(sample_rate: f32, cutoff_hz: f32) -> f32 {
    1.0 - pole_coeff(sample_rate, cutoff_hz)
}

fn dc_block(input: f32, last_in: &mut f32, last_out: &mut f32, coeff: f32) -> f32 {
    let out = input - *last_in + coeff * *last_out;
    *last_in = input;
    *last_out = out;
    out
}

fn high_pass(input: f32, last_in: &mut f32, state: &mut f32, coeff: f32) -> f32 {
    let out = coeff * (*state + input - *last_in);
    *last_in = input;
    *state = out;
    out
}

fn low_pass(input: f32, state: &mut f32, alpha: f32) -> f32 {
    *state += (input - *state) * alpha;
    *state
}

fn soft_clip(x: f32) -> f32 {
    // Keep the response linear in the main [-0.95, 0.95] range so the mixer
    // closely matches Mesen2's `GetOutputVolume() * 4` mapping, while still
    // providing a very soft knee near full scale to avoid harsh clipping on
    // rare transients that overshoot.
    const KNEE_START: f32 = 0.95;
    let abs = x.abs();

    if abs <= KNEE_START {
        x
    } else if abs >= 1.0 {
        x.signum()
    } else {
        let t = (abs - KNEE_START) / (1.0 - KNEE_START);
        let softened = KNEE_START + (1.0 - KNEE_START) * (1.0 - (1.0 - t).powi(2));
        softened.copysign(x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::{AudioChannel, CPU_CLOCK_NTSC};

    #[test]
    fn core_mixer_matches_mesen2_scaling_for_core_channels() {
        let mut mixer = NesSoundMixer::new(1_789_773.0, 48_000);

        // Representative core APU levels within the expected domains:
        // pulse/triangle/noise in 0..15, DMC in 0..127.
        let p1 = 10.0_f32;
        let p2 = 7.0_f32;
        let t = 12.0_f32;
        let n = 5.0_f32;
        let d = 80.0_f32;

        let clock = 0_i64;
        mixer.set_channel_level(AudioChannel::Pulse1, clock, p1);
        mixer.set_channel_level(AudioChannel::Pulse2, clock, p2);
        mixer.set_channel_level(AudioChannel::Triangle, clock, t);
        mixer.set_channel_level(AudioChannel::Noise, clock, n);
        mixer.set_channel_level(AudioChannel::Dmc, clock, d);

        // No expansion audio for this check.
        let (left, right) = mixer.mix_channels_stereo();

        // Reproduce Mesen2's GetOutputVolume() math for this scenario.
        let square_output = (p1 + p2) as f64;
        let tnd_lin = d as f64 + 2.751_671_326_1 * t as f64 + 1.849_358_712_5 * n as f64;

        let square_vol = if square_output > 0.0 {
            (95.88 * 5000.0) / (8128.0 / square_output + 100.0)
        } else {
            0.0
        };

        let tnd_vol = if tnd_lin > 0.0 {
            (159.79 * 5000.0) / (22638.0 / tnd_lin + 100.0)
        } else {
            0.0
        };

        let expected = (square_vol + tnd_vol) * (4.0 / 32_768.0);
        let expected_f32 = expected as f32;

        let diff_l = (left - expected_f32).abs();
        let diff_r = (right - expected_f32).abs();

        assert!(
            diff_l < 1e-4,
            "left mix {} vs expected {}",
            left,
            expected_f32
        );
        assert!(
            diff_r < 1e-4,
            "right mix {} vs expected {}",
            right,
            expected_f32
        );
    }

    #[test]
    fn expansion_weights_match_mesen2() {
        let mut mixer = NesSoundMixer::new(1_789_773.0, 48_000);

        let clock = 0_i64;
        // Use unit amplitudes so the expansion contribution is just the sum of the weights.
        mixer.set_channel_level(AudioChannel::Fds, clock, 1.0);
        mixer.set_channel_level(AudioChannel::Mmc5, clock, 1.0);
        mixer.set_channel_level(AudioChannel::Namco163, clock, 1.0);
        mixer.set_channel_level(AudioChannel::Sunsoft5B, clock, 1.0);
        mixer.set_channel_level(AudioChannel::Vrc6, clock, 1.0);
        mixer.set_channel_level(AudioChannel::Vrc7, clock, 1.0);

        let (left, right) = mixer.mix_channels_stereo();

        // In Mesen2, the expansion part of GetOutputVolume() is:
        // FDS*20 + MMC5*43 + N163*20 + S5B*15 + VRC6*5 + VRC7*1
        let exp_sum = (20.0 + 43.0 + 20.0 + 15.0 + 5.0 + 1.0) * (4.0 / 32_768.0);
        let expected_f32 = exp_sum as f32;

        let diff_l = (left - expected_f32).abs();
        let diff_r = (right - expected_f32).abs();

        assert!(
            diff_l < 1e-4,
            "left expansion {} vs expected {}",
            left,
            expected_f32
        );
        assert!(
            diff_r < 1e-4,
            "right expansion {} vs expected {}",
            right,
            expected_f32
        );
    }

    #[test]
    fn generates_reasonable_samples_for_ntsc_frame() {
        let clock_rate = CPU_CLOCK_NTSC;
        let sample_rate = 48_000_u32;
        let mut mixer = NesSoundMixer::new(clock_rate, sample_rate);

        // One NTSC video frame worth of CPU/APU cycles.
        let cycles_per_frame = (clock_rate / 60.0).round() as i64;
        let mut out = Vec::new();
        mixer.end_frame(cycles_per_frame, &mut out);

        // Stereo interleaved samples: expect roughly 2 * sample_rate / 60.
        let expected = ((sample_rate as f64 / 60.0).round() as usize) * 2;
        let len = out.len();
        assert!(len > 0, "no samples generated for one frame");

        let diff = if len > expected {
            len - expected
        } else {
            expected - len
        };

        // Allow a small rounding error margin.
        assert!(
            diff <= 8,
            "unexpected sample count for one frame: got {len}, expected ~{expected}"
        );
    }

    #[test]
    fn frame_timestamps_are_relative_between_calls() {
        let clock_rate = CPU_CLOCK_NTSC;
        let sample_rate = 48_000_u32;
        let cycles_per_frame = (clock_rate / 60.0).round() as i64;

        // Two frames processed separately with increasing absolute clocks.
        let mut mixer_split = NesSoundMixer::new(clock_rate, sample_rate);
        let mut split = Vec::new();
        mixer_split.end_frame(cycles_per_frame, &mut split);
        mixer_split.end_frame(cycles_per_frame * 2, &mut split);

        // Single call that covers the same total duration from clock 0.
        let mut mixer_single = NesSoundMixer::new(clock_rate, sample_rate);
        let mut single = Vec::new();
        mixer_single.end_frame(cycles_per_frame * 2, &mut single);

        let len_split = split.len();
        let len_single = single.len();
        assert!(len_split > 0 && len_single > 0);

        let diff = if len_split > len_single {
            len_split - len_single
        } else {
            len_single - len_split
        };

        // If `end_frame` wasn't using frame-relative durations, the split case
        // would accumulate 3x the frame time instead of 2x, and the sample
        // counts would diverge significantly. Keep a tight margin here.
        assert!(
            diff <= 8,
            "split/end_frame sample count mismatch: split={len_split}, single={len_single}"
        );
    }

    #[test]
    fn no_truncation_over_one_second_ntsc() {
        let clock_rate = CPU_CLOCK_NTSC;
        let sample_rate = 48_000_u32;
        let mut mixer = NesSoundMixer::new(clock_rate, sample_rate);

        let cycles_per_frame = (clock_rate / 60.0).round() as i64;
        let mut out = Vec::new();

        // Accumulate ~1 second of audio (60 NTSC frames) while flushing
        // blip_buf every frame.
        for frame in 1..=60 {
            let end_clock = cycles_per_frame * frame as i64;
            mixer.end_frame(end_clock, &mut out);
        }

        let expected = (sample_rate as usize) * 2; // stereo interleaved
        let len = out.len();

        let diff = if len > expected {
            len - expected
        } else {
            expected - len
        };

        // Allow a small margin for rounding between frames, but ensure we're
        // not silently dropping a large portion of samples.
        assert!(
            diff <= sample_rate as usize / 30,
            "unexpected sample count over ~1s: got {len}, expected ~{expected}"
        );
    }

    #[test]
    fn update_rates_changes_blip_and_filters() {
        let mut mixer = NesSoundMixer::new(CPU_CLOCK_NTSC, 44_100);

        let old_dc = mixer.dc_coeff;
        let old_rumble = mixer.rumble_coeff;
        let old_lowpass = mixer.lowpass_alpha;

        let clocks_needed_before = mixer.blip_left.clocks_needed(100);

        mixer.update_rates(CPU_CLOCK_NTSC, 48_000);

        let clocks_needed_after = mixer.blip_left.clocks_needed(100);

        assert_ne!(clocks_needed_before, clocks_needed_after);
        assert_ne!(old_dc, mixer.dc_coeff);
        assert_ne!(old_rumble, mixer.rumble_coeff);
        assert_ne!(old_lowpass, mixer.lowpass_alpha);
    }

    #[test]
    fn panning_toggle_clears_blip_buffer() {
        let mut mixer = NesSoundMixer::new(CPU_CLOCK_NTSC, 48_000);

        // Start with centered panning (default MixerSettings).
        let settings = MixerSettings::default();
        mixer.apply_mixer_settings(&settings);
        assert!(!mixer.has_panning);

        // Produce some samples in the blip buffer.
        mixer.add_delta(AudioChannel::Pulse1, 0, 1.0);
        mixer.blip_left.end_frame(100);
        mixer.blip_right.end_frame(100);
        assert!(mixer.blip_left.samples_avail() > 0);
        assert!(mixer.blip_right.samples_avail() > 0);

        // Enable per-channel panning for one channel; this should clear both
        // blip buffers the first time we leave the "all centered" state.
        let mut settings = MixerSettings::default();
        settings.panning[AudioChannel::Pulse1.idx()] = -1.0;
        mixer.apply_mixer_settings(&settings);

        assert!(mixer.has_panning);
        assert_eq!(mixer.blip_left.samples_avail(), 0);
        assert_eq!(mixer.blip_right.samples_avail(), 0);
    }
}

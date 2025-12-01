use std::f32::consts::PI;

use nesium_blip::BlipBuf;

use crate::audio::{
    AudioChannel,
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
    sample_rate: f32,
    last_frame_clock: i64,
    /// Last linear output level per channel (pulse1/2, triangle, noise, DMC, plus per-chip expansion).
    channel_levels: [f32; AudioChannel::COUNT],
    /// Per-channel volume and panning (0.0 = hard left, 1.0 = center, 2.0 = hard right).
    volumes: [f32; AudioChannel::COUNT],
    panning: [f32; AudioChannel::COUNT],
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

        let volumes = [1.0; AudioChannel::COUNT];
        let panning = [1.0; AudioChannel::COUNT];

        Self {
            blip_left: BlipBuf::new(clock_rate, sample_rate as f64, 24),
            blip_right: BlipBuf::new(clock_rate, sample_rate as f64, 24),
            sample_rate: sr,
            last_frame_clock: 0,
            channel_levels: [0.0; AudioChannel::COUNT],
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
            // do the rest. This value can be tweaked based on listening tests.
            master_gain: 0.9,
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
        self.channel_levels = [0.0; AudioChannel::COUNT];
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

    /// Apply per-channel volume and panning settings coming from the host.
    ///
    /// - `volume[i]` is expected in `[0.0, 1.0]` (0 = muted, 1 = full).
    /// - `panning[i]` is expected in `[-1.0, 1.0]` (-1 = hard left, 0 = center, 1 = hard right).
    pub fn apply_mixer_settings(&mut self, settings: &MixerSettings) {
        for (idx, (&vol, &pan)) in settings
            .volume
            .iter()
            .zip(settings.panning.iter())
            .enumerate()
        {
            self.volumes[idx] = vol.clamp(0.0, 1.0);
            // Map [-1, 1] to [0, 2] like Mesen2's (ChannelPanning + 100) / 100.
            self.panning[idx] = (pan.clamp(-1.0, 1.0) + 1.0).clamp(0.0, 2.0);
        }

        self.stereo_filter = settings.stereo_filter;
        self.stereo_delay_ms = settings.stereo_delay_ms.max(0.0);
        self.stereo_panning_angle_deg = settings.stereo_panning_angle_deg;
        self.stereo_comb_delay_ms = settings.stereo_comb_delay_ms.max(0.0);
        self.stereo_comb_strength = settings.stereo_comb_strength.clamp(0.0, 1.0);
    }

    /// Directly apply a channel delta at the given CPU/APU clock.
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
                    self.blip_left
                        .add_delta(rel_clock, delta_left * BLIP_SCALE);
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

            let l_dc = dc_block(
                l,
                &mut self.dc_last_input_l,
                &mut self.dc_last_output_l,
                self.dc_coeff,
            );
            let r_dc = dc_block(
                r,
                &mut self.dc_last_input_r,
                &mut self.dc_last_output_r,
                self.dc_coeff,
            );

            let l_rumble = high_pass(
                l_dc,
                &mut self.rumble_last_input_l,
                &mut self.rumble_state_l,
                self.rumble_coeff,
            );
            let r_rumble = high_pass(
                r_dc,
                &mut self.rumble_last_input_r,
                &mut self.rumble_state_r,
                self.rumble_coeff,
            );

            let l_smoothed =
                low_pass(l_rumble, &mut self.lowpass_state_l, self.lowpass_alpha);
            let r_smoothed =
                low_pass(r_rumble, &mut self.lowpass_state_r, self.lowpass_alpha);

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

        // Map to roughly [-1.0, 1.0] before downstream filtering/soft-clip.
        ((mixed_l / 8_192.0) as f32, (mixed_r / 8_192.0) as f32)
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
    // Gentle saturation to rein in transient spikes without audible pumping.
    (x * 1.05).tanh()
}


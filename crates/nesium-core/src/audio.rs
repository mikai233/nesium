use std::f32::consts::PI;

/// NTSC CPU clock rate (also used to drive the APU).
pub const CPU_CLOCK_NTSC: f64 = 1_789_773.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioChannel {
    Pulse1 = 0,
    Pulse2 = 1,
    Triangle = 2,
    Noise = 3,
    Dmc = 4,
    Expansion = 5,
}

impl AudioChannel {
    const COUNT: usize = 6;

    fn idx(self) -> usize {
        self as usize
    }
}

/// Band-limited NES mixer that accepts per-channel amplitude deltas tagged
/// with the CPU/APU clock time and produces filtered PCM at the host sample
/// rate.
#[derive(Debug)]
pub struct NesSoundMixer {
    resampler: BlipResampler,
    /// Last linear output level per channel (pulse1/2, triangle, noise, DMC, expansion).
    channel_levels: [f32; AudioChannel::COUNT],
    /// Cached mixed amplitude after the non-linear APU combiner.
    mixed_level: f32,
    /// DC-block filter state.
    dc_last_input: f32,
    dc_last_output: f32,
    /// Rumble high-pass state (~90 Hz).
    rumble_last_input: f32,
    rumble_state: f32,
    /// Soft low-pass to tame aliasing and harsh edges (~12 kHz).
    lowpass_state: f32,
    dc_coeff: f32,
    rumble_coeff: f32,
    lowpass_alpha: f32,
    master_gain: f32,
}

/// Simple band-limited step mixer inspired by blip_buf.
///
/// The mixer accepts amplitude deltas tagged with a clock-time (at some
/// `clock_rate`) and produces PCM samples at `sample_rate`, using a small
/// windowed-sinc kernel to spread each step over several output samples.
///
/// This keeps sharp edges and high-frequency content from turning into
/// audible aliasing and reduces clicks compared to naive per-tick sampling.
#[derive(Debug)]
pub struct BlipResampler {
    clock_rate: f64,
    sample_rate: f64,
    clock_to_sample: f64,
    kernel: Vec<f32>,
    half_width: i32,
    /// Accumulates samples that have not yet been handed to the caller.
    buffer: Vec<f32>,
    /// Total number of samples that have already been produced.
    produced_samples: i64,
    /// Integrator used to turn band-limited impulses into a step signal.
    integrator: f32,
}

impl BlipResampler {
    /// Constructs a new resampler.
    ///
    /// - `clock_rate`: input clock frequency in Hz (e.g. CPU/APU clock).
    /// - `sample_rate`: desired output sample rate in Hz (e.g. host audio).
    pub fn new(clock_rate: f64, sample_rate: f64) -> Self {
        let taps = 24; // Moderate kernel width similar to common blip_buf defaults.
        let kernel = make_kernel(taps);
        let half_width = (taps / 2) as i32;
        Self {
            clock_rate,
            sample_rate,
            clock_to_sample: sample_rate / clock_rate,
            kernel,
            half_width,
            buffer: Vec::new(),
            produced_samples: 0,
            integrator: 0.0,
        }
    }

    /// Clears any accumulated state and restarts the resampler from time zero.
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.produced_samples = 0;
        self.integrator = 0.0;
    }

    /// Adds an amplitude delta that occurs at the given `clock_time`.
    ///
    /// `clock_time` is an absolute tick count at `clock_rate`. The caller is
    /// expected to pass monotonically increasing times.
    pub fn add_delta(&mut self, clock_time: i64, delta: f32) {
        if delta == 0.0 {
            return;
        }

        let t = (clock_time as f64) * self.clock_to_sample;
        let base = t.floor() as i64;
        let taps = self.kernel.len() as i32;

        for k in 0..taps {
            let idx = base + k as i64 - self.half_width as i64;
            if idx < self.produced_samples {
                // This sample has already been consumed; skip it.
                continue;
            }
            let rel = (idx - self.produced_samples) as usize;
            if rel >= self.buffer.len() {
                self.buffer.resize(rel + 1, 0.0);
            }
            let w = self.kernel[k as usize];
            self.buffer[rel] += delta * w;
        }
    }

    /// Finalizes a frame that ends at `frame_end_clock` (absolute clock time)
    /// and appends all newly available samples into `out`.
    pub fn end_frame(&mut self, frame_end_clock: i64, out: &mut Vec<f32>) {
        let end_sample_pos = ((frame_end_clock as f64) * self.clock_to_sample).floor() as i64;
        if end_sample_pos <= self.produced_samples {
            return;
        }

        let to_output = (end_sample_pos - self.produced_samples) as usize;
        if to_output == 0 {
            return;
        }

        if self.buffer.len() < to_output {
            self.buffer.resize(to_output, 0.0);
        }

        out.reserve(to_output);
        for i in 0..to_output {
            self.integrator += self.buffer[i];
            out.push(self.integrator);
        }

        // Drop emitted samples but keep any tail that extends past the frame.
        self.buffer.drain(0..to_output);
        self.produced_samples = end_sample_pos;
    }
}

/// Generates a small symmetric low-pass kernel suitable for band-limited
/// step synthesis. This uses a windowed-sinc design with a Hamming window.
fn make_kernel(taps: usize) -> Vec<f32> {
    assert!(taps > 0);
    let mut kernel = Vec::with_capacity(taps);
    let center = (taps - 1) as f32 / 2.0;
    // Relative cutoff (0..1 where 1.0 = Nyquist). Use a slightly lower
    // cutoff to soften very sharp transitions and reduce clicks without
    // making the sound overly muffled.
    let cutoff = 0.35;

    for i in 0..taps {
        let x = i as f32 - center;
        let sinc = if x.abs() < 1e-6 {
            1.0
        } else {
            let arg = PI * x * cutoff;
            arg.sin() / arg
        };
        let w = 0.54 - 0.46 * (2.0 * PI * i as f32 / (taps - 1) as f32).cos();
        kernel.push((sinc * w) as f32);
    }

    // Normalize so that a DC step settles near the target amplitude.
    let sum: f32 = kernel.iter().sum();
    if sum.abs() > 1e-6 {
        for k in &mut kernel {
            *k /= sum;
        }
    }
    kernel
}

impl NesSoundMixer {
    /// Construct a mixer for the given CPU/APU clock and host sample rate.
    pub fn new(clock_rate: f64, sample_rate: u32) -> Self {
        let sr = sample_rate as f32;
        // Cut DC gently to keep bass body while clearing offsets.
        let dc_cut = 5.0_f32;
        // Trim sub-bass rumble to keep percussive clicks from bleeding through.
        let rumble_cut = 120.0_f32;
        // Pull treble a bit lower to tame hiss while keeping clarity.
        let lowpass_cut = 12_000.0_f32;

        let dc_coeff = pole_coeff(sr, dc_cut);
        let rumble_coeff = pole_coeff(sr, rumble_cut);
        let lowpass_alpha = pole_alpha(sr, lowpass_cut);

        Self {
            resampler: BlipResampler::new(clock_rate, sample_rate as f64),
            channel_levels: [0.0; AudioChannel::COUNT],
            mixed_level: 0.0,
            dc_last_input: 0.0,
            dc_last_output: 0.0,
            rumble_last_input: 0.0,
            rumble_state: 0.0,
            lowpass_state: 0.0,
            dc_coeff,
            rumble_coeff,
            lowpass_alpha,
            master_gain: 1.0, // Keep headroom to avoid clipping on sharp transients.
        }
    }

    /// Reset all accumulated state while keeping configuration.
    pub fn reset(&mut self) {
        self.resampler.reset();
        self.channel_levels = [0.0; AudioChannel::COUNT];
        self.mixed_level = 0.0;
        self.dc_last_input = 0.0;
        self.dc_last_output = 0.0;
        self.rumble_last_input = 0.0;
        self.rumble_state = 0.0;
        self.lowpass_state = 0.0;
    }

    /// Directly apply a channel delta at the given CPU/APU clock.
    pub fn add_delta(&mut self, channel: AudioChannel, clock_time: i64, delta: f32) {
        if delta == 0.0 {
            return;
        }
        let idx = channel.idx();
        self.channel_levels[idx] += delta;

        let mixed = self.mix_channels();
        let mixed_delta = mixed - self.mixed_level;
        if mixed_delta != 0.0 {
            self.resampler.add_delta(clock_time, mixed_delta);
            self.mixed_level = mixed;
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
        let mut temp = Vec::new();
        self.resampler.end_frame(frame_end_clock, &mut temp);
        out.reserve(temp.len());

        for sample in temp {
            let dc_blocked = dc_block(
                sample,
                &mut self.dc_last_input,
                &mut self.dc_last_output,
                self.dc_coeff,
            );
            let rumble = high_pass(
                dc_blocked,
                &mut self.rumble_last_input,
                &mut self.rumble_state,
                self.rumble_coeff,
            );
            let smoothed = low_pass(rumble, &mut self.lowpass_state, self.lowpass_alpha);
            let scaled = soft_clip(smoothed * self.master_gain);
            out.push(scaled);
        }
    }

    fn mix_channels(&self) -> f32 {
        let p1 = self.channel_levels[AudioChannel::Pulse1.idx()] as f64;
        let p2 = self.channel_levels[AudioChannel::Pulse2.idx()] as f64;
        let t = self.channel_levels[AudioChannel::Triangle.idx()] as f64;
        // De-emphasize noise and DMC to keep hiss/edge transients down.
        let n = (self.channel_levels[AudioChannel::Noise.idx()] as f64) * 0.4;
        let d = (self.channel_levels[AudioChannel::Dmc.idx()] as f64) * 0.7;
        let expansion = (self.channel_levels[AudioChannel::Expansion.idx()] as f64) * 0.25;

        let pulse_out = if p1 == 0.0 && p2 == 0.0 {
            0.0
        } else {
            95.88 / ((8128.0 / (p1 + p2)) + 100.0)
        };

        let tnd_out = if t == 0.0 && n == 0.0 && d == 0.0 {
            0.0
        } else {
            159.79 / ((1.0 / (t / 8227.0 + n / 12241.0 + d / 22638.0)) + 100.0)
        };

        // Expansion audio kept modest; per-mapper scaling can refine further.
        (pulse_out + tnd_out + expansion) as f32
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

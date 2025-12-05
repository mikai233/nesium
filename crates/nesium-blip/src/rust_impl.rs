//! BlipBuf — Rust port of Shay Green's blip_buf (1.1.0) with matching timing/filter logic.
//!
//! - Original C source: http://www.slack.net/~ant/blip_buf.html
//! - License: LGPL-2.1; see `vendor/LGPL.txt` in the crate root.
//! - API mirrors the C library: add clock-tagged deltas, call [`end_frame`],
//!   then pull samples with [`read_samples`] / [`read_samples_i16`]. Internally
//!   this keeps the same fixed-point resampling, kernel, and high-pass steps as
//!   the reference implementation.

use core::cmp::min;

const PRE_SHIFT: usize = 32;
const TIME_BITS: usize = PRE_SHIFT + 20;
const TIME_UNIT: u64 = 1u64 << TIME_BITS;
const BASS_SHIFT: usize = 9;
const END_FRAME_EXTRA: usize = 2;
const HALF_WIDTH: usize = 8;
const BUF_EXTRA: usize = HALF_WIDTH * 2 + END_FRAME_EXTRA;
const PHASE_BITS: usize = 5;
const PHASE_COUNT: usize = 1 << PHASE_BITS;
const DELTA_BITS: usize = 15;
const DELTA_UNIT: usize = 1 << DELTA_BITS;
const FRAC_BITS: usize = TIME_BITS - PRE_SHIFT;
const BLIP_MAX_RATIO: u64 = 1 << 20;

// Sinc_Generator(0.9, 0.55, 4.5)
const BL_STEP: [[i16; HALF_WIDTH]; PHASE_COUNT + 1] = [
    [43, -115, 350, -488, 1136, -914, 5861, 21022],
    [44, -118, 348, -473, 1076, -799, 5274, 21001],
    [45, -121, 344, -454, 1011, -677, 4706, 20936],
    [46, -122, 336, -431, 942, -549, 4156, 20829],
    [47, -123, 327, -404, 868, -418, 3629, 20679],
    [47, -122, 316, -375, 792, -285, 3124, 20488],
    [47, -120, 303, -344, 714, -151, 2644, 20256],
    [46, -117, 289, -310, 634, -17, 2188, 19985],
    [46, -114, 273, -275, 553, 117, 1758, 19675],
    [44, -108, 255, -237, 471, 247, 1356, 19327],
    [43, -103, 237, -199, 390, 373, 981, 18944],
    [42, -98, 218, -160, 310, 495, 633, 18527],
    [40, -91, 198, -121, 231, 611, 314, 18078],
    [38, -84, 178, -81, 153, 722, 22, 17599],
    [36, -76, 157, -43, 80, 824, -241, 17092],
    [34, -68, 135, -3, 8, 919, -476, 16558],
    [32, -61, 115, 34, -60, 1006, -683, 16001],
    [29, -52, 94, 70, -123, 1083, -862, 15422],
    [27, -44, 73, 106, -184, 1152, -1015, 14824],
    [25, -36, 53, 139, -239, 1211, -1142, 14210],
    [22, -27, 34, 170, -290, 1261, -1244, 13582],
    [20, -20, 16, 199, -335, 1301, -1322, 12942],
    [18, -12, -3, 226, -375, 1331, -1376, 12293],
    [15, -4, -19, 250, -410, 1351, -1408, 11638],
    [13, 3, -35, 272, -439, 1361, -1419, 10979],
    [11, 9, -49, 292, -464, 1362, -1410, 10319],
    [9, 16, -63, 309, -483, 1354, -1383, 9660],
    [7, 22, -75, 322, -496, 1337, -1339, 9005],
    [6, 26, -85, 333, -504, 1312, -1280, 8355],
    [4, 31, -94, 341, -507, 1278, -1205, 7713],
    [3, 35, -102, 347, -506, 1238, -1119, 7082],
    [1, 40, -110, 350, -499, 1190, -1021, 6464],
    [0, 43, -115, 350, -488, 1136, -914, 5861],
];

/// A band-limited buffer that converts deltas at a source clock rate
/// into PCM samples at a fixed output sample rate.
#[derive(Debug, Clone)]
pub struct BlipBuf {
    factor: u64,
    offset: u64,
    avail: usize,
    size: usize,
    integrator: i32,
    buf: Vec<i32>,
}

impl BlipBuf {
    /// Construct a new buffer with the given rates.
    ///
    /// The third parameter is kept for API compatibility and is treated as
    /// a minimum buffer size hint; the actual capacity is at least one
    /// second of audio at the output sample rate.
    pub fn new(clock_rate: f64, sample_rate: f64, min_buffer_samples: usize) -> Self {
        assert!(clock_rate > 0.0, "clock_rate must be positive");
        assert!(sample_rate > 0.0, "sample_rate must be positive");
        assert!(
            clock_rate <= sample_rate * BLIP_MAX_RATIO as f64,
            "clock_rate/sample_rate exceeds blip_max_ratio"
        );

        let size = min_buffer_samples.max(sample_rate.ceil() as usize).max(1);
        let default_factor = TIME_UNIT / BLIP_MAX_RATIO;

        let mut this = Self {
            factor: default_factor,
            offset: default_factor / 2,
            avail: 0,
            size,
            integrator: 0,
            buf: vec![0; size + BUF_EXTRA],
        };

        // Match C behavior: set_rates updates factor but leaves offset as-is;
        // the initial offset comes from blip_new()'s default factor.
        this.set_rates(clock_rate, sample_rate);
        this
    }

    /// Reconfigure the input and output rates.
    ///
    /// This matches `blip_set_rates()` and preserves buffered samples.
    pub fn set_rates(&mut self, clock_rate: f64, sample_rate: f64) {
        assert!(clock_rate > 0.0);
        assert!(sample_rate > 0.0);
        assert!(
            clock_rate <= sample_rate * BLIP_MAX_RATIO as f64,
            "clock_rate/sample_rate exceeds blip_max_ratio"
        );
        self.factor = Self::compute_factor(clock_rate, sample_rate);
    }

    /// Clears all buffered samples.
    pub fn clear(&mut self) {
        self.offset = self.factor / 2;
        self.avail = 0;
        self.integrator = 0;
        self.buf.fill(0);
    }

    /// Number of buffered samples available for reading.
    pub fn samples_avail(&self) -> usize {
        self.avail
    }

    /// Length of time frame (in clocks) needed to make `sample_count`
    /// additional samples available.
    pub fn clocks_needed(&self, sample_count: usize) -> i64 {
        assert!(
            self.avail + sample_count <= self.size,
            "requested samples exceed buffer capacity"
        );
        let needed = (sample_count as u64) * TIME_UNIT;
        if needed < self.offset {
            return 0;
        }
        let clocks = (needed - self.offset).div_ceil(self.factor);
        clocks as i64
    }

    /// Adds a delta into the buffer at the specified frame-relative clock.
    pub fn add_delta(&mut self, clock_time: i64, delta: f32) {
        let delta = delta.round() as i32;
        if delta == 0 {
            return;
        }
        assert!(clock_time >= 0, "clock_time must be non-negative");

        let fixed = (((clock_time as u64).wrapping_mul(self.factor)).wrapping_add(self.offset))
            >> PRE_SHIFT;
        let out_index = self.avail + ((fixed >> FRAC_BITS) as usize);
        assert!(
            out_index <= self.size + END_FRAME_EXTRA,
            "blip_buf overflow in add_delta"
        );

        let phase_shift = FRAC_BITS - PHASE_BITS;
        let phase = ((fixed >> phase_shift) & (PHASE_COUNT as u64 - 1)) as usize;
        let interp_mask = DELTA_UNIT as u64 - 1;
        let interp = ((fixed >> (phase_shift - DELTA_BITS)) & interp_mask) as i32;
        let delta2 = (delta * interp) >> DELTA_BITS;
        let delta1 = delta - delta2;

        let in0 = &BL_STEP[phase];
        let in1 = &BL_STEP[phase + 1];
        for k in 0..HALF_WIDTH {
            let inc = (in0[k] as i32) * delta1 + (in1[k] as i32) * delta2;
            self.buf[out_index + k] = self.buf[out_index + k].wrapping_add(inc);
        }

        let rev = &BL_STEP[PHASE_COUNT - phase];
        let rev_prev = &BL_STEP[PHASE_COUNT - phase - 1];
        for k in 0..HALF_WIDTH {
            let idx = HALF_WIDTH - 1 - k;
            let inc = (rev[idx] as i32) * delta1 + (rev_prev[idx] as i32) * delta2;
            self.buf[out_index + HALF_WIDTH + k] =
                self.buf[out_index + HALF_WIDTH + k].wrapping_add(inc);
        }
    }

    /// Faster, lower-quality version of [`add_delta`].
    pub fn add_delta_fast(&mut self, clock_time: i64, delta: f32) {
        let delta = delta.round() as i32;
        if delta == 0 {
            return;
        }
        assert!(clock_time >= 0, "clock_time must be non-negative");

        let fixed = (((clock_time as u64).wrapping_mul(self.factor)).wrapping_add(self.offset))
            >> PRE_SHIFT;
        let out_index = self.avail + ((fixed >> FRAC_BITS) as usize);
        assert!(
            out_index <= self.size + END_FRAME_EXTRA,
            "blip_buf overflow in add_delta_fast"
        );

        let interp = ((fixed >> (FRAC_BITS - DELTA_BITS)) & (DELTA_UNIT as u64 - 1)) as i32;
        let delta2 = delta * interp;
        let delta1 = delta * DELTA_UNIT as i32 - delta2;

        self.buf[out_index + HALF_WIDTH - 1] =
            self.buf[out_index + HALF_WIDTH - 1].wrapping_add(delta1);
        self.buf[out_index + HALF_WIDTH] = self.buf[out_index + HALF_WIDTH].wrapping_add(delta2);
    }

    /// Makes clocks before `clock_duration` available as output samples.
    pub fn end_frame(&mut self, clock_duration: i64) {
        assert!(clock_duration >= 0, "clock_duration must be non-negative");
        let off = (clock_duration as u64)
            .saturating_mul(self.factor)
            .saturating_add(self.offset);
        self.avail += (off >> TIME_BITS) as usize;
        self.offset = off & (TIME_UNIT - 1);

        assert!(
            self.avail <= self.size,
            "blip_buf overflow in end_frame: avail {} > size {}",
            self.avail,
            self.size
        );
    }

    /// Reads up to `out.len()` samples as f32 in roughly [-1.0, 1.0].
    pub fn read_samples(&mut self, out: &mut [f32]) -> usize {
        let count = min(out.len(), self.avail);
        if count == 0 {
            return 0;
        }

        let mut temp = vec![0i16; count];
        let produced = self.read_samples_i16(&mut temp);
        for (dst, src) in out.iter_mut().zip(temp.into_iter()) {
            *dst = src as f32 / 32768.0;
        }
        produced
    }

    /// Reads up to `out.len()` samples into 16-bit PCM.
    pub fn read_samples_i16(&mut self, out: &mut [i16]) -> usize {
        let count = min(out.len(), self.avail);
        if count == 0 {
            return 0;
        }

        let mut sum = self.integrator;
        for (dst, in_sample) in out.iter_mut().take(count).zip(self.buf.iter()) {
            let mut s = sum >> DELTA_BITS;
            sum = sum.wrapping_add(*in_sample);

            s = clamp_to_i16_c_style(s);

            *dst = s as i16;
            sum = sum.wrapping_sub(s << (DELTA_BITS - BASS_SHIFT));
        }

        self.integrator = sum;
        self.remove_samples(count);
        count
    }

    /// Reads up to `out.len()/2` stereo samples (interleaved). Matches the C API stereo path.
    pub fn read_samples_i16_stereo(&mut self, out: &mut [i16]) -> usize {
        let usable = out.len() / 2;
        let count = min(usable, self.avail);
        if count == 0 {
            return 0;
        }

        let mut sum = self.integrator;
        for (idx, in_sample) in self.buf.iter().take(count).enumerate() {
            let mut s = sum >> DELTA_BITS;
            sum = sum.wrapping_add(*in_sample);

            s = clamp_to_i16_c_style(s);

            out[idx * 2] = s as i16;
            sum = sum.wrapping_sub(s << (DELTA_BITS - BASS_SHIFT));
        }

        self.integrator = sum;
        self.remove_samples(count);
        count
    }

    fn remove_samples(&mut self, count: usize) {
        let old_avail = self.avail;
        let remain = old_avail + BUF_EXTRA - count;
        self.avail = old_avail - count;

        self.buf.copy_within(count..count + remain, 0);
        for v in &mut self.buf[remain..remain + count] {
            *v = 0;
        }
    }

    fn compute_factor(clock_rate: f64, sample_rate: f64) -> u64 {
        let mut factor = (TIME_UNIT as f64 * sample_rate / clock_rate) as u64;
        if (factor as f64) < (TIME_UNIT as f64 * sample_rate / clock_rate) {
            factor += 1;
        }
        factor
    }
}

#[inline]
fn clamp_to_i16_c_style(s: i32) -> i32 {
    // Match blip_buf's CLAMP macro: if casting to i16 changes the value,
    // use (s >> 16) ^ 0x7FFF instead of saturating.
    if (s as i16 as i32) != s {
        (s >> 16) ^ 0x7FFF
    } else {
        s
    }
}

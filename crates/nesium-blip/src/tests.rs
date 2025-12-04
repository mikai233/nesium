use crate::{c_impl::BlipBuf, rust_impl::BlipBuf as RustBlipBuf};
use proptest::prelude::*;

const CLOCK_RATE: f64 = 1_789_773.0;
const SAMPLE_RATE: f64 = 48_000.0;
const BUF_SIZE: usize = 4096;

#[derive(Clone, Debug)]
struct Op {
    at: u32,
    delta: i32,
    fast: bool,
}

#[derive(Clone, Debug)]
struct Frame {
    samples: usize,
    ops: Vec<Op>,
}

fn frame_strategy() -> impl Strategy<Value = Frame> {
    (
        1usize..=600,
        prop::collection::vec((0u32..=5_000, -20_000i32..=20_000i32, any::<bool>()), 0..10),
    )
        .prop_map(|(samples, ops)| Frame {
            samples,
            ops: ops
                .into_iter()
                .map(|(at, delta, fast)| Op { at, delta, fast })
                .collect(),
        })
}

proptest! {
    #[test]
    fn ffi_and_rust_match_randomized_frames(frames in prop::collection::vec(frame_strategy(), 1..6)) {
        let mut c = BlipBuf::new(CLOCK_RATE, SAMPLE_RATE, BUF_SIZE);
        let mut r = RustBlipBuf::new(CLOCK_RATE, SAMPLE_RATE, BUF_SIZE);

        for frame in frames {
            let samples = frame.samples.min(BUF_SIZE.saturating_sub(1)).max(1);
            let clock_duration = c.clocks_needed(samples) as i64;
            let clock_duration = clock_duration.max(1);

            for op in &frame.ops {
                let t = if clock_duration == 0 { 0 } else { op.at % (clock_duration as u32) };
                if op.fast {
                    c.add_delta_fast(t as i64, op.delta as f32);
                    r.add_delta_fast(t as i64, op.delta as f32);
                } else {
                    c.add_delta(t as i64, op.delta as f32);
                    r.add_delta(t as i64, op.delta as f32);
                }
            }

            c.end_frame(clock_duration);
            r.end_frame(clock_duration);

            prop_assert_eq!(c.samples_avail(), r.samples_avail(), "available sample count diverged");

            let mut c_out = vec![0i16; c.samples_avail()];
            let mut r_out = vec![0i16; r.samples_avail()];
            let c_len = c.read_samples_i16(&mut c_out);
            let r_len = r.read_samples_i16(&mut r_out);
            c_out.truncate(c_len);
            r_out.truncate(r_len);

            prop_assert_eq!(c_len, r_len, "produced length diverged");
            prop_assert_eq!(c_out, r_out, "PCM output diverged");
        }
    }
}

proptest! {
    #[test]
    fn ffi_and_rust_match_with_rate_change(frames in prop::collection::vec(frame_strategy(), 2..4)) {
        let mut c = BlipBuf::new(CLOCK_RATE, SAMPLE_RATE, BUF_SIZE);
        let mut r = RustBlipBuf::new(CLOCK_RATE, SAMPLE_RATE, BUF_SIZE);

        for (idx, frame) in frames.into_iter().enumerate() {
            if idx % 2 == 1 {
                let clock = CLOCK_RATE * (1.0 + (idx as f64) * 0.0001);
                let sample = SAMPLE_RATE * (1.0 + (idx as f64) * 0.0002);
                c.set_rates(clock, sample);
                r.set_rates(clock, sample);
            }

            let samples = frame.samples.min(BUF_SIZE.saturating_sub(1)).max(1);
            let clock_duration = c.clocks_needed(samples) as i64;
            let clock_duration = clock_duration.max(1);

            for op in &frame.ops {
                let t = if clock_duration == 0 { 0 } else { op.at % (clock_duration as u32) };
                if op.fast {
                    c.add_delta_fast(t as i64, op.delta as f32);
                    r.add_delta_fast(t as i64, op.delta as f32);
                } else {
                    c.add_delta(t as i64, op.delta as f32);
                    r.add_delta(t as i64, op.delta as f32);
                }
            }

            c.end_frame(clock_duration);
            r.end_frame(clock_duration);

            prop_assert_eq!(c.samples_avail(), r.samples_avail(), "available sample count diverged");
            let mut c_out = vec![0i16; c.samples_avail()];
            let mut r_out = vec![0i16; r.samples_avail()];
            let c_len = c.read_samples_i16(&mut c_out);
            let r_len = r.read_samples_i16(&mut r_out);
            c_out.truncate(c_len);
            r_out.truncate(r_len);
            prop_assert_eq!(c_out, r_out, "PCM output diverged after rate change");
        }
    }
}

proptest! {
    #[test]
    fn ffi_and_rust_clear_and_stereo(frames in prop::collection::vec(frame_strategy(), 1..3)) {
        let mut c = BlipBuf::new(CLOCK_RATE, SAMPLE_RATE, BUF_SIZE);
        let mut r = RustBlipBuf::new(CLOCK_RATE, SAMPLE_RATE, BUF_SIZE);

        for (idx, frame) in frames.into_iter().enumerate() {
            let samples = frame.samples.min(BUF_SIZE.saturating_sub(1)).max(1);
            let clock_duration = c.clocks_needed(samples) as i64;
            let clock_duration = clock_duration.max(1);

            for op in &frame.ops {
                let t = if clock_duration == 0 { 0 } else { op.at % (clock_duration as u32) };
                if op.fast {
                    c.add_delta_fast(t as i64, op.delta as f32);
                    r.add_delta_fast(t as i64, op.delta as f32);
                } else {
                    c.add_delta(t as i64, op.delta as f32);
                    r.add_delta(t as i64, op.delta as f32);
                }
            }

            c.end_frame(clock_duration);
            r.end_frame(clock_duration);

            let mut c_out = vec![0i16; c.samples_avail() * 2 + 2];
            let mut r_out = vec![0i16; r.samples_avail() * 2 + 2];
            let c_len = c.read_samples_i16_stereo(&mut c_out);
            let r_len = r.read_samples_i16_stereo(&mut r_out);
            prop_assert_eq!(c_len, r_len, "stereo length diverged");
            c_out.truncate((c_len as usize) * 2);
            r_out.truncate((r_len as usize) * 2);
            prop_assert_eq!(c_out, r_out, "stereo PCM diverged");

            if idx % 2 == 0 {
                c.clear();
                r.clear();
                prop_assert_eq!(c.samples_avail(), r.samples_avail(), "clear mismatch");
            }
        }
    }
}

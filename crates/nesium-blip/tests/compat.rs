use nesium_blip::{BlipBuf, RustBlipBuf};

const CLOCK_RATE: f64 = 1_789_773.0;
const SAMPLE_RATE: f64 = 48_000.0;
const BUF_SIZE: usize = 4096;

#[test]
fn rust_port_stays_close_to_c_impl() {
    let mut c = BlipBuf::new(CLOCK_RATE, SAMPLE_RATE, BUF_SIZE);
    c.add_delta(0, 16_384.0);
    c.add_delta(100, -8_192.0);
    c.add_delta_fast(400, 12_000.0);
    c.add_delta(800, -4_096.0);
    c.add_delta(1_200, 6_000.0);
    c.end_frame(2_000);

    let mut rust = RustBlipBuf::new(CLOCK_RATE, SAMPLE_RATE, BUF_SIZE);
    rust.add_delta(0, 16_384.0);
    rust.add_delta(100, -8_192.0);
    rust.add_delta_fast(400, 12_000.0);
    rust.add_delta(800, -4_096.0);
    rust.add_delta(1_200, 6_000.0);
    rust.end_frame(2_000);

    let mut c_out = vec![0i16; c.samples_avail()];
    let c_count = c.read_samples_i16(&mut c_out);
    c_out.truncate(c_count);

    let mut r_out = vec![0i16; rust.samples_avail()];
    let r_count = rust.read_samples_i16(&mut r_out);
    r_out.truncate(r_count);

    assert_eq!(
        c_out.len(),
        r_out.len(),
        "length mismatch (c={}, rust={})",
        c_out.len(),
        r_out.len()
    );

    let mut max_diff = 0;
    for (&c, &r) in c_out.iter().zip(r_out.iter()) {
        let diff = (c as i32 - r as i32).abs();
        if diff > max_diff {
            max_diff = diff;
        }
    }

    assert!(max_diff <= 200, "max sample diff too large: {max_diff}");
}

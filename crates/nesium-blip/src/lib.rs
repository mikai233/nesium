#![deny(unsafe_op_in_unsafe_fn)]

//! Bindings and wrappers for Shay Green's blip_buf.
//!
//! - C implementation is compiled from `csrc/blip_buf.c` under LGPL-2.1
//!   (see `csrc/license.md`).
//! - `BlipBuf` uses the C implementation via FFI.
//! - `RustBlipBuf` is the Rust port, kept here for comparison and future
//!   iteration.

use std::ptr::NonNull;

pub mod rust_impl;

pub use rust_impl::BlipBuf as RustBlipBuf;

#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

/// Low-level wrapper that maps 1:1 onto the C blip_buf API.
///
/// Types and semantics follow `blip_buf.h`:
/// - `sample_count`, `sample_count` in `clocks_needed` are `int` (i32).
/// - `clock_time` / `clock_duration` are `unsigned int` (u32).
/// - `delta` is `int` (i32).
#[derive(Debug)]
pub struct BlipBufRaw {
    raw: NonNull<ffi::blip_t>,
}

// The underlying C blip_t is an opaque heap allocation that is only ever
// accessed through &mut self; transferring ownership between threads is fine,
// but sharing is not, so we only implement Send, not Sync.
unsafe impl Send for BlipBufRaw {}

impl BlipBufRaw {
    /// Creates a new buffer that can hold at most `sample_count` samples.
    pub fn new(sample_count: i32) -> Self {
        assert!(sample_count >= 0, "sample_count must be non-negative");
        let raw = unsafe { ffi::blip_new(sample_count) };
        let raw = NonNull::new(raw).expect("blip_new returned null");
        Self { raw }
    }

    /// Sets approximate input clock rate and output sample rate.
    pub fn set_rates(&mut self, clock_rate: f64, sample_rate: f64) {
        unsafe { ffi::blip_set_rates(self.raw.as_ptr(), clock_rate, sample_rate) };
    }

    /// Clears entire buffer.
    pub fn clear(&mut self) {
        unsafe { ffi::blip_clear(self.raw.as_ptr()) };
    }

    /// Length of time frame, in clocks, needed to make `sample_count`
    /// additional samples available.
    pub fn clocks_needed(&self, sample_count: i32) -> i32 {
        unsafe { ffi::blip_clocks_needed(self.raw.as_ptr(), sample_count) }
    }

    /// Makes input clocks before `clock_duration` available for reading.
    pub fn end_frame(&mut self, clock_duration: u32) {
        unsafe { ffi::blip_end_frame(self.raw.as_ptr(), clock_duration) };
    }

    /// Number of buffered samples available for reading.
    pub fn samples_avail(&self) -> i32 {
        unsafe { ffi::blip_samples_avail(self.raw.as_ptr()) }
    }

    /// Adds positive/negative delta into buffer at specified clock time.
    pub fn add_delta(&mut self, clock_time: u32, delta: i32) {
        unsafe { ffi::blip_add_delta(self.raw.as_ptr(), clock_time, delta) };
    }

    /// Same as [`add_delta`], but uses faster, lower-quality synthesis.
    pub fn add_delta_fast(&mut self, clock_time: u32, delta: i32) {
        unsafe { ffi::blip_add_delta_fast(self.raw.as_ptr(), clock_time, delta) };
    }

    /// Reads and removes at most `out.len()` samples into `out`.
    ///
    /// If `stereo` is true, writes into every other element of `out` as in
    /// the C API.
    pub fn read_samples_i16(&mut self, out: &mut [i16], stereo: bool) -> i32 {
        let stereo_flag = if stereo { 1 } else { 0 };
        unsafe {
            ffi::blip_read_samples(
                self.raw.as_ptr(),
                out.as_mut_ptr(),
                out.len() as i32,
                stereo_flag,
            )
        }
    }
}

impl Drop for BlipBufRaw {
    fn drop(&mut self) {
        unsafe {
            ffi::blip_delete(self.raw.as_ptr());
        }
    }
}

/// Higher-level wrapper that uses the C implementation internally but exposes
/// a more ergonomic Rust API (f32 samples, usize counts, etc.).
#[derive(Debug)]
pub struct BlipBuf {
    raw: BlipBufRaw,
    capacity: usize,
}

unsafe impl Send for BlipBuf {}

impl BlipBuf {
    /// Construct a new buffer with the given rates.
    ///
    /// `min_buffer_samples` is used as a lower bound; the actual capacity is
    /// at least one second at the output sample rate to avoid overflow.
    pub fn new(clock_rate: f64, sample_rate: f64, min_buffer_samples: usize) -> Self {
        let size = min_buffer_samples.max(sample_rate.ceil() as usize).max(1);
        let mut raw = BlipBufRaw::new(size as i32);
        raw.set_rates(clock_rate, sample_rate);
        Self {
            raw,
            capacity: size,
        }
    }

    pub fn set_rates(&mut self, clock_rate: f64, sample_rate: f64) {
        self.raw.set_rates(clock_rate, sample_rate);
    }

    pub fn clear(&mut self) {
        self.raw.clear();
    }

    pub fn samples_avail(&self) -> usize {
        self.raw.samples_avail().max(0) as usize
    }

    pub fn clocks_needed(&self, sample_count: usize) -> i64 {
        assert!(
            self.samples_avail() + sample_count <= self.capacity,
            "requested samples exceed buffer capacity"
        );
        self.raw.clocks_needed(sample_count as i32).into()
    }

    pub fn add_delta(&mut self, clock_time: i64, delta: f32) {
        if delta == 0.0 {
            return;
        }
        assert!(clock_time >= 0, "clock_time must be non-negative");
        let delta = delta.round() as i32;
        self.raw.add_delta(clock_time as u32, delta);
    }

    pub fn add_delta_fast(&mut self, clock_time: i64, delta: f32) {
        if delta == 0.0 {
            return;
        }
        assert!(clock_time >= 0, "clock_time must be non-negative");
        let delta = delta.round() as i32;
        self.raw.add_delta_fast(clock_time as u32, delta);
    }

    pub fn end_frame(&mut self, clock_duration: i64) {
        assert!(clock_duration >= 0, "clock_duration must be non-negative");
        self.raw.end_frame(clock_duration as u32);
    }

    pub fn read_samples(&mut self, out: &mut [f32]) -> usize {
        let mut temp = vec![0i16; out.len()];
        let count = self.read_samples_i16(&mut temp);
        for (dst, src) in out.iter_mut().zip(temp.into_iter()) {
            *dst = src as f32 / 32768.0;
        }
        count
    }

    pub fn read_samples_i16(&mut self, out: &mut [i16]) -> usize {
        let count = self.raw.read_samples_i16(out, false);
        count.max(0) as usize
    }
}

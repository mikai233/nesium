#![allow(dead_code)]

use crate::raw;
use std::{
    ffi::c_void,
    sync::atomic::{AtomicUsize, Ordering},
};

/// Collection of callbacks provided by the libretro frontend.
#[derive(Clone, Copy, Default)]
pub(crate) struct CallbackSet {
    pub environment: raw::retro_environment_t,
    pub video: raw::retro_video_refresh_t,
    pub audio_sample: raw::retro_audio_sample_t,
    pub audio_batch: raw::retro_audio_sample_batch_t,
    pub input_poll: raw::retro_input_poll_t,
    pub input_state: raw::retro_input_state_t,
}

impl CallbackSet {
    pub fn set_environment(&mut self, cb: raw::retro_environment_t) {
        self.environment = cb;
    }

    pub fn set_video(&mut self, cb: raw::retro_video_refresh_t) {
        self.video = cb;
    }

    pub fn set_audio_sample(&mut self, cb: raw::retro_audio_sample_t) {
        self.audio_sample = cb;
    }

    pub fn set_audio_batch(&mut self, cb: raw::retro_audio_sample_batch_t) {
        self.audio_batch = cb;
    }

    pub fn set_input_poll(&mut self, cb: raw::retro_input_poll_t) {
        self.input_poll = cb;
    }

    pub fn set_input_state(&mut self, cb: raw::retro_input_state_t) {
        self.input_state = cb;
    }
}

/// Safe wrappers around the callbacks that libretro frontends provide.
pub struct RuntimeHandles {
    callbacks: CallbackSet,
    frame_counter: AtomicUsize,
}

impl RuntimeHandles {
    /// Creates a new [`RuntimeHandles`] from the callbacks stored in [`CallbackSet`].
    pub(crate) fn new(callbacks: CallbackSet) -> Self {
        Self {
            callbacks,
            frame_counter: AtomicUsize::new(0),
        }
    }

    /// Returns the environment callback if the frontend installed one.
    pub fn environment(&self) -> Option<Environment> {
        self.callbacks
            .environment
            .map(|cb| Environment { callback: cb })
    }

    /// Returns the video callback if the frontend installed one.
    pub fn video(&self) -> Option<Video<'_>> {
        self.callbacks.video.map(|cb| Video {
            callback: cb,
            frame_counter: &self.frame_counter,
        })
    }

    /// Returns audio callbacks. The wrapper selects the sample or batch variant
    /// automatically when you push data.
    pub fn audio(&self) -> Audio {
        Audio {
            sample: self.callbacks.audio_sample,
            batch: self.callbacks.audio_batch,
        }
    }

    /// Returns input callbacks if the frontend installed both `poll` and `state`.
    pub fn input(&self) -> Option<Input> {
        match (self.callbacks.input_poll, self.callbacks.input_state) {
            (Some(poll), Some(state)) => Some(Input { poll, state }),
            _ => None,
        }
    }
}

/// Wrapper for `retro_environment_t`.
type EnvironmentCallback = unsafe extern "C" fn(u32, *mut c_void) -> bool;

/// Handle used to invoke `retro_environment_t`.
pub struct Environment {
    callback: EnvironmentCallback,
}

impl Environment {
    /// Invokes the environment callback with the provided command and payload.
    pub fn request<T>(&self, command: u32, data: &mut T) -> bool {
        unsafe { (self.callback)(command, data as *mut T as *mut c_void) }
    }

    /// Returns the raw `retro_environment_t` pointer for FFI calls.
    ///
    /// # Safety
    /// The caller must ensure the returned function pointer is invoked with the
    /// same ABI guarantees that libretro requires.
    pub unsafe fn raw(&self) -> raw::retro_environment_t {
        Some(self.callback)
    }
}

/// Wrapper for `retro_video_refresh_t`.
type VideoCallback = unsafe extern "C" fn(*const c_void, u32, u32, usize);

/// Sends frames to the frontend via `retro_video_refresh_t`.
pub struct Video<'a> {
    callback: VideoCallback,
    frame_counter: &'a AtomicUsize,
}

impl<'a> Video<'a> {
    /// Submits a frame described by [`Frame`].
    pub fn submit(&self, frame: Frame<'_>) {
        let (ptr, width, height, pitch) = frame.into_raw();
        unsafe {
            (self.callback)(ptr, width, height, pitch);
        }
        self.frame_counter.fetch_add(1, Ordering::Relaxed);
    }
}

/// Frame submission metadata for [`Video::submit`].
pub struct Frame<'a> {
    buffer: FrameBuffer<'a>,
    width: u32,
    height: u32,
    pitch: usize,
}

impl<'a> Frame<'a> {
    /// Describes a software frame backed by CPU-accessible pixels.
    pub fn from_pixels(buffer: &'a [u8], width: u32, height: u32, pitch: usize) -> Self {
        Self {
            buffer: FrameBuffer::Pixels(buffer),
            width,
            height,
            pitch,
        }
    }

    /// Describes a GPU-managed framebuffer.
    pub fn hardware(token: *const c_void, width: u32, height: u32, pitch: usize) -> Self {
        Self {
            buffer: FrameBuffer::Hardware(token),
            width,
            height,
            pitch,
        }
    }

    /// Indicates that the previous frame should be duplicated.
    pub fn duplicate() -> Self {
        Self {
            buffer: FrameBuffer::Duplicate,
            width: 0,
            height: 0,
            pitch: 0,
        }
    }

    fn into_raw(self) -> (*const c_void, u32, u32, usize) {
        (self.buffer.as_ptr(), self.width, self.height, self.pitch)
    }
}

enum FrameBuffer<'a> {
    Pixels(&'a [u8]),
    Hardware(*const c_void),
    Duplicate,
}

impl<'a> FrameBuffer<'a> {
    fn as_ptr(&self) -> *const c_void {
        match self {
            FrameBuffer::Pixels(pixels) => pixels.as_ptr() as *const c_void,
            FrameBuffer::Hardware(ptr) => *ptr,
            FrameBuffer::Duplicate => std::ptr::null(),
        }
    }
}

/// Wrapper for libretro audio callbacks.
///
/// The wrapper automatically chooses between the single-sample and batch
/// callbacks depending on which one the frontend supports.
pub struct Audio {
    sample: raw::retro_audio_sample_t,
    batch: raw::retro_audio_sample_batch_t,
}

impl Audio {
    /// Sends a single stereo sample pair to the frontend.
    pub fn push_sample(&self, left: i16, right: i16) {
        match (self.sample, self.batch) {
            (Some(sample), _) => unsafe { sample(left, right) },
            (None, Some(batch)) => {
                let frames = [left, right];
                unsafe {
                    batch(frames.as_ptr(), 1);
                }
            }
            (None, None) => {}
        }
    }

    /// Sends a batch of stereo samples to the frontend.
    ///
    /// When the frontend only provided the single-sample callback, this method
    /// falls back to iterating through each pair.
    pub fn push_frames(&self, frames: &[[i16; 2]]) -> usize {
        if let Some(batch) = self.batch {
            unsafe { batch(frames.as_ptr() as *const i16, frames.len()) }
        } else {
            for frame in frames {
                let [left, right] = *frame;
                self.push_sample(left, right);
            }
            frames.len()
        }
    }
}

/// Helpers for polling libretro inputs.
type PollCallback = unsafe extern "C" fn();
type StateCallback = unsafe extern "C" fn(u32, u32, u32, u32) -> i16;

/// Wrapper over input callbacks provided by the frontend.
pub struct Input {
    poll: PollCallback,
    state: StateCallback,
}

impl Input {
    /// Tells the frontend to sample the current input devices.
    pub fn poll(&self) {
        unsafe { (self.poll)() };
    }

    /// Queries the state of an input element such as a joypad button.
    pub fn state(&self, port: u32, device: u32, index: u32, id: u32) -> i16 {
        unsafe { (self.state)(port, device, index, id) }
    }
}

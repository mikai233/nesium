use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Context, Result};
use cpal::{
    SampleFormat,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use ringbuf::{
    HeapRb,
    traits::{Consumer, Producer, Split},
};

/// Thin audio output wrapper that feeds interleaved stereo PCM samples from the
/// emulator into cpal's default output stream, backed by a lock-free SPSC
/// ring buffer.
pub struct NesAudioPlayer {
    /// Single producer handle used by the emulator thread to push samples.
    producer: ringbuf::HeapProd<f32>,
    /// Output sample rate for this device.
    sample_rate: u32,
    /// Underlying CPAL output stream (kept alive by this struct).
    _stream: cpal::Stream,
    /// Flag to request clearing any queued samples from the audio thread.
    clear_flag: Arc<AtomicBool>,
}

impl NesAudioPlayer {
    /// Create a new audio player on the default output device.
    ///
    /// This currently assumes the default output config uses `f32` samples.
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .context("no default output device")?;

        let supported_config = device
            .default_output_config()
            .context("no default output config")?;

        let sample_format = supported_config.sample_format();
        if sample_format != SampleFormat::F32 {
            anyhow::bail!("only f32 output format is supported, got {sample_format:?}");
        }

        let config: cpal::StreamConfig = supported_config.into();
        let sample_rate = config.sample_rate.0;
        let channels = config.channels as usize;

        // Allocate ~0.2 seconds of audio in the ring buffer.
        // We store raw samples (interleaved channels), so capacity is:
        //   sample_rate * seconds * channels
        let latency_seconds = 0.2_f32;
        let capacity = (sample_rate as f32 * latency_seconds * channels as f32).ceil() as usize;
        let capacity = capacity.max(1);

        let rb = HeapRb::<f32>::new(capacity);
        let (producer, mut consumer) = rb.split();

        let clear_flag = Arc::new(AtomicBool::new(false));
        let clear_flag_for_cb = clear_flag.clone();

        let err_fn = |err| eprintln!("Audio stream error: {err}");

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _| {
                if clear_flag_for_cb.swap(false, Ordering::SeqCst) {
                    while consumer.try_pop().is_some() {}
                }

                for frame in data.chunks_mut(channels) {
                    let left = consumer.try_pop().unwrap_or(0.0);
                    let right = consumer.try_pop().unwrap_or(left);

                    match channels {
                        0 => {}
                        1 => {
                            let mono = (left + right) * 0.5;
                            frame[0] = mono;
                        }
                        _ => {
                            frame[0] = left;
                            frame[1] = right;
                            for ch in &mut frame[2..] {
                                *ch = right;
                            }
                        }
                    }
                }
            },
            err_fn,
            None,
        )?;

        stream.play()?;

        Ok(Self {
            producer,
            sample_rate,
            _stream: stream,
            clear_flag,
        })
    }

    /// Pushes a batch of interleaved stereo samples into the output buffer.
    ///
    /// If the buffer is full, newest samples are dropped rather than blocking.
    pub fn push_samples(&mut self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }

        for chunk in samples.chunks(2) {
            let l = *chunk.first().unwrap_or(&0.0);
            let r = *chunk.get(1).unwrap_or(&l);

            let l = (l * 0.9).clamp(-1.0, 1.0);
            let r = (r * 0.9).clamp(-1.0, 1.0);

            let _ = self.producer.try_push(l);
            let _ = self.producer.try_push(r);
        }
    }

    /// Requests that any queued samples be dropped.
    pub fn clear(&self) {
        self.clear_flag.store(true, Ordering::SeqCst);
    }

    /// Returns the output sample rate.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use cpal::{
    FromSample, Sample, SampleFormat, SizedSample,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

/// CPU clock rate (NTSC).
const CPU_HZ: f64 = 1_789_773.0;

/// Simple timing helper that accumulates PPU time and emits host-rate samples via linear interpolation.
#[derive(Debug, Clone, Copy)]
pub struct AudioTiming {
    pub(crate) sample_period: f64,
    pub(crate) cpu_period: f64,
    pub(crate) accum_time: f64,
    pub(crate) last_sample: f32,
}

impl AudioTiming {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_period: 1.0 / sample_rate as f64,
            cpu_period: 1.0 / CPU_HZ,
            accum_time: 0.0,
            last_sample: 0.0,
        }
    }

    /// Step the accumulator with one CPU/APU tick worth of time and current sample; emit linearly interpolated output.
    pub fn step(&mut self, sample: f32, out: &mut Vec<f32>) {
        self.accum_time += self.cpu_period;
        while self.accum_time >= self.sample_period {
            // Linear interpolate between last_sample and current sample over the elapsed window.
            let t = (self.accum_time - self.sample_period) / self.cpu_period;
            let interp = self.last_sample * (t as f32) + sample * (1.0 - t as f32);
            out.push(interp);
            self.accum_time -= self.sample_period;
        }
        self.last_sample = sample;
    }
}

/// Thin audio output wrapper that feeds PCM samples from the emulator into
/// cpal's default output stream.
pub struct NesAudioPlayer {
    buffer: Arc<Mutex<VecDeque<f32>>>,
    sample_rate: u32,
    max_queue: usize,
    _stream: cpal::Stream,
}

impl NesAudioPlayer {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .context("no default output device")?;

        let config = device.default_output_config()?;
        let sample_rate = config.sample_rate().0;

        let buffer = Arc::new(Mutex::new(VecDeque::with_capacity(
            (sample_rate / 5) as usize,
        )));
        // Allow ~0.2s of queued audio to avoid underruns; avoid aggressive dropping that skews pitch.
        let max_queue = (sample_rate as f32 * 0.2).ceil() as usize;
        let stream = match config.sample_format() {
            SampleFormat::F32 => {
                Self::build_stream::<f32>(&device, &config.into(), buffer.clone())?
            }
            SampleFormat::I16 => {
                Self::build_stream::<i16>(&device, &config.into(), buffer.clone())?
            }
            SampleFormat::U16 => {
                Self::build_stream::<u16>(&device, &config.into(), buffer.clone())?
            }
            other => anyhow::bail!("unsupported sample format {other:?}"),
        };

        stream.play()?;

        Ok(Self {
            buffer,
            sample_rate,
            max_queue,
            _stream: stream,
        })
    }

    fn build_stream<T>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        buffer: Arc<Mutex<VecDeque<f32>>>,
    ) -> Result<cpal::Stream>
    where
        T: Sample + SizedSample + FromSample<f32>,
    {
        let channels = config.channels as usize;
        let err_fn = |err| tracing::error!("Audio stream error: {err}");
        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _| {
                for frame in data.chunks_mut(channels) {
                    let sample = {
                        let mut guard = buffer.lock().unwrap();
                        guard.pop_front().unwrap_or(0.0)
                    };
                    let converted: T = sample.to_sample::<T>();
                    frame.iter_mut().for_each(|out| *out = converted);
                }
            },
            err_fn,
            None,
        )?;
        Ok(stream)
    }

    /// Pushes a batch of mono samples into the output buffer.
    pub fn push_samples(&self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }
        if let Ok(mut guard) = self.buffer.lock() {
            for &raw in samples {
                // Center around 0, keep modest gain.
                let centered = (raw - 0.5) * 0.8;
                guard.push_back(centered.clamp(-1.0, 1.0));
            }
            if guard.len() > self.max_queue {
                let drop_count = guard.len() - self.max_queue;
                for _ in 0..drop_count {
                    guard.pop_front();
                }
            }
        }
    }

    /// Clears any queued samples (useful when resetting the emulator).
    pub fn clear(&self) {
        if let Ok(mut guard) = self.buffer.lock() {
            guard.clear();
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

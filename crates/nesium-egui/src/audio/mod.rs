use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use cpal::{
    FromSample, Sample, SampleFormat, SizedSample,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

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
                let scaled = (raw * 0.9).clamp(-1.0, 1.0);
                guard.push_back(scaled);
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

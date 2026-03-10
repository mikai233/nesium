use std::{
    ffi::c_void,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use anyhow::{Context, Result, anyhow};
use nesium_core::{
    audio::bus::AudioBusConfig,
    ppu::{
        buffer::{ColorFormat, FrameReadyCallback},
        palette::PaletteKind,
    },
    reset_kind::ResetKind,
};
use nesium_runtime::{
    AudioMode, EventTopic, Runtime, RuntimeConfig, RuntimeEventSender, RuntimeHandle, VideoConfig,
};

use crate::video::GameRenderer;

pub struct RuntimeSession {
    _runtime: Runtime,
    handle: RuntimeHandle,
    frame_handle: Arc<nesium_core::ppu::buffer::ExternalFrameHandle>,
    uploaded_frame_seq: AtomicU64,
}

impl RuntimeSession {
    pub fn new() -> Result<Self> {
        let runtime = Runtime::start(RuntimeConfig {
            video: VideoConfig {
                color_format: ColorFormat::Rgba8888,
                ..VideoConfig::default()
            },
            audio: AudioMode::Auto,
        })
        .context("failed to start nesium runtime")?;

        let handle = runtime.handle();
        let frame_handle = handle
            .frame_handle()
            .cloned()
            .ok_or_else(|| anyhow!("runtime did not expose a readable frame handle"))?;

        Ok(Self {
            _runtime: runtime,
            handle,
            frame_handle,
            uploaded_frame_seq: AtomicU64::new(u64::MAX),
        })
    }

    pub fn load_rom(&self, path: &Path) -> Result<()> {
        self.handle
            .load_rom(path.to_path_buf())
            .context("failed to load ROM")?;
        self.handle.set_paused(false);
        self.uploaded_frame_seq.store(u64::MAX, Ordering::Release);
        Ok(())
    }

    pub fn toggle_pause(&self) -> bool {
        let paused = !self.handle.paused();
        self.handle.set_paused(paused);
        paused
    }

    #[allow(dead_code)]
    pub fn reset(&self) -> Result<()> {
        self.handle
            .reset(ResetKind::Soft)
            .context("failed to reset emulator")
    }

    pub fn power_reset(&self) -> Result<()> {
        self.handle
            .reset(ResetKind::PowerOn)
            .context("failed to power reset emulator")?;
        self.handle.set_paused(false);
        self.uploaded_frame_seq.store(u64::MAX, Ordering::Release);
        Ok(())
    }

    pub fn power_off(&self) -> Result<()> {
        self.handle
            .power_off()
            .context("failed to power off emulator")?;
        self.handle.set_paused(false);
        self.uploaded_frame_seq.store(u64::MAX, Ordering::Release);
        Ok(())
    }

    pub fn set_pad_mask(&self, pad: usize, mask: u8) {
        self.handle.set_pad_mask(pad, mask);
    }

    pub fn set_turbo_mask(&self, pad: usize, mask: u8) {
        self.handle.set_turbo_mask(pad, mask);
    }

    pub fn set_integer_fps_mode(&self, enabled: bool) -> Result<()> {
        self.handle
            .set_integer_fps_target(enabled.then_some(60))
            .context("failed to change integer FPS pacing")
    }

    pub fn upload_latest_frame(&self, renderer: &mut GameRenderer) -> bool {
        let frame_seq = self.handle.frame_seq();
        if self.uploaded_frame_seq.load(Ordering::Acquire) == frame_seq {
            return false;
        }

        let plane_index = self.frame_handle.begin_front_copy();
        let frame = self.frame_handle.plane_slice(plane_index);
        let updated = renderer.upload_rgba_frame(
            frame,
            self.frame_handle.width(),
            self.frame_handle.height(),
            self.frame_handle.pitch_bytes(),
        );
        self.frame_handle.end_front_copy();

        if updated {
            self.uploaded_frame_seq.store(frame_seq, Ordering::Release);
        }

        updated
    }

    pub fn set_frame_ready_callback(
        &self,
        cb: Option<FrameReadyCallback>,
        user_data: *mut c_void,
    ) -> Result<()> {
        self.handle
            .set_frame_ready_callback(cb, user_data)
            .context("failed to configure frame ready callback")
    }

    pub fn subscribe_event(
        &self,
        topic: EventTopic,
        sender: Box<dyn RuntimeEventSender>,
    ) -> Result<()> {
        self.handle
            .subscribe_event(topic, sender)
            .context("failed to subscribe runtime event")
    }

    pub fn unsubscribe_event(&self, topic: EventTopic) -> Result<()> {
        self.handle
            .unsubscribe_event(topic)
            .context("failed to unsubscribe runtime event")
    }

    pub fn set_palette_kind(&self, kind: PaletteKind) -> Result<()> {
        self.handle
            .set_palette_kind(kind)
            .context("failed to set palette kind")
    }

    pub fn set_palette_from_pal_file(&self, path: &Path) -> Result<()> {
        self.handle
            .set_palette_from_pal_file(path)
            .context("failed to load .pal file")
    }

    pub fn set_audio_config(&self, config: AudioBusConfig) -> Result<()> {
        self.handle
            .set_audio_config(config)
            .context("failed to update audio config")
    }

    pub fn set_turbo_timing(&self, on_frames: u8, off_frames: u8) {
        self.handle.set_turbo_timing(on_frames, off_frames);
    }
}

use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use librashader::runtime::FilterChainParameters;
use nesium_core::ppu::buffer::{NearestPostProcessor, VideoPostProcessor};
use nesium_core::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};
use nesium_support::video::filters::NesNtscPostProcessor;
use nesium_support::video::filters::NesNtscTuning;
use nesium_support::video::filters::{
    HqxPostProcessor, LcdGridPostProcessor, NtscBisqwitOptions as SupportNtscBisqwitOptions,
    NtscBisqwitPostProcessor, SaiPostProcessor, SaiVariant, ScanlinePostProcessor,
    XbrzPostProcessor,
};
use nesium_support::video::hqx::HqxScale;
use nesium_support::video::ntsc::NesNtscPreset;
use nesium_support::video::ntsc::nes_ntsc_out_width;
use std::sync::{Mutex, OnceLock};

/// Single-select video filter configuration, modeled after Mesen's `VideoFilterType`.
///
/// Notes:
/// - `None` keeps the output at the canonical NES size (256×240).
/// - `PrescaleNx` outputs an integer-scaled frame (N×) in the runtime's packed framebuffer.
/// - `HqNx` outputs an integer-scaled frame (N×) using HQX (hq2x/hq3x/hq4x).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoFilter {
    None,
    Prescale2x,
    Prescale3x,
    Prescale4x,
    Hq2x,
    Hq3x,
    Hq4x,
    Sai2x,
    Super2xSai,
    SuperEagle,
    NtscComposite,
    NtscSVideo,
    NtscRgb,
    NtscMonochrome,
    LcdGrid,
    Scanlines,
    Xbrz2x,
    Xbrz3x,
    Xbrz4x,
    Xbrz5x,
    Xbrz6x,
    NtscBisqwit2x,
    NtscBisqwit4x,
    NtscBisqwit8x,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoOutputInfo {
    pub output_width: u32,
    pub output_height: u32,
}

#[frb]
pub struct ShaderParameter {
    pub name: String,
    pub description: String,
    pub initial: f32,
    pub current: f32,
    pub minimum: f32,
    pub maximum: f32,
    pub step: f32,
}

#[frb]
pub struct ShaderParameters {
    pub path: String,
    // We use a Vec here instead of a HashMap to preserve the order of parameters
    // as provided by librashader. librashader uses `halfbrown::HashMap` which
    // preserves insertion order for small sets (n < 32), which covers most
    // shader presets. Standard `std::collections::HashMap` does not guarantee order.
    pub parameters: Vec<ShaderParameter>,
}

#[frb]
pub fn shader_parameters_stream(sink: StreamSink<ShaderParameters>) -> Result<(), String> {
    crate::senders::shader::set_shader_sink(sink);

    // After registering, we should emit the current state if it exists.
    // This avoids needing a separate manual fetch on startup.
    let current = get_shader_parameters();
    crate::senders::shader::emit_shader_parameters_update(current);

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NtscOptions {
    pub hue: f64,
    pub saturation: f64,
    pub contrast: f64,
    pub brightness: f64,
    pub sharpness: f64,
    pub gamma: f64,
    pub resolution: f64,
    pub artifacts: f64,
    pub fringing: f64,
    pub bleed: f64,
    pub merge_fields: bool,
}

impl Default for NtscOptions {
    fn default() -> Self {
        Self {
            hue: 0.0,
            saturation: 0.0,
            contrast: 0.0,
            brightness: 0.0,
            sharpness: 0.0,
            gamma: 0.0,
            resolution: 0.0,
            artifacts: 0.0,
            fringing: 0.0,
            bleed: 0.0,
            merge_fields: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LcdGridOptions {
    /// Strength in `0.0..=1.0` (0 = off, 1 = strongest / default).
    pub strength: f64,
}

impl Default for LcdGridOptions {
    fn default() -> Self {
        Self { strength: 1.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScanlineOptions {
    /// Scanline intensity in `0.0..=1.0` (0 = off, 1 = strongest).
    pub intensity: f64,
}

impl Default for ScanlineOptions {
    fn default() -> Self {
        // Matches the previous hard-coded value: brightness multiplier ≈ 0.70.
        Self { intensity: 0.30 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NtscBisqwitOptions {
    pub brightness: f64,
    pub contrast: f64,
    pub hue: f64,
    pub saturation: f64,
    pub y_filter_length: f64,
    pub i_filter_length: f64,
    pub q_filter_length: f64,
}

impl Default for NtscBisqwitOptions {
    fn default() -> Self {
        Self {
            brightness: 0.0,
            contrast: 0.0,
            hue: 0.0,
            saturation: 0.0,
            y_filter_length: 0.0,
            i_filter_length: 0.5,
            q_filter_length: 0.5,
        }
    }
}

static NTSC_OPTIONS: OnceLock<Mutex<NtscOptions>> = OnceLock::new();
static LCD_GRID_OPTIONS: OnceLock<Mutex<LcdGridOptions>> = OnceLock::new();
static SCANLINE_OPTIONS: OnceLock<Mutex<ScanlineOptions>> = OnceLock::new();
static NTSC_BISQWIT_OPTIONS: OnceLock<Mutex<NtscBisqwitOptions>> = OnceLock::new();
static CURRENT_FILTER: OnceLock<Mutex<VideoFilter>> = OnceLock::new();

fn ntsc_options() -> NtscOptions {
    *NTSC_OPTIONS
        .get_or_init(|| Mutex::new(NtscOptions::default()))
        .lock()
        .unwrap()
}

fn lcd_grid_options() -> LcdGridOptions {
    *LCD_GRID_OPTIONS
        .get_or_init(|| Mutex::new(LcdGridOptions::default()))
        .lock()
        .unwrap()
}

fn scanline_options() -> ScanlineOptions {
    *SCANLINE_OPTIONS
        .get_or_init(|| Mutex::new(ScanlineOptions::default()))
        .lock()
        .unwrap()
}

fn set_current_filter(filter: VideoFilter) {
    *CURRENT_FILTER
        .get_or_init(|| Mutex::new(VideoFilter::None))
        .lock()
        .unwrap() = filter;
}

fn current_filter() -> VideoFilter {
    *CURRENT_FILTER
        .get_or_init(|| Mutex::new(VideoFilter::None))
        .lock()
        .unwrap()
}

fn ntsc_bisqwit_options() -> NtscBisqwitOptions {
    *NTSC_BISQWIT_OPTIONS
        .get_or_init(|| Mutex::new(NtscBisqwitOptions::default()))
        .lock()
        .unwrap()
}

impl VideoFilter {
    fn scale_factor(self) -> u32 {
        match self {
            VideoFilter::None => 1,
            VideoFilter::Prescale2x => 2,
            VideoFilter::Prescale3x => 3,
            VideoFilter::Prescale4x => 4,
            VideoFilter::Hq2x => 2,
            VideoFilter::Hq3x => 3,
            VideoFilter::Hq4x => 4,
            VideoFilter::Sai2x | VideoFilter::Super2xSai | VideoFilter::SuperEagle => 2,
            VideoFilter::NtscComposite
            | VideoFilter::NtscSVideo
            | VideoFilter::NtscRgb
            | VideoFilter::NtscMonochrome => 0,
            VideoFilter::LcdGrid | VideoFilter::Scanlines => 2,
            VideoFilter::Xbrz2x => 2,
            VideoFilter::Xbrz3x => 3,
            VideoFilter::Xbrz4x => 4,
            VideoFilter::Xbrz5x => 5,
            VideoFilter::Xbrz6x => 6,
            VideoFilter::NtscBisqwit2x => 2,
            VideoFilter::NtscBisqwit4x => 4,
            VideoFilter::NtscBisqwit8x => 8,
        }
    }

    fn output_size(self) -> Result<(u32, u32), String> {
        match self {
            VideoFilter::NtscComposite
            | VideoFilter::NtscSVideo
            | VideoFilter::NtscRgb
            | VideoFilter::NtscMonochrome => {
                let w = nes_ntsc_out_width(SCREEN_WIDTH)
                    .try_into()
                    .map_err(|_| "output_width overflow".to_string())?;
                let h = (SCREEN_HEIGHT as u32)
                    .checked_mul(2)
                    .ok_or_else(|| "output_height overflow".to_string())?;
                Ok((w, h))
            }
            _ => {
                let scale = self.scale_factor();
                let output_width = (SCREEN_WIDTH as u32)
                    .checked_mul(scale)
                    .ok_or_else(|| "output_width overflow".to_string())?;
                let output_height = (SCREEN_HEIGHT as u32)
                    .checked_mul(scale)
                    .ok_or_else(|| "output_height overflow".to_string())?;
                Ok((output_width, output_height))
            }
        }
    }
}

fn is_ntsc_filter(filter: VideoFilter) -> bool {
    matches!(
        filter,
        VideoFilter::NtscComposite
            | VideoFilter::NtscSVideo
            | VideoFilter::NtscRgb
            | VideoFilter::NtscMonochrome
    )
}

fn is_ntsc_bisqwit_filter(filter: VideoFilter) -> bool {
    matches!(
        filter,
        VideoFilter::NtscBisqwit2x | VideoFilter::NtscBisqwit4x | VideoFilter::NtscBisqwit8x
    )
}

fn apply_video_filter(filter: VideoFilter) -> Result<VideoOutputInfo, String> {
    let (output_width, output_height) = filter.output_size()?;

    #[cfg(target_os = "android")]
    crate::android::resize_ahb_swapchain(output_width, output_height)?;

    let processor: Box<dyn VideoPostProcessor> = match filter {
        VideoFilter::Hq2x => Box::new(HqxPostProcessor::new(HqxScale::X2)),
        VideoFilter::Hq3x => Box::new(HqxPostProcessor::new(HqxScale::X3)),
        VideoFilter::Hq4x => Box::new(HqxPostProcessor::new(HqxScale::X4)),
        VideoFilter::Sai2x => Box::new(SaiPostProcessor::new(SaiVariant::Sai2x)),
        VideoFilter::Super2xSai => Box::new(SaiPostProcessor::new(SaiVariant::Super2xSai)),
        VideoFilter::SuperEagle => Box::new(SaiPostProcessor::new(SaiVariant::SuperEagle)),
        VideoFilter::LcdGrid => {
            let o = lcd_grid_options();
            Box::new(LcdGridPostProcessor::new(o.strength))
        }
        VideoFilter::Scanlines => {
            let o = scanline_options();
            Box::new(ScanlinePostProcessor::new(2, o.intensity))
        }
        VideoFilter::Xbrz2x => Box::new(XbrzPostProcessor::new(2)),
        VideoFilter::Xbrz3x => Box::new(XbrzPostProcessor::new(3)),
        VideoFilter::Xbrz4x => Box::new(XbrzPostProcessor::new(4)),
        VideoFilter::Xbrz5x => Box::new(XbrzPostProcessor::new(5)),
        VideoFilter::Xbrz6x => Box::new(XbrzPostProcessor::new(6)),
        VideoFilter::NtscBisqwit2x | VideoFilter::NtscBisqwit4x | VideoFilter::NtscBisqwit8x => {
            let o = ntsc_bisqwit_options();
            let support_o = SupportNtscBisqwitOptions {
                brightness: o.brightness,
                contrast: o.contrast,
                hue: o.hue,
                saturation: o.saturation,
                y_filter_length: o.y_filter_length,
                i_filter_length: o.i_filter_length,
                q_filter_length: o.q_filter_length,
            };
            let scale = filter.scale_factor() as u8;
            Box::new(NtscBisqwitPostProcessor::new(scale, support_o))
        }
        VideoFilter::NtscComposite => {
            let o = ntsc_options();
            Box::new(NesNtscPostProcessor::new_with_tuning(
                NesNtscPreset::Composite,
                NesNtscTuning {
                    hue: o.hue,
                    saturation: o.saturation,
                    contrast: o.contrast,
                    brightness: o.brightness,
                    sharpness: o.sharpness,
                    gamma: o.gamma,
                    resolution: o.resolution,
                    artifacts: o.artifacts,
                    fringing: o.fringing,
                    bleed: o.bleed,
                    merge_fields: o.merge_fields,
                },
            ))
        }
        VideoFilter::NtscSVideo => {
            let o = ntsc_options();
            Box::new(NesNtscPostProcessor::new_with_tuning(
                NesNtscPreset::SVideo,
                NesNtscTuning {
                    hue: o.hue,
                    saturation: o.saturation,
                    contrast: o.contrast,
                    brightness: o.brightness,
                    sharpness: o.sharpness,
                    gamma: o.gamma,
                    resolution: o.resolution,
                    artifacts: o.artifacts,
                    fringing: o.fringing,
                    bleed: o.bleed,
                    merge_fields: o.merge_fields,
                },
            ))
        }
        VideoFilter::NtscRgb => {
            let o = ntsc_options();
            Box::new(NesNtscPostProcessor::new_with_tuning(
                NesNtscPreset::Rgb,
                NesNtscTuning {
                    hue: o.hue,
                    saturation: o.saturation,
                    contrast: o.contrast,
                    brightness: o.brightness,
                    sharpness: o.sharpness,
                    gamma: o.gamma,
                    resolution: o.resolution,
                    artifacts: o.artifacts,
                    fringing: o.fringing,
                    bleed: o.bleed,
                    merge_fields: o.merge_fields,
                },
            ))
        }
        VideoFilter::NtscMonochrome => {
            let o = ntsc_options();
            Box::new(NesNtscPostProcessor::new_with_tuning(
                NesNtscPreset::Monochrome,
                NesNtscTuning {
                    hue: o.hue,
                    saturation: o.saturation,
                    contrast: o.contrast,
                    brightness: o.brightness,
                    sharpness: o.sharpness,
                    gamma: o.gamma,
                    resolution: o.resolution,
                    artifacts: o.artifacts,
                    fringing: o.fringing,
                    bleed: o.bleed,
                    merge_fields: o.merge_fields,
                },
            ))
        }
        _ => Box::new(NearestPostProcessor::default()),
    };

    crate::runtime_handle()
        .set_video_pipeline(output_width, output_height, processor)
        .map_err(|e| e.to_string())?;

    Ok(VideoOutputInfo {
        output_width,
        output_height,
    })
}

#[frb]
pub fn set_video_filter(filter: VideoFilter) -> Result<VideoOutputInfo, String> {
    set_current_filter(filter);
    apply_video_filter(filter)
}

#[frb]
pub fn set_ntsc_options(options: NtscOptions) -> Result<(), String> {
    *NTSC_OPTIONS
        .get_or_init(|| Mutex::new(NtscOptions::default()))
        .lock()
        .unwrap() = options;

    let filter = current_filter();
    if is_ntsc_filter(filter) {
        let _ = apply_video_filter(filter)?;
    }

    Ok(())
}

#[frb]
pub fn set_lcd_grid_options(options: LcdGridOptions) -> Result<(), String> {
    let options = LcdGridOptions {
        strength: options.strength.clamp(0.0, 1.0),
    };

    *LCD_GRID_OPTIONS
        .get_or_init(|| Mutex::new(LcdGridOptions::default()))
        .lock()
        .unwrap() = options;

    let filter = current_filter();
    if filter == VideoFilter::LcdGrid {
        let _ = apply_video_filter(filter)?;
    }

    Ok(())
}

#[frb]
pub fn set_scanline_options(options: ScanlineOptions) -> Result<(), String> {
    let options = ScanlineOptions {
        intensity: options.intensity.clamp(0.0, 1.0),
    };

    *SCANLINE_OPTIONS
        .get_or_init(|| Mutex::new(ScanlineOptions::default()))
        .lock()
        .unwrap() = options;

    let filter = current_filter();
    if filter == VideoFilter::Scanlines {
        let _ = apply_video_filter(filter)?;
    }

    Ok(())
}

#[frb]
pub fn set_ntsc_bisqwit_options(options: NtscBisqwitOptions) -> Result<(), String> {
    let options = NtscBisqwitOptions {
        brightness: options.brightness.clamp(-1.0, 1.0),
        contrast: options.contrast.clamp(-1.0, 1.0),
        hue: options.hue.clamp(-1.0, 1.0),
        saturation: options.saturation.clamp(-1.0, 1.0),
        y_filter_length: options.y_filter_length.clamp(-0.46, 4.0),
        i_filter_length: options.i_filter_length.clamp(0.0, 4.0),
        q_filter_length: options.q_filter_length.clamp(0.0, 4.0),
    };

    *NTSC_BISQWIT_OPTIONS
        .get_or_init(|| Mutex::new(NtscBisqwitOptions::default()))
        .lock()
        .unwrap() = options;

    let filter = current_filter();
    if is_ntsc_bisqwit_filter(filter) {
        let _ = apply_video_filter(filter)?;
    }

    Ok(())
}

#[frb]
pub fn set_shader_enabled(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        crate::android::session::android_set_shader_enabled(enabled);
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        crate::windows::windows_set_shader_enabled(enabled);
        Ok(())
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        crate::apple::apple_set_shader_enabled(enabled);
        Ok(())
    }

    #[cfg(not(any(
        target_os = "android",
        target_os = "windows",
        target_os = "macos",
        target_os = "ios"
    )))]
    {
        let _ = enabled;
        Err("Librashader is only supported on Android, Windows, macOS and iOS for now.".to_string())
    }
}

#[frb]
pub fn set_shader_preset_path(path: Option<String>) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        let path = path.and_then(|p| {
            let trimmed = p.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
        crate::android::session::android_set_shader_preset_path(path);
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        let path = path.and_then(|p| {
            let trimmed = p.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
        crate::windows::windows_set_shader_preset_path(path);
        Ok(())
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        let path = path.and_then(|p| {
            let trimmed = p.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
        crate::apple::apple_set_shader_preset_path(path);
        Ok(())
    }

    #[cfg(not(any(
        target_os = "android",
        target_os = "windows",
        target_os = "macos",
        target_os = "ios"
    )))]
    {
        let _ = path;
        Err("Librashader is only supported on Android, Windows, macOS and iOS for now.".to_string())
    }
}

pub fn get_shader_parameters() -> ShaderParameters {
    let mut current_path = String::new();
    let mut parameters = Vec::new();

    #[cfg(target_os = "android")]
    {
        use librashader::runtime::gl::FilterChain as LibrashaderFilterChain;
        let session_guard = crate::android::session::ANDROID_SHADER_SESSION.load();
        if let Some(session) = session_guard.as_ref() {
            current_path = session.path.clone();
            let chain_guard = session.chain.lock();
            for meta in session.parameters.iter() {
                let name = &meta.id;
                parameters.push(ShaderParameter {
                    name: name.to_string(),
                    description: meta.description.clone(),
                    initial: meta.initial,
                    current: chain_guard
                        .as_ref()
                        .and_then(|c: &LibrashaderFilterChain| c.parameters().parameter_value(name))
                        .unwrap_or(meta.initial),
                    minimum: meta.minimum,
                    maximum: meta.maximum,
                    step: meta.step,
                });
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let session = crate::windows::SHADER_SESSION.load();
        if let Some(session) = session.as_ref() {
            current_path = session.path.clone();
            for meta in session.parameters.iter() {
                let name = &meta.id;
                let current = session
                    .chain
                    .lock()
                    .as_ref()
                    .and_then(|c| c.parameters().parameter_value(name))
                    .unwrap_or(meta.initial);

                parameters.push(ShaderParameter {
                    name: name.to_string(),
                    description: meta.description.clone(),
                    initial: meta.initial,
                    current,
                    minimum: meta.minimum,
                    maximum: meta.maximum,
                    step: meta.step,
                });
            }
        }
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        let session = crate::apple::session::SHADER_SESSION.load();
        if let Some(session) = session.as_ref() {
            current_path = session.path.clone();
            let chain_guard = session.chain.lock();
            for meta in session.parameters.iter() {
                let name = &meta.id;
                let current = chain_guard
                    .as_ref()
                    .and_then(|c| c.parameters().parameter_value(name))
                    .unwrap_or(meta.initial);

                parameters.push(ShaderParameter {
                    name: name.to_string(),
                    description: meta.description.clone(),
                    initial: meta.initial,
                    current,
                    minimum: meta.minimum,
                    maximum: meta.maximum,
                    step: meta.step,
                });
            }
        }
    }

    tracing::info!(
        "get_shader_parameters: path={}, count={}",
        current_path,
        parameters.len()
    );

    ShaderParameters {
        path: current_path,
        parameters,
    }
}

#[frb]
pub fn set_shader_parameter(name: String, value: f32) {
    #[cfg(target_os = "android")]
    crate::android::session::ANDROID_SHADER_SESSION
        .load()
        .as_ref()
        .map(|s| {
            s.chain.lock().as_mut().map(|chain| {
                chain.parameters().set_parameter_value(&name, value);
            });
        });

    #[cfg(target_os = "windows")]
    crate::windows::SHADER_SESSION
        .load()
        .as_ref()
        .map(|session| {
            session.chain.lock().as_mut().map(|chain| {
                chain.parameters().set_parameter_value(&name, value);
            });
        });

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    crate::apple::session::SHADER_SESSION
        .load()
        .as_ref()
        .map(|session| {
            session.chain.lock().as_mut().map(|chain| {
                chain.parameters().set_parameter_value(&name, value);
            });
        });
}

use flutter_rust_bridge::frb;
use nesium_core::ppu::buffer::{NearestPostProcessor, VideoPostProcessor};
use nesium_core::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};
use nesium_support::video::filters::HqxPostProcessor;
use nesium_support::video::filters::NesNtscPostProcessor;
use nesium_support::video::filters::NesNtscTuning;
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
    NtscComposite,
    NtscSVideo,
    NtscRgb,
    NtscMonochrome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoOutputInfo {
    pub output_width: u32,
    pub output_height: u32,
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

static NTSC_OPTIONS: OnceLock<Mutex<NtscOptions>> = OnceLock::new();
static CURRENT_FILTER: OnceLock<Mutex<VideoFilter>> = OnceLock::new();

fn ntsc_options() -> NtscOptions {
    *NTSC_OPTIONS
        .get_or_init(|| Mutex::new(NtscOptions::default()))
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
            VideoFilter::NtscComposite
            | VideoFilter::NtscSVideo
            | VideoFilter::NtscRgb
            | VideoFilter::NtscMonochrome => 0,
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

fn apply_video_filter(filter: VideoFilter) -> Result<VideoOutputInfo, String> {
    let (output_width, output_height) = filter.output_size()?;

    let processor: Box<dyn VideoPostProcessor> = match filter {
        VideoFilter::Hq2x => Box::new(HqxPostProcessor::new(HqxScale::X2)),
        VideoFilter::Hq3x => Box::new(HqxPostProcessor::new(HqxScale::X3)),
        VideoFilter::Hq4x => Box::new(HqxPostProcessor::new(HqxScale::X4)),
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

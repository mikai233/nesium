use librashader::presets::ShaderFeatures as LibrashaderShaderFeatures;
use librashader::runtime::Viewport as LibrashaderViewport;
use librashader::runtime::gl::FilterChain as LibrashaderFilterChain;
use librashader::runtime::gl::FilterChainOptions as LibrashaderFilterChainOptions;
use librashader::runtime::gl::FrameOptions as LibrashaderFrameOptions;
use librashader::runtime::gl::GLImage as LibrashaderGlImage;
use std::sync::Arc;

use librashader::preprocess::ShaderParameter;
use librashader::presets::context::VideoDriver;
use librashader::presets::{ShaderPreset, get_parameter_meta};

pub fn parse_preset(
    path: &str,
    features: LibrashaderShaderFeatures,
) -> Result<(ShaderPreset, Vec<ShaderParameter>), String> {
    let preset = ShaderPreset::try_parse_with_driver_context(path, features, VideoDriver::GlCore)
        .map_err(|e| format!("{:?}", e))?;

    let mut parameters = Vec::new();
    if let Ok(meta) = get_parameter_meta(&preset) {
        for p in meta {
            parameters.push(p.clone());
        }
    }

    Ok((preset, parameters))
}

pub fn load_from_parsed_preset(
    glow_ctx: &Arc<glow::Context>,
    preset: ShaderPreset,
    options: &LibrashaderFilterChainOptions,
) -> Result<LibrashaderFilterChain, String> {
    let chain = unsafe {
        LibrashaderFilterChain::load_from_preset(preset, Arc::clone(glow_ctx), Some(options))
    }
    .map_err(|e| format!("{:?}", e))?;

    Ok(chain)
}

pub fn reload_shader_chain(
    glow_ctx: &Arc<glow::Context>,
    path: &str,
    features: LibrashaderShaderFeatures,
    options: &LibrashaderFilterChainOptions,
) -> Result<(LibrashaderFilterChain, Vec<ShaderParameter>), String> {
    let (preset, parameters) = parse_preset(path, features)?;
    let chain = load_from_parsed_preset(glow_ctx, preset, options)?;
    Ok((chain, parameters))
}

pub fn render_shader_frame(
    chain: &mut LibrashaderFilterChain,
    input: &LibrashaderGlImage,
    viewport: &LibrashaderViewport<&LibrashaderGlImage>,
    frame_count: usize,
    options: &LibrashaderFrameOptions,
) -> Result<(), String> {
    unsafe {
        chain
            .frame(input, viewport, frame_count, Some(options))
            .map_err(|e| format!("{:?}", e))
    }
}

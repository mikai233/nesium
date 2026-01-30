use crate::api::video::ShaderParameter;
use librashader::presets::{ShaderFeatures, ShaderPreset, get_parameter_meta};
use tokio::sync::oneshot;

/// Shared helper to map librashader parameter metadata to API types.
pub fn map_parameters(preset: &ShaderPreset) -> Result<Vec<ShaderParameter>, String> {
    let meta = get_parameter_meta(preset).map_err(|e| format!("{:?}", e))?;
    Ok(meta
        .map(|meta| ShaderParameter {
            name: meta.id.to_string(),
            description: meta.description.clone(),
            initial: meta.initial,
            current: meta.initial,
            minimum: meta.minimum,
            maximum: meta.maximum,
            step: meta.step,
        })
        .collect())
}

/// Shared helper to pre-parse a shader preset in a background thread.
/// Returns a receiver for the (Preset, Parameters) result.
pub fn preparse_preset(
    path: String,
) -> oneshot::Receiver<Result<(ShaderPreset, Vec<ShaderParameter>), String>> {
    let (tx, rx) = oneshot::channel();
    let features = ShaderFeatures::ORIGINAL_ASPECT_UNIFORMS | ShaderFeatures::FRAMETIME_UNIFORMS;

    std::thread::spawn(move || {
        let res = (|| {
            let preset =
                ShaderPreset::try_parse(&path, features).map_err(|e| format!("{:?}", e))?;
            let api_parameters = map_parameters(&preset)?;
            Ok((preset, api_parameters))
        })();
        let _ = tx.send(res);
    });

    rx
}

/// Shared helper to calculate the effective path for a shader.
pub fn get_effective_path(
    enabled: bool,
    preset_path: Option<String>,
    passthrough_path: String,
) -> String {
    if enabled && preset_path.is_some() {
        preset_path.unwrap()
    } else {
        passthrough_path
    }
}

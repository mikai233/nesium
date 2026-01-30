use crate::api::video::ShaderParameters;
use arc_swap::ArcSwapOption;
use librashader::presets::ShaderPreset;
use librashader::runtime::gl::FilterChain as LibrashaderFilterChain;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

use super::renderer::rust_renderer_wake;

#[derive(Debug, Clone)]
pub struct AndroidShaderConfig {
    pub enabled: bool,
    pub preset_path: Option<String>,
    pub generation: u64,
    pub preparsed_preset: Option<ShaderPreset>,
    pub parameters: HashMap<String, f32>,
}

impl Default for AndroidShaderConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            preset_path: None,
            generation: 1,
            preparsed_preset: None,
            parameters: HashMap::new(),
        }
    }
}

pub struct ShaderSession {
    pub chain: Mutex<Option<LibrashaderFilterChain>>,
}

pub static ANDROID_SHADER_CONFIG: ArcSwapOption<AndroidShaderConfig> = ArcSwapOption::const_empty();
pub static ANDROID_SHADER_SESSION: ArcSwapOption<ShaderSession> = ArcSwapOption::const_empty();

pub fn android_shader_snapshot() -> AndroidShaderConfig {
    let guard = ANDROID_SHADER_CONFIG.load();
    if let Some(arc) = &*guard {
        (**arc).clone()
    } else {
        AndroidShaderConfig::default()
    }
}

pub async fn android_set_shader_preset_path(
    path: Option<String>,
) -> Result<ShaderParameters, String> {
    android_set_shader_config(true, path).await
}

pub async fn android_set_shader_config(
    enabled: bool,
    path: Option<String>,
) -> Result<ShaderParameters, String> {
    let mut new_gen = 0;
    let mut changed = false;

    ANDROID_SHADER_CONFIG.rcu(|old| {
        let mut new = old
            .as_ref()
            .map(|a| (**a).clone())
            .unwrap_or(AndroidShaderConfig::default());

        let target_path = path.clone().or_else(|| new.preset_path.clone());

        if new.enabled == enabled && new.preset_path == target_path {
            changed = false;
            old.clone()
        } else {
            changed = true;
            new.enabled = enabled;
            new.preset_path = target_path;
            new.generation = new.generation.wrapping_add(1);
            new.preparsed_preset = None; // Avoid race condition with old preset
            new.parameters.clear(); // Path changed, clear overrides
            new_gen = new.generation;
            Some(Arc::new(new))
        }
    });

    let passthrough_path = crate::android::passthrough::get_passthrough_preset()
        .to_string_lossy()
        .to_string();

    if !changed {
        let config = android_shader_snapshot();

        let api_parameters = config
            .preparsed_preset
            .as_ref()
            .map(|p| crate::shader_utils::map_parameters(p))
            .transpose()?
            .unwrap_or_default();

        let effective_path = crate::shader_utils::get_effective_path(
            config.enabled,
            config.preset_path.clone(),
            passthrough_path,
        );

        return Ok(ShaderParameters {
            path: effective_path,
            parameters: api_parameters,
        });
    }

    // Unified Pull Model logic:
    let config_snapshot = android_shader_snapshot();
    let effective_path = crate::shader_utils::get_effective_path(
        config_snapshot.enabled,
        config_snapshot.preset_path.clone(),
        passthrough_path,
    );

    let rx = crate::shader_utils::preparse_preset(effective_path.clone());
    let (preset, api_parameters) = rx
        .await
        .map_err(|e| format!("Join error: {:?}", e))?
        .map_err(|e| format!("Parse error: {:?}", e))?;

    // Update config with preparsed preset
    ANDROID_SHADER_CONFIG.rcu(|old| {
        let mut new = old.as_ref()?.as_ref().clone();
        if new.generation == new_gen {
            new.preparsed_preset = Some(preset.clone());
            Some(Arc::new(new))
        } else {
            None
        }
    });

    // Wake the renderer to pick up the change
    rust_renderer_wake();

    Ok(ShaderParameters {
        path: effective_path,
        parameters: api_parameters,
    })
}

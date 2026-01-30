use crate::api::video::ShaderParameters;
use arc_swap::ArcSwapOption;
use librashader::presets::ShaderPreset;
use librashader::runtime::gl::FilterChain as LibrashaderFilterChain;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::oneshot;

use super::renderer::rust_renderer_wake;

#[derive(Debug, Clone)]
pub struct AndroidShaderConfig {
    pub enabled: bool,
    pub preset_path: Option<String>,
    pub generation: u64,
}

pub struct ShaderSession {
    pub chain: Mutex<Option<LibrashaderFilterChain>>,
    pub parameters: Vec<librashader::preprocess::ShaderParameter>,
    pub path: String,
}

pub struct PendingShaderData {
    pub preset: ShaderPreset,
    pub parameters: Vec<librashader::preprocess::ShaderParameter>,
    pub generation: u64,
}

pub static ANDROID_SHADER_CONFIG: ArcSwapOption<AndroidShaderConfig> = ArcSwapOption::const_empty();
pub static ANDROID_SHADER_SESSION: ArcSwapOption<ShaderSession> = ArcSwapOption::const_empty();
pub static PENDING_SHADER_DATA: ArcSwapOption<PendingShaderData> = ArcSwapOption::const_empty();

pub fn android_shader_snapshot() -> AndroidShaderConfig {
    let guard = ANDROID_SHADER_CONFIG.load();
    if let Some(arc) = &*guard {
        (**arc).clone()
    } else {
        AndroidShaderConfig {
            enabled: false,
            preset_path: None,
            generation: 1,
        }
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
    let (tx, rx) = tokio::sync::oneshot::channel();

    let mut new_gen = 0;
    let mut changed = false;

    ANDROID_SHADER_CONFIG.rcu(|old| {
        let mut new = old
            .as_ref()
            .map(|a| (**a).clone())
            .unwrap_or(AndroidShaderConfig {
                enabled: false,
                preset_path: None,
                generation: 1,
            });

        let mut target_path = path.clone();
        if target_path.is_none() && path.is_none() {
            target_path = new.preset_path.clone();
        }

        if new.enabled == enabled && new.preset_path == target_path {
            changed = false;
            old.clone()
        } else {
            changed = true;
            new.enabled = enabled;
            new.preset_path = target_path;
            new.generation = new.generation.wrapping_add(1);
            new_gen = new.generation;
            Some(Arc::new(new))
        }
    });

    if !changed {
        if let Some(session) = &*ANDROID_SHADER_SESSION.load() {
            let api_parameters = session
                .parameters
                .iter()
                .map(|meta| crate::api::video::ShaderParameter {
                    name: meta.id.to_string(),
                    description: meta.description.clone(),
                    initial: meta.initial,
                    current: meta.initial,
                    minimum: meta.minimum,
                    maximum: meta.maximum,
                    step: meta.step,
                })
                .collect();

            return Ok(ShaderParameters {
                path: session.path.clone(),
                parameters: api_parameters,
            });
        }
        return Ok(ShaderParameters {
            path: String::new(),
            parameters: Vec::new(),
        });
    }

    RELOAD_CHANNELS.lock().push_back(tx);

    let config = android_shader_snapshot();
    match config.preset_path {
        Some(p) if config.enabled => {
            std::thread::spawn(move || {
                tracing::info!(
                    "Reloading Android GLES shader chain (async, path={}, generation={})",
                    p,
                    new_gen
                );
                let features = librashader::presets::ShaderFeatures::ORIGINAL_ASPECT_UNIFORMS
                    | librashader::presets::ShaderFeatures::FRAMETIME_UNIFORMS;

                match super::chain::parse_preset(&p, features) {
                    Ok((preset, parameters)) => {
                        PENDING_SHADER_DATA.store(Some(Arc::new(PendingShaderData {
                            preset,
                            parameters,
                            generation: new_gen,
                        })));
                        rust_renderer_wake();
                    }
                    Err(e) => {
                        tracing::error!("Background shader parsing failed: {}", e);
                        rust_renderer_wake();
                    }
                }
            });
        }
        _ => {
            rust_renderer_wake();
        }
    }

    rx.await
        .map_err(|e| format!("Reload task cancelled: {:?}", e))?
}

pub static RELOAD_CHANNELS: Mutex<VecDeque<oneshot::Sender<Result<ShaderParameters, String>>>> =
    Mutex::new(VecDeque::new());

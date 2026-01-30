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

pub fn android_set_shader_enabled(enabled: bool) {
    ANDROID_SHADER_CONFIG.rcu(|old| {
        let mut new = old
            .as_ref()
            .map(|a| (**a).clone())
            .unwrap_or(AndroidShaderConfig {
                enabled: false,
                preset_path: None,
                generation: 1,
            });

        if new.enabled == enabled {
            old.clone()
        } else {
            new.enabled = enabled;
            new.generation = new.generation.wrapping_add(1);
            Some(Arc::new(new))
        }
    });

    // Wake renderer so it reloads promptly.
    rust_renderer_wake();
}

pub static RELOAD_CHANNELS: Mutex<VecDeque<oneshot::Sender<Result<ShaderParameters, String>>>> =
    Mutex::new(VecDeque::new());

pub async fn android_set_shader_preset_path(
    path: Option<String>,
) -> Result<ShaderParameters, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    RELOAD_CHANNELS.lock().push_back(tx);

    let mut new_gen = 0;
    ANDROID_SHADER_CONFIG.rcu(|old| {
        let mut new = old
            .as_ref()
            .map(|a| (**a).clone())
            .unwrap_or(AndroidShaderConfig {
                enabled: false,
                preset_path: None,
                generation: 1,
            });

        new.preset_path = path.clone();
        new.generation = new.generation.wrapping_add(1);
        new_gen = new.generation;
        Some(Arc::new(new))
    });

    match path {
        Some(p) => {
            // Background parsing to avoid blocking renderer thread with IO/Preprocessing
            std::thread::spawn(move || {
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
                        // If parsing fails, we still wake the renderer so it can fulfill the channel with the error
                        tracing::error!("Background shader parsing failed: {}", e);
                        rust_renderer_wake();
                    }
                }
            });
        }
        None => {
            rust_renderer_wake();
        }
    }

    rx.await
        .map_err(|e| format!("Reload task cancelled: {:?}", e))?
}

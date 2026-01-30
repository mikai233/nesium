use arc_swap::ArcSwapOption;
use librashader::preprocess::ShaderParameter;
use librashader::runtime::gl::FilterChain as LibrashaderFilterChain;
use parking_lot::Mutex;
use std::sync::Arc;

use super::renderer::rust_renderer_wake;

#[derive(Debug, Clone)]
pub struct AndroidShaderConfig {
    pub enabled: bool,
    pub preset_path: Option<String>,
    pub generation: u64,
}

pub struct ShaderSession {
    pub chain: Mutex<Option<LibrashaderFilterChain>>,
    pub parameters: Vec<ShaderParameter>,
    pub path: String,
}

pub static ANDROID_SHADER_CONFIG: ArcSwapOption<AndroidShaderConfig> = ArcSwapOption::const_empty();
pub static ANDROID_SHADER_SESSION: ArcSwapOption<ShaderSession> = ArcSwapOption::const_empty();

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

pub fn android_set_shader_preset_path(path: Option<String>) {
    ANDROID_SHADER_CONFIG.rcu(|old| {
        let mut new = old
            .as_ref()
            .map(|a| (**a).clone())
            .unwrap_or(AndroidShaderConfig {
                enabled: false,
                preset_path: None,
                generation: 1,
            });

        if new.preset_path == path {
            old.clone()
        } else {
            new.preset_path = path.clone();
            new.generation = new.generation.wrapping_add(1);
            Some(Arc::new(new))
        }
    });

    rust_renderer_wake();
}

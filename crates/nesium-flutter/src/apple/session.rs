use crate::api::video::ShaderParameters;
use arc_swap::ArcSwapOption;
use librashader::presets::ShaderPreset;
use librashader::runtime::mtl::FilterChain as LibrashaderFilterChain;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Tracks the generation currently being loaded or already active in renderer.
pub static ACTIVE_GENERATION: AtomicU64 = AtomicU64::new(0);
pub static ACTIVE_DEVICE_ADDR: AtomicUsize = AtomicUsize::new(0);
pub static LAST_DEVICE_ADDR: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone)]
pub struct AppleShaderConfig {
    pub enabled: bool,
    pub preset_path: Option<String>,
    pub generation: u64,
    pub preparsed_preset: Option<ShaderPreset>,
    pub parameters: HashMap<String, f32>,
}

impl Default for AppleShaderConfig {
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

pub static APPLE_SHADER_CONFIG: ArcSwapOption<AppleShaderConfig> = ArcSwapOption::const_empty();

pub struct ShaderSession {
    pub(crate) chain: Mutex<Option<LibrashaderFilterChain>>,
    pub(crate) generation: u64,
}

// SAFETY:
// `ShaderSession` contains `LibrashaderFilterChain` which wraps Metal objects.
// Metal objects (MTLDevice, etc.) are intrinsically thread-safe.
// We are storing this in a global ArcSwap, so we specifically need `Send`.
// `Sync` is implemented for completeness.
unsafe impl Send for ShaderSession {}
unsafe impl Sync for ShaderSession {}

pub static SHADER_SESSION: ArcSwapOption<ShaderSession> = ArcSwapOption::const_empty();
pub static FRAME_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn apple_shader_snapshot() -> AppleShaderConfig {
    let guard = APPLE_SHADER_CONFIG.load();
    if let Some(arc) = &*guard {
        (**arc).clone()
    } else {
        AppleShaderConfig::default()
    }
}

pub async fn apple_set_shader_preset_path(
    path: Option<String>,
) -> Result<ShaderParameters, String> {
    apple_set_shader_config(true, path).await
}

/// Attempts to trigger a reload if the desired configuration or device differs from active.
pub fn try_trigger_reload(device_ptr: *mut c_void, command_queue_ptr: *mut c_void) -> bool {
    let cfg = apple_shader_snapshot();
    let active_gen = ACTIVE_GENERATION.load(Ordering::Acquire);
    let active_device = ACTIVE_DEVICE_ADDR.load(Ordering::Acquire);

    let needs_reload_gen = active_gen != cfg.generation;
    let needs_reload_device = active_device != device_ptr as usize;

    if needs_reload_gen || needs_reload_device {
        if ACTIVE_GENERATION
            .compare_exchange(
                active_gen,
                cfg.generation,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .is_ok()
        {
            ACTIVE_DEVICE_ADDR.store(device_ptr as usize, Ordering::Release);

            let effective_path = if cfg.enabled && cfg.preset_path.is_some() {
                cfg.preset_path.clone().unwrap()
            } else {
                crate::apple::passthrough::get_passthrough_preset()
                    .to_string_lossy()
                    .to_string()
            };

            super::chain::reload_shader_chain(
                effective_path,
                device_ptr,
                command_queue_ptr,
                cfg.generation,
                cfg.preparsed_preset.clone(),
                cfg.parameters.clone(),
            );
            return true;
        }
    }
    false
}

pub async fn apple_set_shader_config(
    enabled: bool,
    path: Option<String>,
) -> Result<ShaderParameters, String> {
    let mut new_gen = 0;
    let mut changed = false;

    APPLE_SHADER_CONFIG.rcu(|old| {
        let mut new = old
            .as_ref()
            .map(|a| (**a).clone())
            .unwrap_or(AppleShaderConfig::default());

        let target_path = path.clone().or_else(|| new.preset_path.clone());

        if new.enabled == enabled && new.preset_path == target_path {
            changed = false;
            old.clone()
        } else {
            changed = true;
            let path_changed = new.preset_path != target_path;

            new.enabled = enabled;
            new.preset_path = target_path;
            new.generation = new.generation.wrapping_add(1);

            // ALWAYS clear preparsed_preset when enabled or path changes,
            // as the effective_path (and thus the preset meta) will change.
            new.preparsed_preset = None;

            if path_changed {
                new.parameters.clear();
            }

            new_gen = new.generation;
            Some(Arc::new(new))
        }
    });

    let passthrough_path = crate::apple::passthrough::get_passthrough_preset()
        .to_string_lossy()
        .to_string();

    if !changed {
        let config = APPLE_SHADER_CONFIG.load();
        let config = config.as_ref().ok_or("Config is missing")?;

        let api_parameters = config
            .preparsed_preset
            .as_ref()
            .map(|p| crate::shader_utils::map_parameters(p, &config.parameters))
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

    let config_snapshot = apple_shader_snapshot();
    let effective_path = crate::shader_utils::get_effective_path(
        config_snapshot.enabled,
        config_snapshot.preset_path.clone(),
        passthrough_path,
    );

    let rx = crate::shader_utils::preparse_preset(
        effective_path.clone(),
        config_snapshot.parameters.clone(),
    );
    let (preset, api_parameters) = rx
        .await
        .map_err(|e| format!("Join error: {:?}", e))?
        .map_err(|e| format!("Parse error: {:?}", e))?;

    // Update the config with the preparsed preset
    APPLE_SHADER_CONFIG.rcu(|old| {
        let mut new = old.as_ref()?.as_ref().clone();
        if new.generation == new_gen {
            new.preparsed_preset = Some(preset.clone());
            Some(Arc::new(new))
        } else {
            None
        }
    });

    let device_addr = LAST_DEVICE_ADDR.load(Ordering::Acquire);
    if device_addr != 0 {
        // For Metal, reloading requires both device and command queue which are not immediately available here.
        // It's safe to skip as the renderer's next frame will trigger the reload.
    } else {
        SHADER_SESSION.rcu(move |old| {
            if let Some(curr) = old {
                if curr.generation >= new_gen {
                    return None;
                }
            }
            Some(Arc::new(ShaderSession {
                chain: Mutex::new(None),
                generation: new_gen,
            }))
        });
    }

    Ok(ShaderParameters {
        path: effective_path,
        parameters: api_parameters,
    })
}

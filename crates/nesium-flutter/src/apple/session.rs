use crate::api::video::ShaderParameters;
use arc_swap::ArcSwapOption;
use librashader::preprocess::ShaderParameter;
use librashader::runtime::mtl::FilterChain as LibrashaderFilterChain;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize};
use tokio::sync::oneshot;

pub static LOADING_GENERATION: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
pub struct AppleShaderConfig {
    pub enabled: bool,
    pub preset_path: Option<String>,
    pub generation: u64,
}

pub static APPLE_SHADER_CONFIG: ArcSwapOption<AppleShaderConfig> = ArcSwapOption::const_empty();

pub struct ShaderSession {
    pub(crate) chain: Mutex<Option<LibrashaderFilterChain>>,
    pub(crate) generation: u64,
    pub(crate) device_addr: usize,
    pub(crate) parameters: Vec<ShaderParameter>,
    pub(crate) path: String,
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
        AppleShaderConfig {
            enabled: false,
            preset_path: None,
            generation: 1,
        }
    }
}

pub async fn apple_set_shader_preset_path(
    path: Option<String>,
) -> Result<ShaderParameters, String> {
    apple_set_shader_config(true, path).await
}

pub async fn apple_set_shader_config(
    enabled: bool,
    path: Option<String>,
) -> Result<ShaderParameters, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    let mut changed = false;

    APPLE_SHADER_CONFIG.rcu(|old| {
        let mut new = old
            .as_ref()
            .map(|a| (**a).clone())
            .unwrap_or(AppleShaderConfig {
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

            Some(Arc::new(new))
        }
    });

    if !changed {
        if let Some(session) = &*SHADER_SESSION.load() {
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

    // For Apple, we rely on the renderer to trigger the initial reload
    // because we need the device/queue pointers which are passed by the shell
    // only during the rendering callback.
    rx.await
        .map_err(|e| format!("Reload task cancelled: {:?}", e))?
}

pub static RELOAD_CHANNELS: Mutex<VecDeque<oneshot::Sender<Result<ShaderParameters, String>>>> =
    Mutex::new(VecDeque::new());

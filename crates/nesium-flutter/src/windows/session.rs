use crate::api::video::ShaderParameters;
use arc_swap::ArcSwapOption;
use librashader::preprocess::ShaderParameter;
use librashader::runtime::d3d11::FilterChain as LibrashaderFilterChain;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tokio::sync::oneshot;

pub static LOADING_GENERATION: AtomicU64 = AtomicU64::new(0);
pub static LAST_DEVICE_ADDR: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone)]
pub struct WindowsShaderConfig {
    pub enabled: bool,
    pub preset_path: Option<String>,
    pub generation: u64,
}

pub static WINDOWS_SHADER_CONFIG: ArcSwapOption<WindowsShaderConfig> = ArcSwapOption::const_empty();

pub struct ShaderSession {
    // Chain needs to be mutable for frame() calls, but ShaderSession is held in ArcSwap (Arc)
    pub(crate) chain: Mutex<Option<LibrashaderFilterChain>>,
    pub(crate) generation: u64,
    // Store device address to detect if the underlying D3D11 device has changed
    // (e.g. backend switch or recreation).
    pub(crate) device_addr: usize,
    pub(crate) parameters: Vec<ShaderParameter>,
    pub(crate) path: String,
}

pub static SHADER_SESSION: ArcSwapOption<ShaderSession> = ArcSwapOption::const_empty();
pub static FRAME_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn windows_shader_snapshot() -> WindowsShaderConfig {
    let guard = WINDOWS_SHADER_CONFIG.load();
    if let Some(arc) = &*guard {
        (**arc).clone()
    } else {
        WindowsShaderConfig {
            enabled: false,
            preset_path: None,
            generation: 1,
        }
    }
}

pub async fn windows_set_shader_preset_path(
    path: Option<String>,
) -> Result<ShaderParameters, String> {
    windows_set_shader_config(true, path).await
}

pub async fn windows_set_shader_config(
    enabled: bool,
    path: Option<String>,
) -> Result<ShaderParameters, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    let mut new_gen = 0;
    let mut changed = false;
    let mut effective_path = String::new();

    WINDOWS_SHADER_CONFIG.rcu(|old| {
        let mut new = old
            .as_ref()
            .map(|a| (**a).clone())
            .unwrap_or(WindowsShaderConfig {
                enabled: false,
                preset_path: None,
                generation: 1,
            });

        // Use new path if provided, otherwise keep old
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
            new.preset_path = target_path.clone();
            new.generation = new.generation.wrapping_add(1);
            new_gen = new.generation;

            effective_path = if enabled && target_path.is_some() {
                target_path.unwrap()
            } else {
                crate::windows::passthrough::get_passthrough_preset()
                    .to_string_lossy()
                    .to_string()
            };

            Some(Arc::new(new))
        }
    });

    if !changed {
        // Return current parameters if no change
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

    let device_addr = LAST_DEVICE_ADDR.load(Ordering::Acquire);
    if device_addr != 0 {
        let loading_gen = LOADING_GENERATION.load(Ordering::Acquire);
        if loading_gen != new_gen
            && LOADING_GENERATION
                .compare_exchange(loading_gen, new_gen, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
        {
            super::chain::reload_shader_chain(&effective_path, device_addr as *mut _, new_gen);
        }
    }

    rx.await
        .map_err(|e| format!("Reload task cancelled: {:?}", e))?
}

pub static RELOAD_CHANNELS: Mutex<VecDeque<oneshot::Sender<Result<ShaderParameters, String>>>> =
    Mutex::new(VecDeque::new());

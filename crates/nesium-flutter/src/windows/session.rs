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

pub fn windows_set_shader_enabled(enabled: bool) {
    let mut new_gen = 0;
    let mut path = None;
    let mut changed = false;

    WINDOWS_SHADER_CONFIG.rcu(|old| {
        let mut new = old
            .as_ref()
            .map(|a| (**a).clone())
            .unwrap_or(WindowsShaderConfig {
                enabled: false,
                preset_path: None,
                generation: 1,
            });

        if new.enabled == enabled {
            changed = false;
            old.clone()
        } else {
            changed = true;
            new.enabled = enabled;
            new.generation = new.generation.wrapping_add(1);
            new_gen = new.generation;
            path = new.preset_path.clone();
            Some(Arc::new(new))
        }
    });

    if changed {
        let device_addr = LAST_DEVICE_ADDR.load(Ordering::Acquire);
        if device_addr != 0 {
            LOADING_GENERATION.store(new_gen, Ordering::Release);

            let effective_path = if enabled && path.is_some() {
                path.unwrap()
            } else {
                crate::windows::passthrough::get_passthrough_preset()
                    .to_string_lossy()
                    .to_string()
            };

            super::chain::reload_shader_chain(&effective_path, device_addr as *mut _, new_gen);
        }
    }
}

pub static RELOAD_CHANNELS: Mutex<VecDeque<oneshot::Sender<Result<ShaderParameters, String>>>> =
    Mutex::new(VecDeque::new());

pub async fn windows_set_shader_preset_path(
    path: Option<String>,
) -> Result<ShaderParameters, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    RELOAD_CHANNELS.lock().push_back(tx);

    let mut new_gen = 0;
    let mut config_enabled = false;
    WINDOWS_SHADER_CONFIG.rcu(|old| {
        let mut new = old
            .as_ref()
            .map(|a| (**a).clone())
            .unwrap_or(WindowsShaderConfig {
                enabled: false,
                preset_path: None,
                generation: 1,
            });

        new.preset_path = path.clone();
        new.generation = new.generation.wrapping_add(1);
        new_gen = new.generation;
        config_enabled = new.enabled;
        Some(Arc::new(new))
    });

    // Use stored device to trigger reload immediately.
    let device_addr = LAST_DEVICE_ADDR.load(Ordering::Acquire);
    if device_addr != 0 {
        LOADING_GENERATION.store(new_gen, Ordering::Release);

        let effective_path = if config_enabled && path.is_some() {
            path.unwrap()
        } else {
            crate::windows::passthrough::get_passthrough_preset()
                .to_string_lossy()
                .to_string()
        };

        super::chain::reload_shader_chain(&effective_path, device_addr as *mut _, new_gen);
    }

    rx.await
        .map_err(|e| format!("Reload task cancelled: {:?}", e))?
}

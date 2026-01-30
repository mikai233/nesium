use arc_swap::ArcSwapOption;
use librashader::preprocess::ShaderParameter;
use librashader::runtime::d3d11::FilterChain as LibrashaderFilterChain;
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

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
            old.clone()
        } else {
            new.enabled = enabled;
            new.generation = new.generation.wrapping_add(1);
            Some(Arc::new(new))
        }
    });
}

pub fn windows_set_shader_preset_path(path: Option<String>) {
    WINDOWS_SHADER_CONFIG.rcu(|old| {
        let mut new = old
            .as_ref()
            .map(|a| (**a).clone())
            .unwrap_or(WindowsShaderConfig {
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
}

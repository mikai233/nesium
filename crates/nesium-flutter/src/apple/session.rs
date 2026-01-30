use arc_swap::ArcSwapOption;
use librashader::preprocess::ShaderParameter;
use librashader::runtime::mtl::FilterChain as LibrashaderFilterChain;
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

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

pub fn apple_set_shader_enabled(enabled: bool) {
    APPLE_SHADER_CONFIG.rcu(|old| {
        let mut new = old
            .as_ref()
            .map(|a| (**a).clone())
            .unwrap_or(AppleShaderConfig {
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

pub fn apple_set_shader_preset_path(path: Option<String>) {
    APPLE_SHADER_CONFIG.rcu(|old| {
        let mut new = old
            .as_ref()
            .map(|a| (**a).clone())
            .unwrap_or(AppleShaderConfig {
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

use nesium_core::controller::Button;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing;

#[cfg(target_os = "android")]
use std::sync::atomic::AtomicI32;

static HIGH_PRIORITY_ENABLED: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "android")]
static RUNTIME_THREAD_TID: AtomicI32 = AtomicI32::new(-1);

#[cfg(any(target_os = "macos", target_os = "ios"))]
const QOS_CLASS_USER_INTERACTIVE: u32 = 0x21;
#[cfg(any(target_os = "macos", target_os = "ios"))]
const QOS_CLASS_DEFAULT: u32 = 0x15;

#[cfg(any(target_os = "macos", target_os = "ios"))]
unsafe extern "C" {
    fn pthread_set_qos_class_self_np(qos_class: u32, relative_priority: i32) -> i32;
}

pub fn set_high_priority_enabled(enabled: bool) {
    tracing::info!("High priority enabled set to: {}", enabled);
    HIGH_PRIORITY_ENABLED.store(enabled, Ordering::Release);

    #[cfg(target_os = "android")]
    {
        let tid = RUNTIME_THREAD_TID.load(Ordering::Acquire);
        if tid > 0 {
            try_set_thread_nice(tid, if enabled { -2 } else { 0 });
        }
    }

    #[cfg(not(target_os = "android"))]
    apply_priority_to_current_thread(enabled);
}

pub fn is_high_priority_enabled() -> bool {
    HIGH_PRIORITY_ENABLED.load(Ordering::Acquire)
}

pub(crate) fn button_bit(button: Button) -> u8 {
    match button {
        Button::A => 0,
        Button::B => 1,
        Button::Select => 2,
        Button::Start => 3,
        Button::Up => 4,
        Button::Down => 5,
        Button::Left => 6,
        Button::Right => 7,
    }
}

pub(crate) fn apply_priority_to_current_thread(enabled: bool) {
    #[cfg(target_os = "android")]
    {
        let tid = unsafe { libc::gettid() as i32 };
        RUNTIME_THREAD_TID.store(tid, Ordering::Release);
        try_set_thread_nice(tid, if enabled { -2 } else { 0 });
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        unsafe {
            let qos = if enabled {
                QOS_CLASS_USER_INTERACTIVE
            } else {
                QOS_CLASS_DEFAULT
            };
            let res = pthread_set_qos_class_self_np(qos, 0);
            if res == 0 {
                tracing::info!(
                    "macOS Thread QoS {}: {}",
                    if enabled { "raised" } else { "restored" },
                    if enabled {
                        "UserInteractive"
                    } else {
                        "Default"
                    }
                );
            } else {
                tracing::error!("Failed to set macOS Thread QoS: {}", res);
            }
        }
    }

    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Threading::{
            GetCurrentProcess, GetCurrentThread, HIGH_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS,
            SetPriorityClass, SetThreadPriority, THREAD_PRIORITY_HIGHEST, THREAD_PRIORITY_NORMAL,
        };
        unsafe {
            // Process priority class
            let process = GetCurrentProcess();
            let p_priority = if enabled {
                HIGH_PRIORITY_CLASS
            } else {
                NORMAL_PRIORITY_CLASS
            };
            if SetPriorityClass(process, p_priority) != 0 {
                tracing::info!(
                    "Windows process priority class set to: {}",
                    if enabled { "HIGH" } else { "NORMAL" }
                );
            }

            // Thread priority
            let thread = GetCurrentThread();
            let t_priority = if enabled {
                THREAD_PRIORITY_HIGHEST
            } else {
                THREAD_PRIORITY_NORMAL
            };
            if SetThreadPriority(thread, t_priority) != 0 {
                tracing::info!(
                    "Windows thread priority set to: {}",
                    if enabled { "HIGHEST" } else { "NORMAL" }
                );
            }

            if enabled {
                windows_sys::Win32::Media::timeBeginPeriod(1);
            } else {
                windows_sys::Win32::Media::timeEndPeriod(1);
            }
        }
    }
}

pub(crate) fn try_raise_current_thread_priority() {
    apply_priority_to_current_thread(is_high_priority_enabled());
}

#[cfg(not(any(target_os = "android", windows, target_os = "macos", target_os = "ios")))]
pub(crate) fn try_raise_current_thread_priority() {}

#[cfg(target_os = "android")]
fn try_set_thread_nice(tid: i32, nice: i32) {
    let nice = nice.clamp(-20, 19) as libc::c_int;
    unsafe {
        let tid = tid as libc::id_t;
        let res = libc::setpriority(libc::PRIO_PROCESS, tid, nice);
        if res == 0 {
            tracing::info!("Android thread priority (nice) set to: {}", nice);
        } else {
            tracing::error!("Failed to set Android thread priority: {}", res);
        }
    }
}

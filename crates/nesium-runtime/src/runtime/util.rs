use nesium_core::controller::Button;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

static HIGH_PRIORITY_ENABLED: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "android")]
static RUNTIME_THREAD_TID: AtomicI32 = AtomicI32::new(-1);

pub fn set_high_priority_enabled(enabled: bool) {
    HIGH_PRIORITY_ENABLED.store(enabled, Ordering::Release);

    #[cfg(target_os = "android")]
    {
        let tid = RUNTIME_THREAD_TID.load(Ordering::Acquire);
        if tid > 0 {
            try_set_thread_nice(tid, if enabled { -2 } else { 0 });
        }
    }
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

#[cfg(target_os = "android")]
pub(crate) fn try_raise_current_thread_priority() {
    // Store the tid even if the boost is disabled so we can apply it later.
    let tid = unsafe { libc::gettid() as i32 };
    RUNTIME_THREAD_TID.store(tid, Ordering::Release);

    if !is_high_priority_enabled() {
        return;
    }

    try_set_thread_nice(tid, -2);
}

#[cfg(not(target_os = "android"))]
pub(crate) fn try_raise_current_thread_priority() {}

#[cfg(target_os = "android")]
fn try_set_thread_nice(tid: i32, nice: i32) {
    let nice = nice.clamp(-20, 19) as libc::c_int;
    unsafe {
        let tid = tid as libc::id_t;
        let _ = libc::setpriority(libc::PRIO_PROCESS, tid, nice);
    }
}

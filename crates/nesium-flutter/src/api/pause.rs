use flutter_rust_bridge::frb;

#[frb]
pub fn set_paused(paused: bool) {
    crate::runtime_handle().set_paused(paused);
}

#[frb]
pub fn is_paused() -> bool {
    crate::runtime_handle().paused()
}

#[frb]
pub fn toggle_pause() -> bool {
    let handle = crate::runtime_handle();
    let next = !handle.paused();
    handle.set_paused(next);
    next
}

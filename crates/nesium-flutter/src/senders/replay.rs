use flutter_rust_bridge::frb;
use std::sync::{Mutex, OnceLock};

use crate::frb_generated::StreamSink;

static REPLAY_SINK: OnceLock<Mutex<Option<StreamSink<ReplayEventNotification>>>> = OnceLock::new();

#[frb]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayEventNotification {
    QuickSave,
    QuickLoad,
}

pub fn set_replay_sink(sink: StreamSink<ReplayEventNotification>) {
    let mutex = REPLAY_SINK.get_or_init(|| Mutex::new(None));
    let mut guard = mutex.lock().unwrap();
    *guard = Some(sink);
}

pub fn emit_replay_event(event: ReplayEventNotification) {
    if let Some(mutex) = REPLAY_SINK.get() {
        let guard = mutex.lock().unwrap();
        if let Some(sink) = &*guard {
            let _ = sink.add(event);
            return;
        }
    }
    tracing::warn!("Replay event dropped (no sink registered)");
}

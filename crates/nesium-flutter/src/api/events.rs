use std::sync::atomic::{AtomicBool, Ordering};

use flutter_rust_bridge::frb;
use nesium_runtime::RuntimeNotification as CoreRuntimeNotification;

use crate::frb_generated::StreamSink;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeNotificationKind {
    AudioInitFailed,
}

#[derive(Debug, Clone)]
pub struct RuntimeNotification {
    pub kind: RuntimeNotificationKind,
    pub error: Option<String>,
}

impl From<CoreRuntimeNotification> for RuntimeNotification {
    fn from(value: CoreRuntimeNotification) -> Self {
        match value {
            CoreRuntimeNotification::AudioInitFailed { error } => RuntimeNotification {
                kind: RuntimeNotificationKind::AudioInitFailed,
                error: Some(error),
            },
        }
    }
}

static NOTIFICATION_STREAM_STARTED: AtomicBool = AtomicBool::new(false);

/// Runtime notification stream.
///
/// This blocks on the runtime notification receiver and forwards notifications to Dart.
#[frb]
pub async fn runtime_notifications(sink: StreamSink<RuntimeNotification>) -> Result<(), String> {
    if NOTIFICATION_STREAM_STARTED.swap(true, Ordering::AcqRel) {
        let _ = sink.add_error("runtime notification stream already started".to_string());
        return Ok(());
    }

    let Some(handle) = crate::try_runtime_handle() else {
        NOTIFICATION_STREAM_STARTED.store(false, Ordering::Release);
        return Err("runtime not started".to_string());
    };
    let handle = handle.clone();
    std::thread::spawn(move || {
        loop {
            let Some(notification) = handle.recv_notification_blocking() else {
                break;
            };
            if sink.add(RuntimeNotification::from(notification)).is_err() {
                break;
            }
        }
        NOTIFICATION_STREAM_STARTED.store(false, Ordering::Release);
    });

    Ok(())
}

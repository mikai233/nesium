use flutter_rust_bridge::frb;
use nesium_runtime::runtime::EventTopic;
use nesium_runtime::{RuntimeEvent, RuntimeEventSender};

use crate::frb_generated::StreamSink;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeNotificationKind {
    AudioInitFailed,
}

impl RuntimeNotificationKind {
    pub fn from_event(event: &RuntimeEvent) -> Self {
        match event {
            RuntimeEvent::AudioInitFailed { .. } => Self::AudioInitFailed,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeNotification {
    pub kind: RuntimeNotificationKind,
    pub error: Option<String>,
}

impl From<RuntimeEvent> for RuntimeNotification {
    fn from(value: RuntimeEvent) -> Self {
        RuntimeNotification {
            kind: RuntimeNotificationKind::from_event(&value),
            error: match value {
                RuntimeEvent::AudioInitFailed { error } => Some(error),
            },
        }
    }
}

pub struct FlutterRuntimeEventSender {
    sink: StreamSink<RuntimeNotification>,
}

impl FlutterRuntimeEventSender {
    pub fn new(sink: StreamSink<RuntimeNotification>) -> Self {
        Self { sink }
    }
}

impl RuntimeEventSender for FlutterRuntimeEventSender {
    fn send(&self, event: RuntimeEvent) -> bool {
        let _: Result<_, _> = self.sink.add(RuntimeNotification::from(event));
        true // Always return true to avoid being pruned by RuntimePubSub
    }
}

/// Runtime notification stream.
///
/// This registers the sink directly with the runtime.
#[frb]
pub async fn runtime_notifications(sink: StreamSink<RuntimeNotification>) -> Result<(), String> {
    // If the runtime is already running, subscribe.
    // runtime_handle() returns &'static Handle, so it is valid if ensure_runtime has been called (which usually happens on app start).
    let handle = crate::runtime_handle();
    let sender = FlutterRuntimeEventSender::new(sink);

    // Subscribe to relevant topics
    handle
        .subscribe_event(EventTopic::Notification, sender)
        .map_err(|e| format!("Failed to subscribe to Notification events: {}", e))?;

    Ok(())
}

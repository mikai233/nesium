use std::any::Any;

use nesium_runtime::{Event, NotificationEvent, RuntimeEventSender};

use crate::api::events::{RuntimeNotification, RuntimeNotificationKind};
use crate::frb_generated::StreamSink;

/// Sender that forwards RuntimeEvent to Flutter as RuntimeNotification.
pub struct FlutterRuntimeEventSender {
    pub(crate) sink: StreamSink<RuntimeNotification>,
}

impl FlutterRuntimeEventSender {
    pub fn new(sink: StreamSink<RuntimeNotification>) -> Self {
        Self { sink }
    }
}

impl RuntimeEventSender for FlutterRuntimeEventSender {
    fn send(&self, event: Box<dyn Event>) -> bool {
        let any: Box<dyn Any> = event;
        if let Ok(notification) = any.downcast::<NotificationEvent>() {
            let notification = match *notification {
                NotificationEvent::AudioInitFailed { error } => RuntimeNotification {
                    kind: RuntimeNotificationKind::AudioInitFailed,
                    error: Some(error),
                },
            };
            let _ = self.sink.add(notification);
            return true;
        }
        true
    }
}

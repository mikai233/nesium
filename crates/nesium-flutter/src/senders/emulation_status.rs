use std::any::Any;

use nesium_runtime::runtime::EmulationStatus;
use nesium_runtime::{Event, RuntimeEventSender};

use crate::api::events::EmulationStatusNotification;
use crate::frb_generated::StreamSink;

pub struct EmulationStatusSender {
    pub(crate) sink: StreamSink<EmulationStatusNotification>,
}

impl EmulationStatusSender {
    pub fn new(sink: StreamSink<EmulationStatusNotification>) -> Self {
        Self { sink }
    }
}

impl RuntimeEventSender for EmulationStatusSender {
    fn send(&self, event: Box<dyn Event>) -> bool {
        let any: Box<dyn Any> = event;
        if let Ok(status) = any.downcast::<EmulationStatus>() {
            let _ = self.sink.add(EmulationStatusNotification {
                paused: status.paused,
                rewinding: status.rewinding,
                fast_forwarding: status.fast_forwarding,
            });
            return true;
        }
        true
    }
}

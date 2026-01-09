//! History Viewer state sender for Flutter.
//!
//! This sender converts runtime `HistoryState` events into `HistorySnapshot`
//! for the Flutter UI.

use std::any::Any;

use nesium_runtime::{Event, HistoryState, RuntimeEventSender};

use crate::api::events::HistorySnapshot;
use crate::frb_generated::StreamSink;

pub const HISTORY_TEXTURE_ID: u32 = 5;

/// Sender that streams HistorySnapshot to Flutter for the History Viewer.
pub struct HistoryStateSender {
    sink: StreamSink<HistorySnapshot>,
}

impl HistoryStateSender {
    pub fn new(sink: StreamSink<HistorySnapshot>) -> Self {
        Self { sink }
    }
}

impl RuntimeEventSender for HistoryStateSender {
    fn send(&self, event: Box<dyn Event>) -> bool {
        let any: Box<dyn Any> = event;
        if let Ok(state) = any.downcast::<HistoryState>() {
            let task = crate::event_worker::EventTask::History {
                state,
                sink: self.sink.clone(),
            };
            return crate::event_worker::event_worker().send(task);
        }
        true
    }
}

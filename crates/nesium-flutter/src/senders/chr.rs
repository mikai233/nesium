use std::any::Any;

use nesium_runtime::{Event, RuntimeEventSender};

use crate::frb_generated::StreamSink;

/// Auxiliary texture ID for CHR/Tile Viewer.
pub const CHR_TEXTURE_ID: u32 = 2;

/// Sender that updates the CHR auxiliary texture AND streams ChrSnapshot to Flutter.
pub struct ChrTextureAndStateSender {
    sink: StreamSink<crate::api::events::ChrSnapshot>,
}

impl ChrTextureAndStateSender {
    pub fn new(sink: StreamSink<crate::api::events::ChrSnapshot>) -> Self {
        Self { sink }
    }
}

impl RuntimeEventSender for ChrTextureAndStateSender {
    fn send(&self, event: Box<dyn Event>) -> bool {
        let any: Box<dyn Any> = event;
        if let Ok(state) = any.downcast::<nesium_runtime::ChrState>() {
            // Queue all work to worker thread - texture update and streaming
            let _ = crate::event_worker::event_worker().send(crate::event_worker::EventTask::Chr {
                state,
                sink: self.sink.clone(),
            });
            return true;
        }
        true
    }
}

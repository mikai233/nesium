use std::any::Any;

use nesium_runtime::{Event, RuntimeEventSender};

use crate::frb_generated::StreamSink;

/// Auxiliary texture ID for Tile Viewer.
pub const TILE_VIEWER_TEXTURE_ID: u32 = 2;

/// Sender that updates the Tile auxiliary texture AND streams TileSnapshot to Flutter.
pub struct TileTextureAndStateSender {
    sink: StreamSink<crate::api::events::TileSnapshot>,
}

impl TileTextureAndStateSender {
    pub fn new(sink: StreamSink<crate::api::events::TileSnapshot>) -> Self {
        Self { sink }
    }
}

impl RuntimeEventSender for TileTextureAndStateSender {
    fn send(&self, event: Box<dyn Event>) -> bool {
        let any: Box<dyn Any> = event;
        if let Ok(state) = any.downcast::<nesium_runtime::TileState>() {
            // Queue all work to worker thread - texture update and streaming
            let _ =
                crate::event_worker::event_worker().send(crate::event_worker::EventTask::Tile {
                    state,
                    sink: self.sink.clone(),
                });
            return true;
        }
        true
    }
}

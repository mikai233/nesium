use std::any::Any;

use nesium_runtime::{Event, RuntimeEventSender};

use crate::frb_generated::StreamSink;

/// Auxiliary texture ID for Sprite Viewer thumbnails.
pub const SPRITE_TEXTURE_ID: u32 = 3;
/// Auxiliary texture ID for Sprite Viewer screen preview.
pub const SPRITE_SCREEN_TEXTURE_ID: u32 = 4;

/// Sender that updates the Sprite auxiliary texture AND streams SpriteSnapshot to Flutter.
pub struct SpriteTextureAndStateSender {
    sink: StreamSink<crate::api::events::SpriteSnapshot>,
}

impl SpriteTextureAndStateSender {
    pub fn new(sink: StreamSink<crate::api::events::SpriteSnapshot>) -> Self {
        Self { sink }
    }
}

impl RuntimeEventSender for SpriteTextureAndStateSender {
    fn send(&self, event: Box<dyn Event>) -> bool {
        let any: Box<dyn Any> = event;
        if let Ok(state) = any.downcast::<nesium_runtime::SpriteState>() {
            // Queue all work to worker thread - texture updates and streaming
            let _ =
                crate::event_worker::event_worker().send(crate::event_worker::EventTask::Sprite {
                    state,
                    sink: self.sink.clone(),
                });
            return true;
        }
        true
    }
}

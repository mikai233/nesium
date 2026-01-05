use std::any::Any;

use nesium_runtime::{Event, PaletteState, RuntimeEventSender};

use crate::api::events::PaletteSnapshot;
use crate::frb_generated::StreamSink;

/// Sender that streams PaletteSnapshot to Flutter.
///
/// Unlike other viewers, the Palette Viewer doesn't use auxiliary textures
/// (Flutter renders the 32 color boxes directly).
pub struct PaletteStateSender {
    sink: StreamSink<PaletteSnapshot>,
}

impl PaletteStateSender {
    pub fn new(sink: StreamSink<PaletteSnapshot>) -> Self {
        Self { sink }
    }
}

impl RuntimeEventSender for PaletteStateSender {
    fn send(&self, event: Box<dyn Event>) -> bool {
        let any: Box<dyn Any> = event;
        if let Ok(state) = any.downcast::<PaletteState>() {
            // Flatten the BGRA palette: 64 colors × 4 bytes = 256 bytes
            let flattened_palette: Vec<u8> = state
                .bgra_palette
                .iter()
                .flat_map(|c| c.iter().copied())
                .collect();

            let snapshot = PaletteSnapshot {
                palette: state.palette.to_vec(),
                bgra_palette: flattened_palette,
            };
            let _ = self.sink.add(snapshot);
        }
        true
    }
}

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
            // Update auxiliary texture
            crate::aux_texture::aux_update(CHR_TEXTURE_ID, &state.rgba);

            // Convert BGRA palette to RGBA for Flutter
            let mut rgba_palette = Vec::with_capacity(64 * 4);
            for px in state.bgra_palette.iter() {
                #[cfg(any(target_os = "macos", target_os = "ios"))]
                {
                    rgba_palette.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
                }
                #[cfg(not(any(target_os = "macos", target_os = "ios")))]
                {
                    rgba_palette.extend_from_slice(px);
                }
            }

            let snapshot = crate::api::events::ChrSnapshot {
                palette: state.palette.to_vec(),
                rgba_palette,
                selected_palette: state.selected_palette,
                width: state.width,
                height: state.height,
                source: state.source as u8,
                source_size: state.source_size,
                start_address: state.start_address,
                column_count: state.column_count,
                row_count: state.row_count,
                layout: state.layout as u8,
                background: state.background as u8,
                use_grayscale_palette: state.use_grayscale_palette,
                bg_pattern_base: state.bg_pattern_base,
                sprite_pattern_base: state.sprite_pattern_base,
                large_sprites: state.large_sprites,
            };

            let _ = self.sink.add(snapshot);
            return true;
        }
        true
    }
}

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
            // Update auxiliary texture with thumbnails
            crate::aux_texture::aux_update(SPRITE_TEXTURE_ID, &state.thumbnails_rgba);
            // Update auxiliary texture with screen preview
            crate::aux_texture::aux_update(SPRITE_SCREEN_TEXTURE_ID, &state.screen_rgba);

            // Convert sprites to FRB-compatible format
            let sprites: Vec<crate::api::events::SpriteInfo> = state
                .sprites
                .iter()
                .map(|s| crate::api::events::SpriteInfo {
                    index: s.index,
                    x: s.x,
                    y: s.y,
                    tile_index: s.tile_index,
                    palette: s.palette,
                    flip_h: s.flip_h,
                    flip_v: s.flip_v,
                    behind_bg: s.behind_bg,
                    visible: s.visible,
                })
                .collect();

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

            let snapshot = crate::api::events::SpriteSnapshot {
                sprites,
                thumbnail_width: state.thumbnail_width,
                thumbnail_height: state.thumbnail_height,
                large_sprites: state.large_sprites,
                pattern_base: state.pattern_base,
                rgba_palette,
            };

            let _ = self.sink.add(snapshot);
            return true;
        }
        true
    }
}

#pragma once

#include <flutter_linux/flutter_linux.h>
#include <stdint.h>

G_BEGIN_DECLS

// Auxiliary texture for debugger views (Tilemap, Pattern, etc.)
//
// Similar to NesiumTexture but:
// - Identified by a unique ID
// - Data comes from Rust aux_texture module instead of NES emulator
//
// Each auxiliary texture is a CPU-backed pixel buffer that Flutter pulls
// via `copy_pixels()` on the engine thread.
#define NESIUM_TYPE_AUX_TEXTURE (nesium_aux_texture_get_type())
G_DECLARE_FINAL_TYPE(NesiumAuxTexture, nesium_aux_texture, NESIUM, AUX_TEXTURE,
                     FlPixelBufferTexture)

NesiumAuxTexture *nesium_aux_texture_new(uint32_t id, uint32_t width,
                                         uint32_t height);

// Returns the ID of this auxiliary texture.
uint32_t nesium_aux_texture_get_id(NesiumAuxTexture *texture);

// Copies from Rust buffer into the back buffer and commits.
void nesium_aux_texture_update_from_rust(NesiumAuxTexture *texture);

G_END_DECLS

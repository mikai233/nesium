#pragma once

#include <flutter_linux/flutter_linux.h>
#include <stdint.h>

G_BEGIN_DECLS

// A CPU-backed pixel buffer texture used by the Linux runner.
//
// - Flutter pulls pixels via `copy_pixels()` on the engine thread.
// - A background worker thread copies the latest Rust frame into the back buffer.
// - After publishing, the runner marks the texture as frame-available.
#define NESIUM_TYPE_TEXTURE (nesium_texture_get_type())
G_DECLARE_FINAL_TYPE(NesiumTexture, nesium_texture, NESIUM, TEXTURE,
                     FlPixelBufferTexture)

NesiumTexture *nesium_texture_new();

// Prepares a writable back buffer for the next frame.
// Returns false on allocation failure.
//
// The returned pointer remains valid until `nesium_texture_end_write()` is called.
// The caller must write tightly-packed RGBA pixels with the given stride.
bool nesium_texture_begin_write(NesiumTexture *texture,
                                uint32_t width,
                                uint32_t height,
                                uint32_t stride_bytes,
                                uint8_t **out_ptr);

// Publishes the last begun write as the new front buffer.
void nesium_texture_end_write(NesiumTexture *texture);

G_END_DECLS
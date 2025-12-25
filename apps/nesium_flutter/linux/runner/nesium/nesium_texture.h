#pragma once

#include <flutter_linux/flutter_linux.h>

G_BEGIN_DECLS

#define NESIUM_TYPE_TEXTURE (nesium_texture_get_type())
G_DECLARE_FINAL_TYPE(NesiumTexture, nesium_texture, NESIUM, TEXTURE,
                     FlPixelBufferTexture)

NesiumTexture *nesium_texture_new();

G_END_DECLS

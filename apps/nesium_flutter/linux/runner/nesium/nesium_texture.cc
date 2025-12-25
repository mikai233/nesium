#include "nesium_texture.h"

#include <cstddef>
#include <cstdint>

constexpr uint32_t kWidth = 256;
constexpr uint32_t kHeight = 240;

struct _NesiumTexture {
  FlPixelBufferTexture parent_instance;

  uint8_t *pixels = nullptr;
  size_t size_bytes = 0;
};

G_DEFINE_TYPE(NesiumTexture, nesium_texture, fl_pixel_buffer_texture_get_type())

static gboolean nesium_texture_copy_pixels(FlPixelBufferTexture *texture,
                                           const uint8_t **out_buffer,
                                           uint32_t *width, uint32_t *height,
                                           GError ** /*error*/) {
  auto *self = NESIUM_TEXTURE(texture);
  if (self->pixels == nullptr) {
    return FALSE;
  }

  *out_buffer = self->pixels;
  *width = kWidth;
  *height = kHeight;
  return TRUE;
}

static void nesium_texture_dispose(GObject *object) {
  auto *self = NESIUM_TEXTURE(object);

  if (self->pixels != nullptr) {
    g_free(self->pixels);
    self->pixels = nullptr;
    self->size_bytes = 0;
  }

  G_OBJECT_CLASS(nesium_texture_parent_class)->dispose(object);
}

static void nesium_texture_class_init(NesiumTextureClass *klass) {
  auto *gobject_class = G_OBJECT_CLASS(klass);
  gobject_class->dispose = nesium_texture_dispose;

  auto *pixel_texture_class = FL_PIXEL_BUFFER_TEXTURE_CLASS(klass);
  pixel_texture_class->copy_pixels = nesium_texture_copy_pixels;
}

static void nesium_texture_init(NesiumTexture *texture) {
  auto *self = NESIUM_TEXTURE(texture);

  self->size_bytes =
      static_cast<size_t>(kWidth) * static_cast<size_t>(kHeight) * 4u;
  self->pixels = static_cast<uint8_t *>(g_malloc(self->size_bytes));

  // Deterministic test pattern (verify external texture path before wiring
  // Rust).
  for (uint32_t y = 0; y < kHeight; y++) {
    for (uint32_t x = 0; x < kWidth; x++) {
      const size_t i = (static_cast<size_t>(y) * kWidth + x) * 4u;
      self->pixels[i + 0] = static_cast<uint8_t>(x);     // R
      self->pixels[i + 1] = static_cast<uint8_t>(y);     // G
      self->pixels[i + 2] = static_cast<uint8_t>(x ^ y); // B
      self->pixels[i + 3] = 255;                         // A
    }
  }
}

NesiumTexture *nesium_texture_new() {
  return NESIUM_TEXTURE(g_object_new(NESIUM_TYPE_TEXTURE, nullptr));
}

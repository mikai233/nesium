#include "nesium_aux_texture.h"

#include <stddef.h>

#include <glib.h>

// C ABI from Rust (aux_texture.rs)
extern "C" {
void nesium_aux_create(uint32_t id, uint32_t width, uint32_t height);
size_t nesium_aux_copy(uint32_t id, uint8_t *dst, uint32_t dst_pitch,
                       uint32_t dst_height);
void nesium_aux_destroy(uint32_t id);
}

namespace {
// A permanent fallback pixel used before the first real frame is published.
constexpr uint8_t kFallbackPixelRGBA[4] = {0, 0, 0, 0};
} // namespace

// IMPORTANT:
// This struct MUST be defined in the global namespace because the type is
// forward-declared by G_DECLARE_FINAL_TYPE in the header.
struct _NesiumAuxTexture {
  FlPixelBufferTexture parent_instance;

  uint32_t id;

  // Protects buffer pointers and metadata below.
  GMutex mutex;

  // Double-buffered, CPU-owned RGBA pixels.
  uint8_t *buffers[2] = {nullptr, nullptr};
  size_t buffer_capacity = 0; // bytes per buffer.

  // Front buffer index used by `copy_pixels()`.
  gint front_index = 0;

  // Published frame metadata for the current front buffer.
  gboolean has_frame = FALSE;
  uint32_t width = 0;
  uint32_t height = 0;
};

G_DEFINE_TYPE(NesiumAuxTexture, nesium_aux_texture,
              fl_pixel_buffer_texture_get_type())

static gboolean nesium_aux_texture_copy_pixels(FlPixelBufferTexture *texture,
                                               const uint8_t **out_buffer,
                                               uint32_t *width,
                                               uint32_t *height,
                                               GError ** /*error*/) {
  auto *self = NESIUM_AUX_TEXTURE(texture);

  // Always initialize output parameters.
  *out_buffer = kFallbackPixelRGBA;
  *width = 1;
  *height = 1;

  g_mutex_lock(&self->mutex);

  const int front = self->front_index;
  const uint8_t *ptr = self->buffers[front];
  const gboolean has_frame = self->has_frame;
  const uint32_t w = self->width;
  const uint32_t h = self->height;
  const size_t cap = self->buffer_capacity;

  g_mutex_unlock(&self->mutex);

  if (!has_frame || ptr == nullptr || w == 0 || h == 0) {
    return TRUE;
  }

  const size_t needed = static_cast<size_t>(w) * static_cast<size_t>(h) * 4u;
  if (needed == 0 || needed > cap) {
    return TRUE;
  }

  *out_buffer = ptr;
  *width = w;
  *height = h;
  return TRUE;
}

static void nesium_aux_texture_dispose(GObject *object) {
  G_OBJECT_CLASS(nesium_aux_texture_parent_class)->dispose(object);
}

static void nesium_aux_texture_finalize(GObject *object) {
  auto *self = NESIUM_AUX_TEXTURE(object);

  // Destroy Rust-side backing store
  nesium_aux_destroy(self->id);

  g_mutex_lock(&self->mutex);

  if (self->buffers[0] != nullptr) {
    g_free(self->buffers[0]);
    self->buffers[0] = nullptr;
  }
  if (self->buffers[1] != nullptr) {
    g_free(self->buffers[1]);
    self->buffers[1] = nullptr;
  }

  self->buffer_capacity = 0;
  self->front_index = 0;
  self->has_frame = FALSE;
  self->width = 0;
  self->height = 0;

  g_mutex_unlock(&self->mutex);

  g_mutex_clear(&self->mutex);

  G_OBJECT_CLASS(nesium_aux_texture_parent_class)->finalize(object);
}

static void nesium_aux_texture_class_init(NesiumAuxTextureClass *klass) {
  auto *gobject_class = G_OBJECT_CLASS(klass);
  gobject_class->dispose = nesium_aux_texture_dispose;
  gobject_class->finalize = nesium_aux_texture_finalize;

  auto *pixel_texture_class = FL_PIXEL_BUFFER_TEXTURE_CLASS(klass);
  pixel_texture_class->copy_pixels = nesium_aux_texture_copy_pixels;
}

static void nesium_aux_texture_init(NesiumAuxTexture *texture) {
  auto *self = NESIUM_AUX_TEXTURE(texture);
  g_mutex_init(&self->mutex);

  self->front_index = 0;
  self->has_frame = FALSE;
}

static bool ensure_capacity_once(NesiumAuxTexture *self, size_t needed_bytes) {
  if (needed_bytes == 0)
    return false;

  // Allocate exactly once. If the texture size changes later, reject the
  // update.
  if (self->buffer_capacity != 0) {
    return needed_bytes <= self->buffer_capacity &&
           self->buffers[0] != nullptr && self->buffers[1] != nullptr;
  }

  uint8_t *b0 = static_cast<uint8_t *>(g_malloc(needed_bytes));
  uint8_t *b1 = static_cast<uint8_t *>(g_malloc(needed_bytes));
  if (b0 == nullptr || b1 == nullptr) {
    if (b0 != nullptr)
      g_free(b0);
    if (b1 != nullptr)
      g_free(b1);
    return false;
  }

  self->buffers[0] = b0;
  self->buffers[1] = b1;
  self->buffer_capacity = needed_bytes;
  return true;
}

NesiumAuxTexture *nesium_aux_texture_new(uint32_t id, uint32_t width,
                                         uint32_t height) {
  auto *texture =
      NESIUM_AUX_TEXTURE(g_object_new(NESIUM_TYPE_AUX_TEXTURE, nullptr));

  texture->id = id;
  texture->width = width;
  texture->height = height;

  const size_t needed =
      static_cast<size_t>(width) * static_cast<size_t>(height) * 4u;

  g_mutex_lock(&texture->mutex);
  ensure_capacity_once(texture, needed);
  g_mutex_unlock(&texture->mutex);

  // Create Rust-side backing store
  nesium_aux_create(id, width, height);

  return texture;
}

uint32_t nesium_aux_texture_get_id(NesiumAuxTexture *texture) {
  if (texture == nullptr)
    return 0;
  return texture->id;
}

void nesium_aux_texture_update_from_rust(NesiumAuxTexture *texture) {
  if (texture == nullptr)
    return;

  auto *self = NESIUM_AUX_TEXTURE(texture);

  g_mutex_lock(&self->mutex);

  const int front = self->front_index;
  const int back = 1 - front;

  uint8_t *dst = self->buffers[back];
  const uint32_t w = self->width;
  const uint32_t h = self->height;
  const size_t cap = self->buffer_capacity;

  g_mutex_unlock(&self->mutex);

  if (dst == nullptr || w == 0 || h == 0)
    return;

  const uint32_t pitch = w * 4u;
  const size_t needed = static_cast<size_t>(pitch) * static_cast<size_t>(h);

  if (needed == 0 || needed > cap)
    return;

  // Copy from Rust buffer
  const size_t copied = nesium_aux_copy(self->id, dst, pitch, h);

  if (copied > 0) {
    // Publish the back buffer as the new front buffer
    g_mutex_lock(&self->mutex);
    self->front_index = back;
    self->has_frame = TRUE;
    g_mutex_unlock(&self->mutex);
  }
}

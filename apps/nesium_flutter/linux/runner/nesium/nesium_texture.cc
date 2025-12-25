#include "nesium_texture.h"

#include <stddef.h>

// IMPORTANT:
// This struct MUST be defined in the global namespace because the type is
// forward-declared by G_DECLARE_FINAL_TYPE in the header.
struct _NesiumTexture {
  FlPixelBufferTexture parent_instance;

  // Protects buffer pointers and metadata below.
  GMutex mutex;

  // Double-buffered, CPU-owned RGBA pixels.
  uint8_t* buffers[2] = {nullptr, nullptr};
  size_t buffer_capacity = 0;  // bytes, per-buffer.

  // Front buffer index used by `copy_pixels()`.
  gint front_index = 0;

  // Frame metadata for the current front buffer.
  uint32_t width = 0;
  uint32_t height = 0;
  uint32_t stride_bytes = 0;

  // Write-in-progress state (back buffer).
  gint write_index = 1;
  uint32_t write_width = 0;
  uint32_t write_height = 0;
  uint32_t write_stride_bytes = 0;
};

G_DEFINE_TYPE(NesiumTexture, nesium_texture, fl_pixel_buffer_texture_get_type())

static gboolean nesium_texture_copy_pixels(FlPixelBufferTexture* texture,
                                          const uint8_t** out_buffer,
                                          uint32_t* width,
                                          uint32_t* height,
                                          GError** /*error*/) {
  auto* self = NESIUM_TEXTURE(texture);

  g_mutex_lock(&self->mutex);

  const int front = g_atomic_int_get(&self->front_index);
  const uint8_t* ptr = self->buffers[front];
  const uint32_t w = self->width;
  const uint32_t h = self->height;

  g_mutex_unlock(&self->mutex);

  if (ptr == nullptr || w == 0 || h == 0) {
    return FALSE;
  }

  *out_buffer = ptr;
  *width = w;
  *height = h;
  return TRUE;
}

static void nesium_texture_dispose(GObject* object) {
  auto* self = NESIUM_TEXTURE(object);

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
  self->width = 0;
  self->height = 0;
  self->stride_bytes = 0;
  g_mutex_unlock(&self->mutex);

  g_mutex_clear(&self->mutex);

  G_OBJECT_CLASS(nesium_texture_parent_class)->dispose(object);
}

static void nesium_texture_class_init(NesiumTextureClass* klass) {
  auto* gobject_class = G_OBJECT_CLASS(klass);
  gobject_class->dispose = nesium_texture_dispose;

  auto* pixel_texture_class = FL_PIXEL_BUFFER_TEXTURE_CLASS(klass);
  pixel_texture_class->copy_pixels = nesium_texture_copy_pixels;
}

static void nesium_texture_init(NesiumTexture* texture) {
  auto* self = NESIUM_TEXTURE(texture);
  g_mutex_init(&self->mutex);

  // Start with a valid but empty state.
  self->front_index = 0;
  self->write_index = 1;
}

static bool ensure_capacity(NesiumTexture* self, size_t needed_bytes) {
  if (needed_bytes == 0) return false;
  if (self->buffer_capacity >= needed_bytes && self->buffers[0] != nullptr &&
      self->buffers[1] != nullptr) {
    return true;
  }

  // Reallocate both buffers to the new capacity.
  uint8_t* b0 = static_cast<uint8_t*>(g_malloc(needed_bytes));
  uint8_t* b1 = static_cast<uint8_t*>(g_malloc(needed_bytes));
  if (b0 == nullptr || b1 == nullptr) {
    if (b0 != nullptr) g_free(b0);
    if (b1 != nullptr) g_free(b1);
    return false;
  }

  if (self->buffers[0] != nullptr) g_free(self->buffers[0]);
  if (self->buffers[1] != nullptr) g_free(self->buffers[1]);

  self->buffers[0] = b0;
  self->buffers[1] = b1;
  self->buffer_capacity = needed_bytes;
  return true;
}

NesiumTexture* nesium_texture_new() {
  return NESIUM_TEXTURE(g_object_new(NESIUM_TYPE_TEXTURE, nullptr));
}

bool nesium_texture_begin_write(NesiumTexture* texture,
                               uint32_t width,
                               uint32_t height,
                               uint32_t stride_bytes,
                               uint8_t** out_ptr) {
  if (texture == nullptr || out_ptr == nullptr) return false;
  if (width == 0 || height == 0) return false;
  if (stride_bytes < width * 4u) return false;

  auto* self = NESIUM_TEXTURE(texture);

  const size_t needed =
      static_cast<size_t>(stride_bytes) * static_cast<size_t>(height);

  g_mutex_lock(&self->mutex);

  if (!ensure_capacity(self, needed)) {
    g_mutex_unlock(&self->mutex);
    return false;
  }

  const int front = g_atomic_int_get(&self->front_index);
  const int back = 1 - front;

  self->write_index = back;
  self->write_width = width;
  self->write_height = height;
  self->write_stride_bytes = stride_bytes;

  *out_ptr = self->buffers[back];

  g_mutex_unlock(&self->mutex);
  return true;
}

void nesium_texture_end_write(NesiumTexture* texture) {
  if (texture == nullptr) return;
  auto* self = NESIUM_TEXTURE(texture);

  g_mutex_lock(&self->mutex);

  // Publish the back buffer as the new front buffer.
  g_atomic_int_set(&self->front_index, self->write_index);
  self->width = self->write_width;
  self->height = self->write_height;
  self->stride_bytes = self->write_stride_bytes;

  g_mutex_unlock(&self->mutex);
}
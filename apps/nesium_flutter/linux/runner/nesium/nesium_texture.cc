#include "nesium_texture.h"

#include <stddef.h>

#include <glib.h>

namespace {
// A permanent fallback pixel used before the first real frame is published.
constexpr uint8_t kFallbackPixelRGBA[4] = {0, 0, 0, 0};
} // namespace

// CPU-backed double-buffered texture.
// - Flutter pulls pixels via `copy_pixels()` on the engine thread.
// - A background worker writes into the back buffer via begin/end write.
// - Publishing swaps the front buffer for the next engine pull.

// IMPORTANT:
// This struct MUST be defined in the global namespace because the type is
// forward-declared by G_DECLARE_FINAL_TYPE in the header.
struct _NesiumTexture {
  FlPixelBufferTexture parent_instance;

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

  // Write-in-progress state (back buffer).
  gboolean write_active = FALSE;
  gint write_index = 1;
  uint32_t write_width = 0;
  uint32_t write_height = 0;

  // Retired buffers kept until finalize to avoid use-after-free by the engine.
  GSList *retired_buffers = nullptr;
};

G_DEFINE_TYPE(NesiumTexture, nesium_texture, fl_pixel_buffer_texture_get_type())

static gboolean nesium_texture_copy_pixels(FlPixelBufferTexture *texture,
                                           const uint8_t **out_buffer,
                                           uint32_t *width, uint32_t *height,
                                           GError ** /*error*/) {
  auto *self = NESIUM_TEXTURE(texture);

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

  // The Flutter pixel-buffer texture callback does not provide a stride output.
  // The engine assumes tightly-packed RGBA: stride == width * 4.
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

static void nesium_texture_dispose(GObject *object) {
  // `dispose()` is intended for releasing references to other GObjects.
  // This texture owns raw memory that may be accessed by the engine while
  // rendering, so memory is released in `finalize()` instead.
  G_OBJECT_CLASS(nesium_texture_parent_class)->dispose(object);
}

static void nesium_texture_finalize(GObject *object) {
  auto *self = NESIUM_TEXTURE(object);

  g_mutex_lock(&self->mutex);

  if (self->buffers[0] != nullptr) {
    g_free(self->buffers[0]);
    self->buffers[0] = nullptr;
  }
  if (self->buffers[1] != nullptr) {
    g_free(self->buffers[1]);
    self->buffers[1] = nullptr;
  }

  if (self->retired_buffers != nullptr) {
    g_slist_free_full(self->retired_buffers, g_free);
    self->retired_buffers = nullptr;
  }

  self->buffer_capacity = 0;
  self->front_index = 0;
  self->has_frame = FALSE;
  self->width = 0;
  self->height = 0;
  self->write_active = FALSE;

  g_mutex_unlock(&self->mutex);

  g_mutex_clear(&self->mutex);

  G_OBJECT_CLASS(nesium_texture_parent_class)->finalize(object);
}

static void nesium_texture_class_init(NesiumTextureClass *klass) {
  auto *gobject_class = G_OBJECT_CLASS(klass);
  gobject_class->dispose = nesium_texture_dispose;
  gobject_class->finalize = nesium_texture_finalize;

  auto *pixel_texture_class = FL_PIXEL_BUFFER_TEXTURE_CLASS(klass);
  pixel_texture_class->copy_pixels = nesium_texture_copy_pixels;
}

static void nesium_texture_init(NesiumTexture *texture) {
  auto *self = NESIUM_TEXTURE(texture);
  g_mutex_init(&self->mutex);

  self->front_index = 0;
  self->has_frame = FALSE;
  self->write_index = 1;
  self->write_active = FALSE;
  self->retired_buffers = nullptr;
}

static bool ensure_capacity(NesiumTexture *self, size_t needed_bytes) {
  if (needed_bytes == 0)
    return false;

  // If we already have enough space, do nothing.
  if (self->buffer_capacity >= needed_bytes && self->buffers[0] != nullptr &&
      self->buffers[1] != nullptr) {
    return true;
  }

  // Growth needed. Allocate new buffers.
  uint8_t *b0 = static_cast<uint8_t *>(g_malloc(needed_bytes));
  uint8_t *b1 = static_cast<uint8_t *>(g_malloc(needed_bytes));
  if (b0 == nullptr || b1 == nullptr) {
    if (b0 != nullptr)
      g_free(b0);
    if (b1 != nullptr)
      g_free(b1);
    return false;
  }

  // If we had old buffers, push them to the retired list.
  // We keep only the single most recent set of retired buffers to prevent
  // memory growth over long sessions. Two generations of buffers (current +
  // previous) is plenty for engine safety.
  if (self->retired_buffers != nullptr) {
    g_slist_free_full(self->retired_buffers, g_free);
    self->retired_buffers = nullptr;
  }

  if (self->buffers[0] != nullptr) {
    self->retired_buffers =
        g_slist_prepend(self->retired_buffers, self->buffers[0]);
  }
  if (self->buffers[1] != nullptr) {
    self->retired_buffers =
        g_slist_prepend(self->retired_buffers, self->buffers[1]);
  }

  self->buffers[0] = b0;
  self->buffers[1] = b1;
  self->buffer_capacity = needed_bytes;
  return true;
}

NesiumTexture *nesium_texture_new() {
  return NESIUM_TEXTURE(g_object_new(NESIUM_TYPE_TEXTURE, nullptr));
}

bool nesium_texture_begin_write(NesiumTexture *texture, uint32_t width,
                                uint32_t height, uint32_t stride_bytes,
                                uint8_t **out_ptr) {
  if (texture == nullptr || out_ptr == nullptr)
    return false;
  if (width == 0 || height == 0)
    return false;

  // The engine expects tightly-packed RGBA.
  if (stride_bytes != width * 4u)
    return false;

  auto *self = NESIUM_TEXTURE(texture);

  const size_t needed =
      static_cast<size_t>(stride_bytes) * static_cast<size_t>(height);

  g_mutex_lock(&self->mutex);

  if (self->write_active) {
    g_mutex_unlock(&self->mutex);
    return false;
  }

  if (!ensure_capacity(self, needed)) {
    g_mutex_unlock(&self->mutex);
    return false;
  }

  const int front = self->front_index;
  const int back = 1 - front;

  // Expose the back buffer to the caller for writing.
  self->write_active = TRUE;
  self->write_index = back;
  self->write_width = width;
  self->write_height = height;

  *out_ptr = self->buffers[back];

  g_mutex_unlock(&self->mutex);
  return true;
}

void nesium_texture_end_write(NesiumTexture *texture) {
  if (texture == nullptr)
    return;
  auto *self = NESIUM_TEXTURE(texture);

  g_mutex_lock(&self->mutex);

  if (!self->write_active) {
    g_mutex_unlock(&self->mutex);
    return;
  }

  // Publish the back buffer as the new front buffer for the engine thread.
  self->front_index = self->write_index;
  self->width = self->write_width;
  self->height = self->write_height;
  self->has_frame = TRUE;
  self->write_active = FALSE;

  g_mutex_unlock(&self->mutex);
}

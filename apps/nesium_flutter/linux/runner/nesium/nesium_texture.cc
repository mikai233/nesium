#include "nesium_texture.h"

#include <epoxy/gl.h>
#include <glib.h>
#include <stddef.h>

extern "C" bool
nesium_linux_apply_shader(uint32_t input_tex, uint32_t output_tex,
                          uint32_t src_width, uint32_t src_height,
                          uint32_t dst_width, uint32_t dst_height,
                          uint64_t frame_count);

namespace {
constexpr uint8_t kFallbackPixelRGBA[4] = {0, 0, 0, 0};
} // namespace

struct _NesiumTexture {
  FlTextureGL parent_instance;

  GMutex mutex;

  uint8_t *buffers[2] = {nullptr, nullptr};
  size_t buffer_capacity = 0;
  gint front_index = 0;

  gboolean has_frame = FALSE;
  uint32_t width = 0;
  uint32_t height = 0;

  gboolean write_active = FALSE;
  gint write_index = 1;
  uint32_t write_width = 0;
  uint32_t write_height = 0;

  uint32_t source_tex = 0;
  uint32_t target_tex = 0;
  uint32_t last_upload_width = 0;
  uint32_t last_upload_height = 0;
  uint64_t frame_count = 0;

  // Kept until finalize to avoid use-after-free by the engine.
  GSList *retired_buffers = nullptr;
};

G_DEFINE_TYPE(NesiumTexture, nesium_texture, fl_texture_gl_get_type())

static gboolean
nesium_texture_gl_populate_texture(FlTextureGL *texture, uint32_t *target,
                                   uint32_t *name, uint32_t *width,
                                   uint32_t *height, GError **error) {
  auto *self = NESIUM_TEXTURE(texture);

  g_mutex_lock(&self->mutex);

  if (self->width == 0 || self->height == 0) {
    g_mutex_unlock(&self->mutex);
    return FALSE;
  }

  // 1. Ensure textures exist.
  if (self->source_tex == 0) {
    glGenTextures(1, &self->source_tex);
    glBindTexture(GL_TEXTURE_2D, self->source_tex);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
  }

  if (self->target_tex == 0) {
    glGenTextures(1, &self->target_tex);
    glBindTexture(GL_TEXTURE_2D, self->target_tex);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
  }

  // 2. Upload CPU buffer to GPU if changed.
  if (self->has_frame) {
    glBindTexture(GL_TEXTURE_2D, self->source_tex);
    if (self->width != self->last_upload_width ||
        self->height != self->last_upload_height) {
      glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA8, self->width, self->height, 0,
                   GL_RGBA, GL_UNSIGNED_BYTE, self->buffers[self->front_index]);
      self->last_upload_width = self->width;
      self->last_upload_height = self->height;
    } else {
      glTexSubImage2D(GL_TEXTURE_2D, 0, 0, 0, self->width, self->height,
                      GL_RGBA, GL_UNSIGNED_BYTE,
                      self->buffers[self->front_index]);
    }
    self->has_frame = FALSE;
  }

  // 3. Apply shader.
  // Output size currently matches input size.
  uint32_t out_w = self->width;
  uint32_t out_h = self->height;

  if (nesium_linux_apply_shader(self->source_tex, self->target_tex, self->width,
                                self->height, out_w, out_h,
                                self->frame_count)) {
    *target = GL_TEXTURE_2D;
    *name = self->target_tex;
    *width = out_w;
    *height = out_h;
    self->frame_count++;
  } else {
    *target = GL_TEXTURE_2D;
    *name = self->source_tex;
    *width = self->width;
    *height = self->height;
  }

  g_mutex_unlock(&self->mutex);
  return TRUE;
}

static void nesium_texture_dispose(GObject *object) {
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

  // Note: texture deletion depends on a current GL context, which may not exist
  // here. We leave them for process exit or context destruction.

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

  auto *gl_texture_class = FL_TEXTURE_GL_CLASS(klass);
  gl_texture_class->populate = nesium_texture_gl_populate_texture;
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

  if (self->buffer_capacity >= needed_bytes && self->buffers[0] != nullptr &&
      self->buffers[1] != nullptr) {
    return true;
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

  const int back = 1 - self->front_index;

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

  self->front_index = self->write_index;
  self->width = self->write_width;
  self->height = self->write_height;
  self->has_frame = TRUE;
  self->write_active = FALSE;

  g_mutex_unlock(&self->mutex);
}

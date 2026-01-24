#include "nesium_channels.h"

#include <flutter_linux/flutter_linux.h>

#include <atomic>
#include <condition_variable>
#include <cstdint>
#include <mutex>
#include <optional>
#include <thread>

#include "nesium_texture.h"

static constexpr const char *kChannelName = "nesium";
static constexpr const char *kMethodCreate = "createNesTexture";
static constexpr const char *kMethodDispose = "disposeNesTexture";
static constexpr const char *kMethodSetPresentBufferSize = "setPresentBufferSize";

// Texture upload pipeline (Linux):
// 1) Rust runtime emits a frame-ready callback from its render thread.
// 2) We coalesce callbacks and wake a dedicated copy worker.
// 3) The copy worker blits the latest frame into a double-buffered RGBA
// texture. 4) We schedule a GTK main-thread notify to present the new frame.

// ---- Rust FFI (linked at build time) ----
//
// The Linux runner links against libnesium_flutter.so, so we can call the
// exported C ABI functions directly. If the symbols are missing, the build will
// fail at link time instead of failing at runtime.
extern "C" {
void nesium_runtime_start();

using FrameReadyCallback = void (*)(uint32_t buffer_index, uint32_t width,
                                    uint32_t height, uint32_t pitch_bytes,
                                    void *user_data);

void nesium_set_frame_ready_callback(FrameReadyCallback cb, void *user_data);

void nesium_copy_frame(uint32_t buffer_index, uint8_t *dst_rgba,
                       uint32_t dst_pitch_bytes, uint32_t dst_height);
}

struct PendingFrame {
  uint32_t buffer_index;
  uint32_t width;
  uint32_t height;
  uint32_t pitch_bytes;
};

struct _NesiumChannels {
  FlMethodChannel *channel = nullptr;
  FlTextureRegistrar *registrar = nullptr;

  FlTexture *texture = nullptr;
  int64_t texture_id = -1;

  bool runtime_started = false;

  // Copy worker thread. The Rust callback only posts the latest frame metadata.
  std::thread copy_thread;
  std::mutex mu;
  std::condition_variable cv;
  bool stop = false;
  std::optional<PendingFrame> pending;

  // Coalesce notifications to the GTK main thread.
  std::atomic<bool> notify_scheduled{false};

  // Keep the instance alive while async GTK callbacks are in flight.
  std::atomic<int> ref_count{1};
  std::atomic<bool> shutting_down{false};
};

static void nesium_channels_ref(NesiumChannels *self) {
  self->ref_count.fetch_add(1, std::memory_order_relaxed);
}

static void nesium_channels_unref(NesiumChannels *self) {
  if (self->ref_count.fetch_sub(1, std::memory_order_acq_rel) == 1) {
    delete self;
  }
}

static FlMethodResponse *make_error(const char *code, const char *message) {
  g_autoptr(FlValue) details = fl_value_new_null();
  return FL_METHOD_RESPONSE(
      fl_method_error_response_new(code, message, details));
}

static FlMethodResponse *make_ok_with_int64(int64_t value) {
  g_autoptr(FlValue) result = fl_value_new_int(value);
  return FL_METHOD_RESPONSE(fl_method_success_response_new(result));
}

static gboolean notify_on_main(gpointer user_data) {
  auto *self = static_cast<NesiumChannels *>(user_data);
  self->notify_scheduled.store(false, std::memory_order_release);

  if (self->shutting_down.load(std::memory_order_acquire)) {
    return G_SOURCE_REMOVE;
  }

  if (self->registrar != nullptr && self->texture != nullptr) {
    fl_texture_registrar_mark_texture_frame_available(self->registrar,
                                                      self->texture);
  }

  return G_SOURCE_REMOVE;
}

static void notify_on_main_destroy(gpointer user_data) {
  auto *self = static_cast<NesiumChannels *>(user_data);
  nesium_channels_unref(self);
}

static void schedule_notify(NesiumChannels *self) {
  if (self->shutting_down.load(std::memory_order_acquire)) {
    return;
  }

  bool expected = false;
  if (!self->notify_scheduled.compare_exchange_strong(
          expected, true, std::memory_order_acq_rel)) {
    return;
  }

  // Run on the GTK main loop, keeping the instance alive until callback runs.
  nesium_channels_ref(self);
  g_main_context_invoke_full(nullptr, G_PRIORITY_DEFAULT, notify_on_main, self,
                             notify_on_main_destroy);
}

static void copy_worker_main(NesiumChannels *self) {
  for (;;) {
    PendingFrame f{};

    {
      std::unique_lock<std::mutex> lk(self->mu);
      self->cv.wait(lk,
                    [&] { return self->stop || self->pending.has_value(); });
      if (self->stop) {
        return;
      }

      // Coalesce: always process the latest pending frame.
      f = *self->pending;
      self->pending.reset();
    }

    if (self->texture == nullptr ||
        self->shutting_down.load(std::memory_order_acquire)) {
      continue;
    }

    auto *tex = NESIUM_TEXTURE(self->texture);

    // Flutter's pixel buffer texture expects tightly-packed RGBA.
    const uint32_t out_w = f.width;
    const uint32_t out_h = f.height;
    const uint32_t dst_stride = out_w * 4u;

    uint8_t *dst = nullptr;
    if (!nesium_texture_begin_write(tex, out_w, out_h, dst_stride, &dst)) {
      continue;
    }

    // Copy the current Rust frame into the writable back buffer.
    nesium_copy_frame(f.buffer_index, dst, dst_stride, out_h);

    // Publish and request a redraw.
    nesium_texture_end_write(tex);
    schedule_notify(self);
  }
}

static void frame_ready_cb(uint32_t buffer_index, uint32_t width,
                           uint32_t height, uint32_t pitch_bytes,
                           void *user_data) {
  auto *self = static_cast<NesiumChannels *>(user_data);
  if (self->shutting_down.load(std::memory_order_acquire)) {
    return;
  }

  // Keep the callback lightweight: overwrite the latest pending frame and wake
  // the copy worker.
  {
    std::lock_guard<std::mutex> lk(self->mu);
    self->pending = PendingFrame{buffer_index, width, height, pitch_bytes};
  }

  self->cv.notify_one();
}

static void ensure_copy_worker(NesiumChannels *self) {
  if (self->copy_thread.joinable()) {
    return;
  }

  self->stop = false;
  self->copy_thread = std::thread([self] { copy_worker_main(self); });
}

static void stop_copy_worker(NesiumChannels *self) {
  {
    std::lock_guard<std::mutex> lk(self->mu);
    self->stop = true;
    self->pending.reset();
  }
  self->cv.notify_one();

  if (self->copy_thread.joinable()) {
    self->copy_thread.join();
  }
}

static void handle_create_texture(NesiumChannels *self, FlMethodCall *call) {
  if (self->registrar == nullptr) {
    fl_method_call_respond(
        call, make_error("no_registrar", "Texture registrar is not available"),
        nullptr);
    return;
  }

  // Reuse existing texture if already registered.
  if (self->texture != nullptr && self->texture_id >= 0) {
    fl_method_call_respond(call, make_ok_with_int64(self->texture_id), nullptr);
    return;
  }

  FlTexture *texture = FL_TEXTURE(nesium_texture_new());
  if (texture == nullptr) {
    fl_method_call_respond(
        call, make_error("texture_create_failed", "Failed to create texture"),
        nullptr);
    return;
  }

  if (!fl_texture_registrar_register_texture(self->registrar, texture)) {
    g_object_unref(texture);
    fl_method_call_respond(
        call,
        make_error("texture_register_failed", "Failed to register texture"),
        nullptr);
    return;
  }

  self->texture = texture;
  self->texture_id = fl_texture_get_id(texture);

  // Start the copy worker and hook the Rust callback.
  ensure_copy_worker(self);

  if (!self->runtime_started) {
    nesium_runtime_start();
    self->runtime_started = true;
  }

  nesium_set_frame_ready_callback(frame_ready_cb, self);

  fl_method_call_respond(call, make_ok_with_int64(self->texture_id), nullptr);
}

static void handle_set_present_buffer_size(NesiumChannels *self,
                                          FlMethodCall *call) {
  (void)self;
  FlValue *args = fl_method_call_get_args(call);
  if (fl_value_get_type(args) != FL_VALUE_TYPE_MAP) {
    fl_method_call_respond(call, make_error("BAD_ARGS", "Missing arguments"),
                           nullptr);
    return;
  }

  FlValue *width_value = fl_value_lookup_string(args, "width");
  FlValue *height_value = fl_value_lookup_string(args, "height");
  if (width_value == nullptr || height_value == nullptr) {
    fl_method_call_respond(
        call, make_error("BAD_ARGS", "Missing width/height"), nullptr);
    return;
  }

  const uint32_t width = static_cast<uint32_t>(fl_value_get_int(width_value));
  const uint32_t height = static_cast<uint32_t>(fl_value_get_int(height_value));
  if (width == 0 || height == 0) {
    fl_method_call_respond(call,
                           make_error("BAD_ARGS", "width/height must be > 0"),
                           nullptr);
    return;
  }

  g_autoptr(FlValue) result = fl_value_new_null();
  fl_method_call_respond(
      call, FL_METHOD_RESPONSE(fl_method_success_response_new(result)), nullptr);
}

static void handle_dispose_texture(NesiumChannels *self, FlMethodCall *call) {
  // Unhook the Rust callback.
  nesium_set_frame_ready_callback(nullptr, nullptr);

  stop_copy_worker(self);

  if (self->registrar != nullptr && self->texture != nullptr) {
    fl_texture_registrar_unregister_texture(self->registrar, self->texture);
    g_object_unref(self->texture);
  }

  self->texture = nullptr;
  self->texture_id = -1;

  g_autoptr(FlValue) result = fl_value_new_null();
  fl_method_call_respond(
      call, FL_METHOD_RESPONSE(fl_method_success_response_new(result)),
      nullptr);
}

static void method_call_cb(FlMethodChannel * /*channel*/, FlMethodCall *call,
                           gpointer user_data) {
  auto *self = static_cast<NesiumChannels *>(user_data);
  const gchar *name = fl_method_call_get_name(call);

  if (g_strcmp0(name, kMethodCreate) == 0) {
    handle_create_texture(self, call);
    return;
  }

  if (g_strcmp0(name, kMethodSetPresentBufferSize) == 0) {
    handle_set_present_buffer_size(self, call);
    return;
  }

  if (g_strcmp0(name, kMethodDispose) == 0) {
    handle_dispose_texture(self, call);
    return;
  }

  fl_method_call_respond(
      call, FL_METHOD_RESPONSE(fl_method_not_implemented_response_new()),
      nullptr);
}

NesiumChannels *nesium_channels_new(FlView *view) {
  if (view == nullptr)
    return nullptr;

  FlEngine *engine = fl_view_get_engine(view);
  if (engine == nullptr)
    return nullptr;

  auto *self = new _NesiumChannels();

  self->registrar = fl_engine_get_texture_registrar(engine);
  if (self->registrar != nullptr) {
    g_object_ref(self->registrar);
  }

  FlBinaryMessenger *messenger = fl_engine_get_binary_messenger(engine);
  g_autoptr(FlStandardMethodCodec) codec = fl_standard_method_codec_new();
  self->channel =
      fl_method_channel_new(messenger, kChannelName, FL_METHOD_CODEC(codec));

  fl_method_channel_set_method_call_handler(self->channel, method_call_cb, self,
                                            nullptr);

  return self;
}

void nesium_channels_free(NesiumChannels *self) {
  if (self == nullptr)
    return;

  self->shutting_down.store(true, std::memory_order_release);

  // Unhook callback and stop worker first.
  nesium_set_frame_ready_callback(nullptr, nullptr);

  stop_copy_worker(self);

  if (self->registrar != nullptr && self->texture != nullptr) {
    fl_texture_registrar_unregister_texture(self->registrar, self->texture);
    g_object_unref(self->texture);
  }

  self->texture = nullptr;
  self->texture_id = -1;

  if (self->channel != nullptr) {
    g_object_unref(self->channel);
    self->channel = nullptr;
  }

  if (self->registrar != nullptr) {
    g_object_unref(self->registrar);
    self->registrar = nullptr;
  }

  nesium_channels_unref(self);
}

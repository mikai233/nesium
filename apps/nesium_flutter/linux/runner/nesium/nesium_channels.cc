#include "nesium_channels.h"

#include <flutter_linux/flutter_linux.h>

#include <dlfcn.h>

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

// ---- Rust FFI (resolved via dlsym at runtime) ----
//
// The Rust shared object is expected to be loaded by Dart FFI (dlopen).
// On Linux, Dart may load the .so with RTLD_LOCAL, which means symbols are not
// visible via RTLD_DEFAULT. We therefore fall back to dlopen(RTLD_GLOBAL) +
// dlsym(handle, ...) to resolve the symbols reliably.

using FrameReadyCallback = void (*)(uint32_t buffer_index, uint32_t width,
                                    uint32_t height, uint32_t pitch_bytes,
                                    void *user_data);

using NesiumRuntimeStartFn = void (*)();
using NesiumSetFrameReadyCallbackFn = void (*)(FrameReadyCallback cb,
                                               void *user_data);
using NesiumCopyFrameFn = void (*)(uint32_t buffer_index, uint8_t *dst_rgba,
                                   uint32_t dst_pitch_bytes,
                                   uint32_t dst_height);

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

  // Handle to the Rust shared object (used for dlsym fallback on Linux).
  void *rust_handle = nullptr;

  // Rust API function pointers.
  NesiumRuntimeStartFn runtime_start = nullptr;
  NesiumSetFrameReadyCallbackFn set_frame_ready_cb = nullptr;
  NesiumCopyFrameFn copy_frame = nullptr;

  bool runtime_started = false;

  // Copy worker.
  std::thread copy_thread;
  std::mutex mu;
  std::condition_variable cv;
  bool stop = false;
  std::optional<PendingFrame> pending;

  // Coalesce notifications to the GTK main thread.
  std::atomic<bool> notify_scheduled{false};
};

static FlMethodResponse *make_error(const char *code, const char *message) {
  g_autoptr(FlValue) details = fl_value_new_null();
  return FL_METHOD_RESPONSE(
      fl_method_error_response_new(code, message, details));
}

static FlMethodResponse *make_ok_with_int64(int64_t value) {
  g_autoptr(FlValue) result = fl_value_new_int(value);
  return FL_METHOD_RESPONSE(fl_method_success_response_new(result));
}

static void *resolve_sym(NesiumChannels *self, const char *name) {
  // First try the global namespace.
  // This works if the Rust .so was loaded with RTLD_GLOBAL.
  (void)dlerror();
  void *sym = dlsym(RTLD_DEFAULT, name);
  if (sym != nullptr) {
    return sym;
  }

  // Fallback: ensure the Rust .so is globally visible and resolve via its
  // handle. This covers the common case where Dart FFI loaded the library with
  // RTLD_LOCAL.
  if (self->rust_handle == nullptr) {
    // Best-effort: load by soname. The runner binary usually has an rpath that
    // includes $ORIGIN/lib where Flutter bundles native libraries.
    self->rust_handle = dlopen("libnesium_flutter.so", RTLD_NOW | RTLD_GLOBAL);
    if (self->rust_handle == nullptr) {
      // Keep the error for diagnostics.
      const char *err = dlerror();
      if (err != nullptr) {
        g_warning("dlopen(libnesium_flutter.so) failed: %s", err);
      }
      return nullptr;
    }
  }

  (void)dlerror();
  sym = dlsym(self->rust_handle, name);
  if (sym == nullptr) {
    const char *err = dlerror();
    if (err != nullptr) {
      g_warning("dlsym(%s) failed: %s", name, err);
    }
  }
  return sym;
}

static bool resolve_rust_api(NesiumChannels *self) {
  if (self->set_frame_ready_cb != nullptr && self->copy_frame != nullptr) {
    return true;
  }

  self->runtime_start = reinterpret_cast<NesiumRuntimeStartFn>(
      resolve_sym(self, "nesium_runtime_start"));
  self->set_frame_ready_cb = reinterpret_cast<NesiumSetFrameReadyCallbackFn>(
      resolve_sym(self, "nesium_set_frame_ready_callback"));
  self->copy_frame = reinterpret_cast<NesiumCopyFrameFn>(
      resolve_sym(self, "nesium_copy_frame"));

  return self->set_frame_ready_cb != nullptr && self->copy_frame != nullptr;
}

static gboolean notify_on_main(gpointer user_data) {
  auto *self = static_cast<NesiumChannels *>(user_data);
  self->notify_scheduled.store(false, std::memory_order_release);

  if (self->registrar != nullptr && self->texture != nullptr) {
    fl_texture_registrar_mark_texture_frame_available(self->registrar,
                                                      self->texture);
  }

  return G_SOURCE_REMOVE;
}

static void schedule_notify(NesiumChannels *self) {
  bool expected = false;
  if (!self->notify_scheduled.compare_exchange_strong(
          expected, true, std::memory_order_acq_rel)) {
    return;
  }

  // Run on the GTK main loop.
  g_main_context_invoke(nullptr, notify_on_main, self);
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

    if (self->texture == nullptr || self->copy_frame == nullptr) {
      continue;
    }

    auto *tex = NESIUM_TEXTURE(self->texture);

    // Force tightly-packed RGBA for Flutter's pixel buffer texture.
    const uint32_t dst_stride = f.width * 4u;

    uint8_t *dst = nullptr;
    if (!nesium_texture_begin_write(tex, f.width, f.height, dst_stride, &dst)) {
      continue;
    }

    // Copy the current Rust frame into the writable back buffer.
    self->copy_frame(f.buffer_index, dst, dst_stride, f.height);

    // Publish and request a redraw.
    nesium_texture_end_write(tex);
    schedule_notify(self);
  }
}

static void frame_ready_cb(uint32_t buffer_index, uint32_t width,
                           uint32_t height, uint32_t pitch_bytes,
                           void *user_data) {
  auto *self = static_cast<NesiumChannels *>(user_data);

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

  if (!resolve_rust_api(self)) {
    fl_method_call_respond(
        call,
        make_error("rust_api_unavailable",
                   "Rust symbols not found. Make sure the Rust shared library "
                   "is loaded before calling createNesTexture."),
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

  if (!self->runtime_started && self->runtime_start != nullptr) {
    self->runtime_start();
    self->runtime_started = true;
  }

  self->set_frame_ready_cb(frame_ready_cb, self);

  fl_method_call_respond(call, make_ok_with_int64(self->texture_id), nullptr);
}

static void handle_dispose_texture(NesiumChannels *self, FlMethodCall *call) {
  // Unhook the Rust callback (best-effort).
  if (self->set_frame_ready_cb != nullptr) {
    self->set_frame_ready_cb(nullptr, nullptr);
  }

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

  // Unhook callback and stop worker first.
  if (self->set_frame_ready_cb != nullptr) {
    self->set_frame_ready_cb(nullptr, nullptr);
  }

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

  if (self->rust_handle != nullptr) {
    dlclose(self->rust_handle);
    self->rust_handle = nullptr;
  }

  delete self;
}

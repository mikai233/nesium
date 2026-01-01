#include "nesium_aux_channels.h"

#include <flutter_linux/flutter_linux.h>

#include <atomic>
#include <map>
#include <memory>
#include <set>
#include <thread>

#include "nesium_aux_texture.h"

static constexpr const char *kChannelName = "nesium_aux";
static constexpr const char *kMethodCreate = "createAuxTexture";
static constexpr const char *kMethodDispose = "disposeAuxTexture";
static constexpr const char *kMethodPause = "pauseAuxTexture";

struct TextureEntry {
  FlTexture *texture = nullptr;
  int64_t texture_id = -1;
};

struct _NesiumAuxChannels {
  FlMethodChannel *channel = nullptr;
  FlTextureRegistrar *registrar = nullptr;

  // Map from aux texture ID to Flutter texture.
  std::map<uint32_t, TextureEntry> textures;

  // Set of paused texture IDs.
  std::set<uint32_t> paused_ids;

  // Update thread: periodically updates all textures from Rust buffers.
  std::thread update_thread;
  std::atomic<bool> stop{false};
  std::atomic<bool> shutting_down{false};
};

static void nesium_aux_channels_unref(NesiumAuxChannels *self) { delete self; }

static FlMethodResponse *make_error(const char *code, const char *message) {
  g_autoptr(FlValue) details = fl_value_new_null();
  return FL_METHOD_RESPONSE(
      fl_method_error_response_new(code, message, details));
}

static FlMethodResponse *make_ok_with_int64(int64_t value) {
  g_autoptr(FlValue) result = fl_value_new_int(value);
  return FL_METHOD_RESPONSE(fl_method_success_response_new(result));
}

static FlMethodResponse *make_ok_null() {
  g_autoptr(FlValue) result = fl_value_new_null();
  return FL_METHOD_RESPONSE(fl_method_success_response_new(result));
}

static void update_worker_main(NesiumAuxChannels *self) {
  // Update at ~60Hz.
  const int sleep_ms = 16;

  while (!self->stop.load(std::memory_order_acquire)) {
    // Update all textures from Rust buffers.
    for (auto &[id, entry] : self->textures) {
      if (entry.texture == nullptr)
        continue;

      // Skip paused textures.
      if (self->paused_ids.count(id) > 0)
        continue;

      auto *tex = NESIUM_AUX_TEXTURE(entry.texture);
      nesium_aux_texture_update_from_rust(tex);

      // Notify Flutter that the texture has a new frame.
      if (self->registrar != nullptr &&
          !self->shutting_down.load(std::memory_order_acquire)) {
        fl_texture_registrar_mark_texture_frame_available(self->registrar,
                                                          entry.texture);
      }
    }

    g_usleep(sleep_ms * 1000); // Convert to microseconds
  }
}

static void handle_create_aux_texture(NesiumAuxChannels *self,
                                      FlMethodCall *call) {
  if (self->registrar == nullptr) {
    fl_method_call_respond(
        call, make_error("no_registrar", "Texture registrar is not available"),
        nullptr);
    return;
  }

  FlValue *args = fl_method_call_get_args(call);
  if (fl_value_get_type(args) != FL_VALUE_TYPE_MAP) {
    fl_method_call_respond(call, make_error("BAD_ARGS", "Missing arguments"),
                           nullptr);
    return;
  }

  FlValue *id_value = fl_value_lookup_string(args, "id");
  FlValue *width_value = fl_value_lookup_string(args, "width");
  FlValue *height_value = fl_value_lookup_string(args, "height");

  if (id_value == nullptr || width_value == nullptr ||
      height_value == nullptr) {
    fl_method_call_respond(
        call, make_error("BAD_ARGS", "Missing id/width/height"), nullptr);
    return;
  }

  const uint32_t id = static_cast<uint32_t>(fl_value_get_int(id_value));
  const uint32_t width = static_cast<uint32_t>(fl_value_get_int(width_value));
  const uint32_t height = static_cast<uint32_t>(fl_value_get_int(height_value));

  // Clean up existing texture with this ID.
  auto it = self->textures.find(id);
  if (it != self->textures.end()) {
    if (it->second.texture != nullptr) {
      fl_texture_registrar_unregister_texture(self->registrar,
                                              it->second.texture);
      g_object_unref(it->second.texture);
    }
    self->textures.erase(it);
  }

  // Create new texture.
  FlTexture *texture = FL_TEXTURE(nesium_aux_texture_new(id, width, height));
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

  const int64_t texture_id = fl_texture_get_id(texture);
  self->textures[id] = TextureEntry{texture, texture_id};

  // Start update thread if this is the first texture.
  if (self->textures.size() == 1 && !self->update_thread.joinable()) {
    self->stop.store(false, std::memory_order_release);
    self->update_thread = std::thread([self] { update_worker_main(self); });
  }

  fl_method_call_respond(call, make_ok_with_int64(texture_id), nullptr);
}

static void handle_dispose_aux_texture(NesiumAuxChannels *self,
                                       FlMethodCall *call) {
  FlValue *args = fl_method_call_get_args(call);
  if (fl_value_get_type(args) != FL_VALUE_TYPE_MAP) {
    fl_method_call_respond(call, make_error("BAD_ARGS", "Missing arguments"),
                           nullptr);
    return;
  }

  FlValue *id_value = fl_value_lookup_string(args, "id");
  if (id_value == nullptr) {
    fl_method_call_respond(call, make_error("BAD_ARGS", "Missing id"), nullptr);
    return;
  }

  const uint32_t id = static_cast<uint32_t>(fl_value_get_int(id_value));

  auto it = self->textures.find(id);
  if (it != self->textures.end()) {
    if (self->registrar != nullptr && it->second.texture != nullptr) {
      fl_texture_registrar_unregister_texture(self->registrar,
                                              it->second.texture);
      g_object_unref(it->second.texture);
    }
    self->textures.erase(it);
  }
  self->paused_ids.erase(id);

  // Stop update thread if no textures remain.
  if (self->textures.empty() && self->update_thread.joinable()) {
    self->stop.store(true, std::memory_order_release);
    self->update_thread.join();
  }

  fl_method_call_respond(call, make_ok_null(), nullptr);
}

static void handle_pause_aux_texture(NesiumAuxChannels *self,
                                     FlMethodCall *call) {
  FlValue *args = fl_method_call_get_args(call);
  if (fl_value_get_type(args) != FL_VALUE_TYPE_MAP) {
    fl_method_call_respond(call, make_error("BAD_ARGS", "Missing arguments"),
                           nullptr);
    return;
  }

  FlValue *id_value = fl_value_lookup_string(args, "id");
  if (id_value == nullptr) {
    fl_method_call_respond(call, make_error("BAD_ARGS", "Missing id"), nullptr);
    return;
  }

  const uint32_t id = static_cast<uint32_t>(fl_value_get_int(id_value));
  self->paused_ids.insert(id);

  fl_method_call_respond(call, make_ok_null(), nullptr);
}

static void method_call_cb(FlMethodChannel * /*channel*/, FlMethodCall *call,
                           gpointer user_data) {
  auto *self = static_cast<NesiumAuxChannels *>(user_data);
  const gchar *name = fl_method_call_get_name(call);

  if (g_strcmp0(name, kMethodCreate) == 0) {
    handle_create_aux_texture(self, call);
    return;
  }

  if (g_strcmp0(name, kMethodDispose) == 0) {
    handle_dispose_aux_texture(self, call);
    return;
  }

  if (g_strcmp0(name, kMethodPause) == 0) {
    handle_pause_aux_texture(self, call);
    return;
  }

  fl_method_call_respond(
      call, FL_METHOD_RESPONSE(fl_method_not_implemented_response_new()),
      nullptr);
}

NesiumAuxChannels *nesium_aux_channels_new(FlView *view) {
  if (view == nullptr)
    return nullptr;

  FlEngine *engine = fl_view_get_engine(view);
  if (engine == nullptr)
    return nullptr;

  auto *self = new _NesiumAuxChannels();

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

void nesium_aux_channels_free(NesiumAuxChannels *self) {
  if (self == nullptr)
    return;

  self->shutting_down.store(true, std::memory_order_release);

  // Stop update thread.
  if (self->update_thread.joinable()) {
    self->stop.store(true, std::memory_order_release);
    self->update_thread.join();
  }

  // Unregister all textures.
  for (auto &[id, entry] : self->textures) {
    if (self->registrar != nullptr && entry.texture != nullptr) {
      fl_texture_registrar_unregister_texture(self->registrar, entry.texture);
      g_object_unref(entry.texture);
    }
  }
  self->textures.clear();

  if (self->channel != nullptr) {
    g_object_unref(self->channel);
    self->channel = nullptr;
  }

  if (self->registrar != nullptr) {
    g_object_unref(self->registrar);
    self->registrar = nullptr;
  }

  nesium_aux_channels_unref(self);
}

#include "nesium_channels.h"

#include <flutter_linux/flutter_linux.h>

#include "nesium_texture.h"

static constexpr const char *kChannelName = "nesium";
static constexpr const char *kMethodCreate = "createNesTexture";
static constexpr const char *kMethodDispose = "disposeNesTexture";

struct _NesiumChannels {
  FlMethodChannel *channel = nullptr;
  FlTextureRegistrar *registrar = nullptr;

  // Single texture instance for bring-up.
  FlTexture *texture = nullptr;
  int64_t texture_id = -1;
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

  fl_method_call_respond(call, make_ok_with_int64(self->texture_id), nullptr);
}

static void handle_dispose_texture(NesiumChannels *self, FlMethodCall *call) {
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

  delete self;
}

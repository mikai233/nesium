#include "nesium_aux_texture_plugin.h"

#include <atomic>
#include <cstdint>
#include <map>
#include <memory>
#include <mutex>
#include <thread>
#include <windows.h>

#include "flutter/method_channel.h"
#include "flutter/plugin_registrar.h"
#include "flutter/plugin_registrar_windows.h"
#include "flutter/standard_method_codec.h"
#include "flutter/texture_registrar.h"

#include "nesium_texture.h"

// Windows auxiliary texture plugin for debugger views (Tilemap, Pattern, etc.)
//
// This plugin creates software textures similar to the main NES texture,
// but receives data from the Rust aux_texture module instead of the NES
// emulator.
//
// The Rust module provides a double-buffered BGRA texture that we copy
// into Flutter's pixel buffer texture on demand.

namespace {

// Rust C ABI function pointers for auxiliary textures
struct RustAuxApi {
  HMODULE dll = nullptr;

  void (*aux_create)(uint32_t id, uint32_t width, uint32_t height) = nullptr;
  size_t (*aux_copy)(uint32_t id, uint8_t *dst, uint32_t dst_pitch,
                     uint32_t dst_height) = nullptr;
  void (*aux_destroy)(uint32_t id) = nullptr;

  // Bind to the already-loaded nesium_flutter module
  bool BindLoadedModule(const wchar_t *module_name) {
    dll = ::GetModuleHandleW(module_name);
    if (!dll) {
      return false;
    }

    aux_create = reinterpret_cast<decltype(aux_create)>(
        ::GetProcAddress(dll, "nesium_aux_create"));
    aux_copy = reinterpret_cast<decltype(aux_copy)>(
        ::GetProcAddress(dll, "nesium_aux_copy"));
    aux_destroy = reinterpret_cast<decltype(aux_destroy)>(
        ::GetProcAddress(dll, "nesium_aux_destroy"));

    return aux_create && aux_copy && aux_destroy;
  }
};

// Global API instance
static RustAuxApi g_aux_api;

// Represents one auxiliary texture registered with Flutter.
class AuxTextureEntry {
public:
  AuxTextureEntry(uint32_t id, uint32_t width, uint32_t height)
      : id_(id), texture_(std::make_unique<NesiumTexture>(width, height)) {
    // Create the Rust-side backing store.
    if (g_aux_api.aux_create) {
      g_aux_api.aux_create(id, width, height);
    }

    // Create Flutter texture variant.
    texture_variant_ =
        std::make_unique<flutter::TextureVariant>(flutter::PixelBufferTexture(
            [this](size_t w, size_t h) -> const FlutterDesktopPixelBuffer * {
              return texture_ ? texture_->CopyPixelBuffer(w, h) : nullptr;
            }));
  }

  ~AuxTextureEntry() {
    if (g_aux_api.aux_destroy) {
      g_aux_api.aux_destroy(id_);
    }
  }

  // Copies from Rust buffer into the back buffer and commits.
  void UpdateFromRust() {
    if (!texture_ || !g_aux_api.aux_copy)
      return;

    auto [dst, write_index] = texture_->acquireWritableBuffer();
    g_aux_api.aux_copy(id_, dst, static_cast<uint32_t>(texture_->stride()),
                       static_cast<uint32_t>(texture_->height()));
    texture_->commitLatestReady(write_index);
  }

  flutter::TextureVariant *texture_variant() { return texture_variant_.get(); }

private:
  uint32_t id_;
  std::unique_ptr<NesiumTexture> texture_;
  std::unique_ptr<flutter::TextureVariant> texture_variant_;
};

class NesiumAuxTexturePlugin : public flutter::Plugin {
public:
  explicit NesiumAuxTexturePlugin(flutter::PluginRegistrarWindows *registrar)
      : registrar_(registrar),
        texture_registrar_(registrar->texture_registrar()) {
    // Bind to the already-loaded Rust DLL
    if (!g_aux_api.BindLoadedModule(L"nesium_flutter.dll")) {
      // Log error but continue - textures just won't work until DLL is loaded
      OutputDebugStringW(
          L"NesiumAuxTexturePlugin: Failed to bind to nesium_flutter.dll\n");
    }

    channel_ =
        std::make_unique<flutter::MethodChannel<flutter::EncodableValue>>(
            registrar_->messenger(), "nesium_aux",
            &flutter::StandardMethodCodec::GetInstance());

    channel_->SetMethodCallHandler([this](const auto &call, auto result) {
      HandleMethodCall(call, std::move(result));
    });

    // Update thread: periodically updates all registered textures from Rust
    // buffers.
    update_thread_ = std::thread([this] { UpdateThreadMain(); });
  }

  ~NesiumAuxTexturePlugin() override {
    shutting_down_.store(true, std::memory_order_release);
    if (update_thread_.joinable()) {
      update_thread_.join();
    }
  }

private:
  void HandleMethodCall(
      const flutter::MethodCall<flutter::EncodableValue> &call,
      std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result) {
    if (call.method_name() == "createAuxTexture") {
      CreateAuxTexture(call, std::move(result));
      return;
    }
    if (call.method_name() == "disposeAuxTexture") {
      DisposeAuxTexture(call, std::move(result));
      return;
    }
    result->NotImplemented();
  }

  void CreateAuxTexture(
      const flutter::MethodCall<flutter::EncodableValue> &call,
      std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result) {
    const auto *args = std::get_if<flutter::EncodableMap>(call.arguments());
    if (!args) {
      result->Error("BAD_ARGS", "Missing arguments");
      return;
    }

    auto id_it = args->find(flutter::EncodableValue("id"));
    auto width_it = args->find(flutter::EncodableValue("width"));
    auto height_it = args->find(flutter::EncodableValue("height"));

    if (id_it == args->end() || width_it == args->end() ||
        height_it == args->end()) {
      result->Error("BAD_ARGS", "Missing id/width/height");
      return;
    }

    const auto *id_val = std::get_if<int32_t>(&id_it->second);
    const auto *width_val = std::get_if<int32_t>(&width_it->second);
    const auto *height_val = std::get_if<int32_t>(&height_it->second);

    if (!id_val || !width_val || !height_val) {
      result->Error("BAD_ARGS", "Invalid argument types");
      return;
    }

    const uint32_t id = static_cast<uint32_t>(*id_val);
    const uint32_t width = static_cast<uint32_t>(*width_val);
    const uint32_t height = static_cast<uint32_t>(*height_val);

    std::lock_guard<std::mutex> lock(textures_mutex_);

    // Clean up any existing texture with this ID.
    auto existing_it = textures_.find(id);
    if (existing_it != textures_.end()) {
      texture_registrar_->UnregisterTexture(existing_it->second.flutter_id);
      textures_.erase(existing_it);
    }

    // Create new texture entry.
    auto entry = std::make_unique<AuxTextureEntry>(id, width, height);
    const int64_t flutter_id =
        texture_registrar_->RegisterTexture(entry->texture_variant());

    textures_[id] = {flutter_id, std::move(entry)};

    result->Success(flutter::EncodableValue(flutter_id));
  }

  void DisposeAuxTexture(
      const flutter::MethodCall<flutter::EncodableValue> &call,
      std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result) {
    const auto *args = std::get_if<flutter::EncodableMap>(call.arguments());
    if (!args) {
      result->Error("BAD_ARGS", "Missing arguments");
      return;
    }

    auto id_it = args->find(flutter::EncodableValue("id"));
    if (id_it == args->end()) {
      result->Error("BAD_ARGS", "Missing id");
      return;
    }

    const auto *id_val = std::get_if<int32_t>(&id_it->second);
    if (!id_val) {
      result->Error("BAD_ARGS", "Invalid id type");
      return;
    }

    const uint32_t id = static_cast<uint32_t>(*id_val);

    std::lock_guard<std::mutex> lock(textures_mutex_);

    auto it = textures_.find(id);
    if (it != textures_.end()) {
      texture_registrar_->UnregisterTexture(it->second.flutter_id);
      textures_.erase(it);
    }

    result->Success(flutter::EncodableValue());
  }

  void UpdateThreadMain() {
    // Update at ~60Hz.
    const int sleep_ms = 16;

    while (!shutting_down_.load(std::memory_order_acquire)) {
      {
        std::lock_guard<std::mutex> lock(textures_mutex_);
        for (auto &[id, tex_info] : textures_) {
          tex_info.entry->UpdateFromRust();
          texture_registrar_->MarkTextureFrameAvailable(tex_info.flutter_id);
        }
      }

      ::Sleep(sleep_ms);
    }
  }

private:
  flutter::PluginRegistrarWindows *registrar_;
  flutter::TextureRegistrar *texture_registrar_;
  std::unique_ptr<flutter::MethodChannel<flutter::EncodableValue>> channel_;

  struct TextureInfo {
    int64_t flutter_id;
    std::unique_ptr<AuxTextureEntry> entry;
  };

  std::mutex textures_mutex_;
  std::map<uint32_t, TextureInfo> textures_;

  std::atomic<bool> shutting_down_{false};
  std::thread update_thread_;
};

} // namespace

void NesiumAuxTexturePluginRegisterWithRegistrar(
    FlutterDesktopPluginRegistrarRef registrar) {
  auto *cpp_registrar =
      flutter::PluginRegistrarManager::GetInstance()
          ->GetRegistrar<flutter::PluginRegistrarWindows>(registrar);

  auto plugin = std::make_unique<NesiumAuxTexturePlugin>(cpp_registrar);
  cpp_registrar->AddPlugin(std::move(plugin));
}

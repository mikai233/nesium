#include "nesium_aux_texture_plugin.h"

#include <atomic>
#include <cstdint>
#include <map>
#include <memory>
#include <mutex>
#include <set>
#include <thread>
#include <windows.h>

#include "flutter/method_channel.h"
#include "flutter/plugin_registrar.h"
#include "flutter/plugin_registrar_windows.h"
#include "flutter/standard_method_codec.h"
#include "flutter/texture_registrar.h"

#include "nesium_rust_ffi.h"
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

// Represents one auxiliary texture registered with Flutter.
class AuxTextureEntry {
public:
  AuxTextureEntry(uint32_t id, uint32_t width, uint32_t height)
      : id_(id), texture_(std::make_unique<NesiumTexture>(width, height)) {
    // Create the Rust-side backing store.
    nesium_aux_create(id, width, height);

    // Create Flutter texture variant.
    texture_variant_ =
        std::make_unique<flutter::TextureVariant>(flutter::PixelBufferTexture(
            [this](size_t w, size_t h) -> const FlutterDesktopPixelBuffer * {
              return texture_ ? texture_->CopyPixelBuffer(w, h) : nullptr;
            }));
  }

  ~AuxTextureEntry() {
    nesium_aux_destroy(id_);
  }

  // Copies from Rust buffer into the back buffer and commits.
  void UpdateFromRust() {
    if (!texture_)
      return;

    auto [dst, write_index] = texture_->acquireWritableBuffer();
    nesium_aux_copy(id_, dst, static_cast<uint32_t>(texture_->stride()),
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
    if (call.method_name() == "pauseAuxTexture") {
      PauseAuxTexture(call, std::move(result));
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
      // Unregistration is asynchronous. Keep the entry alive until the engine
      // completes unregistration to avoid use-after-free in texture callbacks.
      auto keep_alive = existing_it->second.entry;
      texture_registrar_->UnregisterTexture(existing_it->second.flutter_id,
                                            [keep_alive]() {});
      textures_.erase(existing_it);
    }

    // Create new texture entry.
    auto entry = std::make_shared<AuxTextureEntry>(id, width, height);
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
      // Unregistration is asynchronous. Keep the entry alive until the engine
      // completes unregistration to avoid use-after-free in texture callbacks.
      auto keep_alive = it->second.entry;
      texture_registrar_->UnregisterTexture(it->second.flutter_id,
                                            [keep_alive]() {});
      textures_.erase(it);
    }
    paused_ids_.erase(id);

    result->Success(flutter::EncodableValue());
  }

  void PauseAuxTexture(
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
    paused_ids_.insert(id);

    result->Success(flutter::EncodableValue());
  }

  void UpdateThreadMain() {
    // Update at ~60Hz.
    const int sleep_ms = 16;

    while (!shutting_down_.load(std::memory_order_acquire)) {
      {
        std::lock_guard<std::mutex> lock(textures_mutex_);
        for (auto &[id, tex_info] : textures_) {
          // Skip paused textures.
          if (paused_ids_.count(id) > 0)
            continue;
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
    std::shared_ptr<AuxTextureEntry> entry;
  };

  std::mutex textures_mutex_;
  std::map<uint32_t, TextureInfo> textures_;
  std::set<uint32_t> paused_ids_;

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

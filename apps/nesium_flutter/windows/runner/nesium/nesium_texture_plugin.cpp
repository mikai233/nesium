#include "nesium_texture_plugin.h"

#include <atomic>
#include <condition_variable>
#include <cstdint>
#include <memory>
#include <mutex>
#include <thread>

#include "flutter/method_channel.h"
#include "flutter/plugin_registrar.h"
#include "flutter/plugin_registrar_windows.h"
#include "flutter/standard_method_codec.h"
#include "flutter/texture_registrar.h"

#include "nesium_rust_ffi.h"
#include "nesium_texture.h"

// Windows software-texture backend for Nesium (Flutter desktop).
//
// Design notes:
// - The Rust library is linked as an import library and will be loaded by the
//   OS loader when the Runner starts (provided `nesium_flutter.dll` is alongside
//   the executable).
// - This plugin wires the "frame-ready" callback to a copy worker thread.
// - Frames are copied into a double-buffered CPU RGBA backing store (see
// `NesiumTexture`),
//   and `MarkTextureFrameAvailable(textureId)` is used to notify Flutter that a
//   new frame is ready.

namespace {

constexpr uint32_t kEmptyPending = 0xFFFFFFFFu;

class NesiumTexturePlugin : public flutter::Plugin {
public:
  explicit NesiumTexturePlugin(flutter::PluginRegistrarWindows *registrar)
      : registrar_(registrar),
        texture_registrar_(registrar->texture_registrar()) {
    channel_ =
        std::make_unique<flutter::MethodChannel<flutter::EncodableValue>>(
            registrar_->messenger(), "nesium",
            &flutter::StandardMethodCodec::GetInstance());

    channel_->SetMethodCallHandler([this](const auto &call, auto result) {
      HandleMethodCall(call, std::move(result));
    });

    // Copy worker thread:
    // - waits for a pending bufferIndex (latest-only)
    // - calls Rust `copy_frame` into CPU buffer
    // - marks Flutter texture frame available
    worker_ = std::thread([this] { CopyWorkerMain(); });
  }

  ~NesiumTexturePlugin() override {
    nesium_set_frame_ready_callback(nullptr, nullptr);
    shutting_down_.store(true, std::memory_order_release);
    {
      std::lock_guard<std::mutex> lk(mu_);
      cv_.notify_all();
    }
    if (worker_.joinable()) {
      worker_.join();
    }
  }

private:
  void HandleMethodCall(
      const flutter::MethodCall<flutter::EncodableValue> &call,
      std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result) {
    if (call.method_name() == "createNesTexture") {
      CreateNesTexture(std::move(result));
      return;
    }
    if (call.method_name() == "disposeNesTexture") {
      DisposeNesTexture(std::move(result));
      return;
    }
    result->NotImplemented();
  }

  void CreateNesTexture(
      std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result) {
    if (texture_id_.load(std::memory_order_acquire) >= 0) {
      result->Success(flutter::EncodableValue(texture_id_.load()));
      return;
    }

    // NES native framebuffer size.
    const int width = 256;
    const int height = 240;

    texture_ = std::make_unique<NesiumTexture>(width, height);

    // Flutter software texture: engine calls this callback to fetch the latest
    // CPU buffer.
    texture_variant_ =
        std::make_unique<flutter::TextureVariant>(flutter::PixelBufferTexture(
            [this](size_t w, size_t h) -> const FlutterDesktopPixelBuffer * {
              return texture_ ? texture_->CopyPixelBuffer(w, h) : nullptr;
            }));

    const int64_t id =
        texture_registrar_->RegisterTexture(texture_variant_.get());
    texture_id_.store(id, std::memory_order_release);

    // Wire callback and start runtime after texture registration is ready.
    nesium_set_frame_ready_callback(&NesiumTexturePlugin::OnFrameReadyThunk,
                                    this);
    nesium_runtime_start();

    result->Success(flutter::EncodableValue(id));
  }

  void DisposeNesTexture(
      std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result) {
    nesium_set_frame_ready_callback(nullptr, nullptr);

    pending_index_.store(kEmptyPending, std::memory_order_release);
    copy_scheduled_.store(false, std::memory_order_release);

    const int64_t id = texture_id_.load(std::memory_order_acquire);
    if (id >= 0) {
      texture_registrar_->UnregisterTexture(id);
    }

    texture_variant_.reset();
    texture_.reset();
    texture_id_.store(-1, std::memory_order_release);

    result->Success(flutter::EncodableValue());
  }

  static void OnFrameReadyThunk(uint32_t bufferIndex, uint32_t width,
                                uint32_t height, uint32_t pitch, void *user) {
    static_cast<NesiumTexturePlugin *>(user)->OnFrameReady(bufferIndex, width,
                                                           height, pitch);
  }

  // Called from the Rust runtime thread. Must be lightweight and non-blocking.
  // We store only the latest bufferIndex and schedule at most one copy drain.
  void OnFrameReady(uint32_t bufferIndex, uint32_t, uint32_t, uint32_t) {
    pending_index_.store(bufferIndex, std::memory_order_release);

    bool expected = false;
    if (!copy_scheduled_.compare_exchange_strong(expected, true,
                                                 std::memory_order_acq_rel)) {
      return;
    }

    std::lock_guard<std::mutex> lk(mu_);
    cv_.notify_one();
  }

  void CopyWorkerMain() {
    while (!shutting_down_.load(std::memory_order_acquire)) {
      {
        std::unique_lock<std::mutex> lk(mu_);
        cv_.wait(lk, [this] {
          return shutting_down_.load(std::memory_order_acquire) ||
                 copy_scheduled_.load(std::memory_order_acquire);
        });
      }

      if (shutting_down_.load(std::memory_order_acquire)) {
        break;
      }

      const uint32_t idx =
          pending_index_.exchange(kEmptyPending, std::memory_order_acq_rel);
      copy_scheduled_.store(false, std::memory_order_release);

      auto *tex = texture_.get();
      const int64_t tid = texture_id_.load(std::memory_order_acquire);
      if (!tex || tid < 0 || idx == kEmptyPending) {
        continue;
      }

      // Copy RGBA pixels into the back buffer, then publish it atomically.
      auto [dst, write_index] = tex->acquireWritableBuffer();
      nesium_copy_frame(idx, dst, static_cast<uint32_t>(tex->stride()),
                        static_cast<uint32_t>(tex->height()));
      tex->commitLatestReady(write_index);

      // Notify Flutter that the texture has a new frame.
      texture_registrar_->MarkTextureFrameAvailable(tid);

      // If another frame arrived while copying, schedule one more drain
      // (latest-only).
      if (pending_index_.load(std::memory_order_acquire) != kEmptyPending) {
        bool expected = false;
        if (copy_scheduled_.compare_exchange_strong(
                expected, true, std::memory_order_acq_rel)) {
          std::lock_guard<std::mutex> lk(mu_);
          cv_.notify_one();
        }
      }
    }
  }

private:
  flutter::PluginRegistrarWindows *registrar_;
  flutter::TextureRegistrar *texture_registrar_;
  std::unique_ptr<flutter::MethodChannel<flutter::EncodableValue>> channel_;

  std::unique_ptr<NesiumTexture> texture_;
  std::unique_ptr<flutter::TextureVariant> texture_variant_;

  std::atomic<int64_t> texture_id_{-1};

  // Latest-only signaling from Rust thread -> copy worker thread.
  std::atomic<uint32_t> pending_index_{kEmptyPending};
  std::atomic<bool> copy_scheduled_{false};
  std::atomic<bool> shutting_down_{false};

  std::mutex mu_;
  std::condition_variable cv_;
  std::thread worker_;
};

} // namespace

void NesiumTexturePluginRegisterWithRegistrar(
    FlutterDesktopPluginRegistrarRef registrar) {
  auto *cpp_registrar =
      flutter::PluginRegistrarManager::GetInstance()
          ->GetRegistrar<flutter::PluginRegistrarWindows>(registrar);

  auto plugin = std::make_unique<NesiumTexturePlugin>(cpp_registrar);
  cpp_registrar->AddPlugin(std::move(plugin));
}

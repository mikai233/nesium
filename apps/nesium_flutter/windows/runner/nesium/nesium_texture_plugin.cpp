#include "nesium_texture_plugin.h"

#include <atomic>
#include <condition_variable>
#include <cstdint>
#include <memory>
#include <mutex>
#include <thread>
#include <utility>

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>

#include "flutter/plugin_registrar.h"
#include "flutter/plugin_registrar_windows.h"
#include "flutter/standard_method_codec.h"
#include "flutter/texture_registrar.h"
#include <avrt.h>
#include <flutter/method_channel.h>

#include "nesium_gpu_texture.h"
#include "nesium_rust_ffi.h"
#include "nesium_texture.h"

// Windows texture backend for Nesium (Flutter desktop).
//
// Design notes:
// - Attempts to use D3D11 GPU texture sharing for zero-copy frame presentation.
// - Falls back to CPU PixelBufferTexture if D3D11 initialization fails.
// - The Rust library is linked as an import library and will be loaded by the
//   OS loader when the Runner starts.

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

    // Initial priority application
    if (prefer_high_priority_.load()) {
      ::SetPriorityClass(::GetCurrentProcess(), HIGH_PRIORITY_CLASS);
    }

    // Copy/present worker thread (used for both GPU and CPU paths)
    worker_ = std::thread([this] { WorkerMain(); });
  }

  ~NesiumTexturePlugin() {
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
      int width = texture_width_;
      int height = texture_height_;
      if (const auto *args =
              std::get_if<flutter::EncodableMap>(call.arguments())) {
        auto it_w = args->find(flutter::EncodableValue("width"));
        auto it_h = args->find(flutter::EncodableValue("height"));
        if (it_w != args->end() && it_h != args->end()) {
          width = std::get<int>(it_w->second);
          height = std::get<int>(it_h->second);
        }
      }
      CreateNesTexture(std::move(result), width, height);
      return;
    }
    if (call.method_name() == "setPresentBufferSize") {
      SetPresentBufferSize(call, std::move(result));
      return;
    }
    if (call.method_name() == "disposeNesTexture") {
      DisposeNesTexture(std::move(result));
      return;
    }
    if (call.method_name() == "setWindowsVideoBackend") {
      SetWindowsVideoBackend(call, std::move(result));
      return;
    }
    if (call.method_name() == "setWindowsHighPriority") {
      SetWindowsHighPriority(call, std::move(result));
      return;
    }
    result->NotImplemented();
  }

  void SetWindowsHighPriority(
      const flutter::MethodCall<flutter::EncodableValue> &call,
      std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result) {
    const auto *args = std::get_if<flutter::EncodableMap>(call.arguments());
    if (!args) {
      result->Error("Invalid arguments", "Expected map");
      return;
    }

    auto it = args->find(flutter::EncodableValue("enabled"));
    if (it == args->end()) {
      result->Error("Invalid arguments", "Missing enabled");
      return;
    }

    bool enabled = std::get<bool>(it->second);
    prefer_high_priority_.store(enabled, std::memory_order_release);

    if (enabled) {
      ::SetPriorityClass(::GetCurrentProcess(), HIGH_PRIORITY_CLASS);
    } else {
      ::SetPriorityClass(::GetCurrentProcess(), NORMAL_PRIORITY_CLASS);
    }

    if (result) {
      result->Success();
    }
  }

  void CreateNesTexture(
      std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result,
      int width, int height) {
    if (texture_id_.load(std::memory_order_acquire) >= 0) {
      if (result) {
        result->Success(flutter::EncodableValue(texture_id_.load()));
      }
      return;
    }

    if (width <= 0 || height <= 0) {
      if (result) {
        result->Error("Invalid arguments", "width/height must be > 0");
      }
      return;
    }

    texture_width_ = width;
    texture_height_ = height;

    std::shared_ptr<NesiumGpuTexture> gpu_texture;
    std::shared_ptr<NesiumTexture> cpu_texture;
    std::shared_ptr<flutter::TextureVariant> texture_variant;

    // Try D3D11 GPU texture if preferred.
    if (prefer_gpu_) {
      IDXGIAdapter *adapter = nullptr;
      if (auto *view = registrar_->GetView()) {
        adapter = view->GetGraphicsAdapter();
      }
      // Use current known source size (or default) and requested destination
      // size.
      gpu_texture = NesiumGpuTexture::Create(src_width_, src_height_, width,
                                             height, adapter);
    }

    if (gpu_texture && gpu_texture->is_valid()) {
      // GPU path: use GpuSurfaceTexture with DXGI shared handle.
      //
      // Rationale: We signal 'false' (RGBA) to the Rust core because
      // librashader currently requires RGBA intermediate textures on Windows.
      // The conversion to BGRA (required by Flutter/D2D) is performed by the
      // shader backend.
      nesium_set_color_format(false);
      auto gpu_texture_for_callback = gpu_texture;
      texture_variant =
          std::make_shared<flutter::TextureVariant>(flutter::GpuSurfaceTexture(
              kFlutterDesktopGpuSurfaceTypeDxgiSharedHandle,
              [gpu_texture_for_callback](size_t w, size_t h)
                  -> const FlutterDesktopGpuSurfaceDescriptor * {
                return gpu_texture_for_callback
                           ? gpu_texture_for_callback->GetGpuSurface(w, h)
                           : nullptr;
              }));
    } else {
      // Fallback: CPU PixelBufferTexture.
      // CPU path uses RGBA format.
      nesium_set_color_format(false);
      gpu_texture.reset();
      // CPU texture works on source size, scaling happens in Flutter.
      // Or maybe it should also be decoupled? CPU texture usually just copies,
      // so it's source size.
      cpu_texture = std::make_shared<NesiumTexture>(src_width_, src_height_);
      auto cpu_texture_for_callback = cpu_texture;
      texture_variant =
          std::make_shared<flutter::TextureVariant>(flutter::PixelBufferTexture(
              [cpu_texture_for_callback](
                  size_t w, size_t h) -> const FlutterDesktopPixelBuffer * {
                return cpu_texture_for_callback
                           ? cpu_texture_for_callback->CopyPixelBuffer(w, h)
                           : nullptr;
              }));
    }

    {
      std::lock_guard<std::mutex> lk(texture_state_mu_);
      gpu_texture_ = std::move(gpu_texture);
      cpu_texture_ = std::move(cpu_texture);
      texture_variant_ = std::move(texture_variant);
    }

    const int64_t id =
        texture_registrar_->RegisterTexture(texture_variant_.get());
    texture_id_.store(id, std::memory_order_release);

    // Wire callback and start runtime after texture registration is ready.
    nesium_set_frame_ready_callback(&NesiumTexturePlugin::OnFrameReadyThunk,
                                    this);
    nesium_runtime_start();

    if (result) {
      result->Success(flutter::EncodableValue(id));
    }
  }

  void DisposeNesTexture(
      std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result) {
    nesium_set_frame_ready_callback(nullptr, nullptr);

    pending_index_.store(kEmptyPending, std::memory_order_release);
    work_scheduled_.store(false, std::memory_order_release);

    const int64_t id = texture_id_.exchange(-1, std::memory_order_acq_rel);

    std::shared_ptr<flutter::TextureVariant> texture_variant_to_release;
    {
      std::lock_guard<std::mutex> lk(texture_state_mu_);
      texture_variant_to_release = std::move(texture_variant_);
      gpu_texture_.reset();
      cpu_texture_.reset();
    }

    if (id >= 0) {
      // Unregistration is asynchronous. Keep the registered TextureVariant
      // alive until the engine completes unregistration to avoid use-after-free
      // in texture callbacks.
      texture_registrar_->UnregisterTexture(id,
                                            [texture_variant_to_release]() {});
    }

    if (result) {
      result->Success(flutter::EncodableValue());
    }
  }

  void SetPresentBufferSize(
      const flutter::MethodCall<flutter::EncodableValue> &call,
      std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result) {
    const auto *args = std::get_if<flutter::EncodableMap>(call.arguments());
    if (!args) {
      result->Error("Invalid arguments", "Expected map");
      return;
    }

    auto it_w = args->find(flutter::EncodableValue("width"));
    auto it_h = args->find(flutter::EncodableValue("height"));
    if (it_w == args->end() || it_h == args->end()) {
      result->Error("Invalid arguments", "Missing width/height");
      return;
    }

    int width = std::get<int>(it_w->second);
    int height = std::get<int>(it_h->second);
    if (width <= 0 || height <= 0) {
      result->Error("Invalid arguments", "width/height must be > 0");
      return;
    }

    texture_width_ = width;
    texture_height_ = height;

    // Best-effort: pre-resize the presentation buffer.
    {
      std::lock_guard<std::mutex> lk(texture_state_mu_);
      if (gpu_texture_) {
        gpu_texture_->ResizeOutput(width, height);
      }
      // CPU texture: we don't resize output here because it's driven by source
      // content size usually. Flutter scales it.
    }
    result->Success();
  }

  void SetWindowsVideoBackend(
      const flutter::MethodCall<flutter::EncodableValue> &call,
      std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result) {
    const auto *args = std::get_if<flutter::EncodableMap>(call.arguments());
    if (!args) {
      result->Error("Invalid arguments", "Expected map");
      return;
    }

    auto it = args->find(flutter::EncodableValue("useGpu"));
    if (it == args->end()) {
      result->Error("Invalid arguments", "Missing useGpu");
      return;
    }

    bool use_gpu = std::get<bool>(it->second);
    if (use_gpu == prefer_gpu_) {
      result->Success();
      return;
    }

    prefer_gpu_ = use_gpu;
    int64_t new_id = -1;

    // If texture is already active, we must recreate it to apply the change.
    if (texture_id_.load(std::memory_order_acquire) >= 0) {
      DisposeNesTexture(nullptr);
      CreateNesTexture(nullptr, texture_width_, texture_height_);
      new_id = texture_id_.load(std::memory_order_acquire);
    }

    if (new_id >= 0) {
      result->Success(flutter::EncodableValue(new_id));
    } else {
      result->Success();
    }
  }

  static void OnFrameReadyThunk(uint32_t bufferIndex, uint32_t width,
                                uint32_t height, uint32_t pitch, void *user) {
    static_cast<NesiumTexturePlugin *>(user)->OnFrameReady(bufferIndex, width,
                                                           height, pitch);
  }

  // Called from the Rust runtime thread. Must be lightweight and non-blocking.
  void OnFrameReady(uint32_t bufferIndex, uint32_t width, uint32_t height,
                    uint32_t) {
    pending_width_.store(width, std::memory_order_release);
    pending_height_.store(height, std::memory_order_release);
    pending_index_.store(bufferIndex, std::memory_order_release);

    bool expected = false;
    if (!work_scheduled_.compare_exchange_strong(expected, true,
                                                 std::memory_order_acq_rel)) {
      return;
    }

    std::lock_guard<std::mutex> lk(mu_);
    cv_.notify_one();
  }

  void WorkerMain() {
    DWORD mm_task_index = 0;
    HANDLE mm_handle = nullptr;

    while (!shutting_down_.load(std::memory_order_acquire)) {
      const bool high_prio =
          prefer_high_priority_.load(std::memory_order_acquire);
      if (high_prio && !mm_handle) {
        mm_handle = ::AvSetMmThreadCharacteristicsW(L"Games", &mm_task_index);
      } else if (!high_prio && mm_handle) {
        ::AvRevertMmThreadCharacteristics(mm_handle);
        mm_handle = nullptr;
      }
      {
        std::unique_lock<std::mutex> lk(mu_);
        cv_.wait(lk, [this] {
          return shutting_down_.load(std::memory_order_acquire) ||
                 work_scheduled_.load(std::memory_order_acquire);
        });
      }

      if (shutting_down_.load(std::memory_order_acquire)) {
        break;
      }

      const uint32_t idx =
          pending_index_.exchange(kEmptyPending, std::memory_order_acq_rel);
      work_scheduled_.store(false, std::memory_order_release);

      const int64_t tid = texture_id_.load(std::memory_order_acquire);
      if (tid < 0 || idx == kEmptyPending) {
        continue;
      }

      std::shared_ptr<NesiumGpuTexture> gpu_texture;
      std::shared_ptr<NesiumTexture> cpu_texture;
      {
        std::lock_guard<std::mutex> lk(texture_state_mu_);
        gpu_texture = gpu_texture_;
        cpu_texture = cpu_texture_;
      }

      const uint32_t frame_w = pending_width_.load(std::memory_order_acquire);
      const uint32_t frame_h = pending_height_.load(std::memory_order_acquire);
      if (frame_w == 0 || frame_h == 0) {
        continue;
      }

      if (gpu_texture) {
        // Resize Source if frame size changed
        if (src_width_ != static_cast<int>(frame_w) ||
            src_height_ != static_cast<int>(frame_h)) {
          gpu_texture->ResizeSource(static_cast<int>(frame_w),
                                    static_cast<int>(frame_h));
          src_width_ = static_cast<int>(frame_w);
          src_height_ = static_cast<int>(frame_h);
        }
        // Note: Output dimension updates (dst_width/height) are handled via
        // SetPresentBufferSize, which triggers ResizeOutput on the GPU texture.

      } else if (cpu_texture) {
        if (cpu_texture->width() != static_cast<int>(frame_w) ||
            cpu_texture->height() != static_cast<int>(frame_h)) {
          cpu_texture->Resize(static_cast<int>(frame_w),
                              static_cast<int>(frame_h));
          src_width_ = static_cast<int>(frame_w);
          src_height_ = static_cast<int>(frame_h);
        }
      }

      if (gpu_texture) {
        // GPU path: map, copy, unmap, commit
        auto [dst, pitch] = gpu_texture->MapWriteBuffer();
        if (dst) {
          nesium_copy_frame(idx, dst, pitch,
                            static_cast<uint32_t>(
                                gpu_texture->height())); // This is src height
          gpu_texture->UnmapAndCommit();
        }
      } else if (cpu_texture) {
        // CPU fallback path
        auto [dst, write_index] = cpu_texture->acquireWritableBuffer();
        nesium_copy_frame(idx, dst,
                          static_cast<uint32_t>(cpu_texture->stride()),
                          static_cast<uint32_t>(cpu_texture->height()));
        cpu_texture->commitLatestReady(write_index);
      }

      // Notify Flutter that the texture has a new frame.
      texture_registrar_->MarkTextureFrameAvailable(tid);

      // If another frame arrived while processing, schedule one more drain.
      if (pending_index_.load(std::memory_order_acquire) != kEmptyPending) {
        bool expected = false;
        if (work_scheduled_.compare_exchange_strong(
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

  // GPU texture (preferred)
  std::shared_ptr<NesiumGpuTexture> gpu_texture_;
  // CPU texture (fallback)
  std::shared_ptr<NesiumTexture> cpu_texture_;

  std::shared_ptr<flutter::TextureVariant> texture_variant_;
  bool prefer_gpu_ = true;

  std::atomic<int64_t> texture_id_{-1};

  int texture_width_ = 256;
  int texture_height_ = 240;
  int src_width_ = 256;
  int src_height_ = 240;

  // Latest-only signaling from Rust thread -> worker thread.
  std::atomic<uint32_t> pending_index_{kEmptyPending};
  std::atomic<uint32_t> pending_width_{0};
  std::atomic<uint32_t> pending_height_{0};
  std::atomic<bool> work_scheduled_{false};
  std::atomic<bool> shutting_down_{false};
  std::atomic<bool> prefer_high_priority_{true};

  std::mutex texture_state_mu_;

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

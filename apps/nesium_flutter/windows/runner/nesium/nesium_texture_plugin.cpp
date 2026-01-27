#include "nesium_texture_plugin.h"

#include <atomic>
#include <chrono>
#include <cstdint>
#include <map>
#include <memory>
#include <mutex>
#include <numeric>
#include <utility>
#include <variant>
#include <vector>

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>

#include "flutter/plugin_registrar.h"
#include "flutter/plugin_registrar_windows.h"
#include "flutter/standard_method_codec.h"
#include "flutter/texture_registrar.h"
#include <flutter/encodable_value.h>
#include <flutter/method_channel.h>

#include "nesium_gpu_texture.h"
#include "nesium_native_window.h"
#include "nesium_rust_ffi.h"
#include "nesium_texture.h"

#pragma comment(lib, "d3d11.lib")
#pragma comment(lib, "dxgi.lib")

// Windows texture backend for Nesium (Flutter desktop).
//
// Design notes:
// - Attempts to use D3D11 GPU texture sharing for zero-copy frame presentation.
// - Falls back to CPU PixelBufferTexture if D3D11 initialization fails.
// - The Rust library is linked as an import library and will be loaded by the
//   OS loader when the Runner starts.

namespace {

class NesiumTexturePlugin;

class NesiumTexturePlugin : public flutter::Plugin {
public:
  static double GetDouble(const flutter::EncodableValue &value) {
    if (std::holds_alternative<double>(value))
      return std::get<double>(value);
    if (std::holds_alternative<int32_t>(value))
      return static_cast<double>(std::get<int32_t>(value));
    if (std::holds_alternative<int64_t>(value))
      return static_cast<double>(std::get<int64_t>(value));
    return 0.0;
  }
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

    if (auto *view = registrar_->GetView()) {
      parent_hwnd_ = view->GetNativeWindow();
      OutputDebugStringA("[Nesium] Plugin initialized with View HWND\n");
    }
  }

  virtual ~NesiumTexturePlugin() {
    nesium_set_frame_ready_callback(nullptr, nullptr);
    shutting_down_.store(true, std::memory_order_release);
  }

private:
  void HandleMethodCall(
      const flutter::MethodCall<flutter::EncodableValue> &call,
      std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result) {
    const std::string &method = call.method_name();
    if (method == "createNesTexture") {
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
    } else if (method == "setPresentBufferSize") {
      SetPresentBufferSize(call, std::move(result));
    } else if (method == "disposeNesTexture") {
      DisposeNesTexture(std::move(result));
    } else if (method == "setWindowsVideoBackend") {
      SetWindowsVideoBackend(call, std::move(result));
    } else if (method == "setNativeOverlay") {
      SetNativeOverlay(call, std::move(result));
    } else {
      result->NotImplemented();
    }
  }

  void SetNativeOverlay(
      const flutter::MethodCall<flutter::EncodableValue> &call,
      std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result) {
    OutputDebugStringA("[Nesium] SetNativeOverlay called\n");
    const auto *args = std::get_if<flutter::EncodableMap>(call.arguments());
    if (!args) {
      result->Error("Invalid arguments", "Expected map");
      return;
    }

    auto it_enabled = args->find(flutter::EncodableValue("enabled"));
    bool enabled =
        it_enabled != args->end() && std::get<bool>(it_enabled->second);

    if (enabled) {
      OutputDebugStringA("[Nesium] Native Overlay Enabled\n");
      if (!native_window_) {
        OutputDebugStringA("[Nesium] Creating Native Window...\n");
        std::shared_ptr<NesiumGpuTexture> gpu_texture;
        {
          std::lock_guard<std::mutex> lk(texture_state_mu_);
          gpu_texture = gpu_texture_;
        }
        if (gpu_texture && parent_hwnd_) {
          native_window_ = NesiumNativeWindow::Create(
              parent_hwnd_,
              reinterpret_cast<ID3D11Device *>(gpu_texture->GetDevice()));
        } else {
          if (!gpu_texture)
            OutputDebugStringA("[Nesium] SKIP: gpu_texture is null\n");
          if (!parent_hwnd_)
            OutputDebugStringA("[Nesium] SKIP: parent_hwnd_ is null\n");
        }
      }

      if (native_window_) {
        auto it_x = args->find(flutter::EncodableValue("x"));
        auto it_y = args->find(flutter::EncodableValue("y"));
        auto it_w = args->find(flutter::EncodableValue("width"));
        auto it_h = args->find(flutter::EncodableValue("height"));

        if (it_x != args->end() && it_y != args->end() && it_w != args->end() &&
            it_h != args->end()) {
          overlay_x_ = static_cast<int>(GetDouble(it_x->second));
          overlay_y_ = static_cast<int>(GetDouble(it_y->second));
          overlay_w_ = static_cast<int>(GetDouble(it_w->second));
          overlay_h_ = static_cast<int>(GetDouble(it_h->second));
          UpdateOverlayPos();
        }
        native_window_->SetVisible(true);
      }
    } else {
      if (native_window_) {
        native_window_->SetVisible(false);
      }
    }

    result->Success();
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
    {
      char buf[256];
      sprintf_s(
          buf,
          "[Nesium] CreateNesTexture: prefer_gpu=%d, src=%dx%d, dst=%dx%d\n",
          prefer_gpu_, src_width_, src_height_, width, height);
      OutputDebugStringA(buf);
    }

    if (prefer_gpu_) {
      IDXGIAdapter *adapter = nullptr;
      if (auto *view = registrar_->GetView()) {
        adapter = view->GetGraphicsAdapter();
        char buf[128];
        sprintf_s(buf, "[Nesium] Graphics Adapter: %p\n", adapter);
        OutputDebugStringA(buf);
      } else {
        OutputDebugStringA("[Nesium] SKIP: view is null\n");
      }
      // Use current known source size (or default) and requested destination
      // size.
      gpu_texture = NesiumGpuTexture::Create(src_width_, src_height_, width,
                                             height, adapter);
      if (!gpu_texture) {
        OutputDebugStringA(
            "[Nesium] NesiumGpuTexture::Create failed (returned null)\n");
      } else if (!gpu_texture->is_valid()) {
        OutputDebugStringA(
            "[Nesium] NesiumGpuTexture is invalid after Create\n");
      }
    }

    if (gpu_texture && gpu_texture->is_valid()) {
      // GPU path: use GpuSurfaceTexture with DXGI shared handle.
      //
      // Optimization: We now use a pure BGRA pipeline.
      // Core (BGRA) -> Staging (BGRA) -> Shader Input (BGRA) -> Shared (BGRA).
      nesium_set_color_format(true);
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
      OutputDebugStringA("[Nesium] Falling back to CPU texture path\n");
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

    int width = static_cast<int>(GetDouble(it_w->second));
    int height = static_cast<int>(GetDouble(it_h->second));
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
    if (shutting_down_.load(std::memory_order_acquire)) {
      return;
    }

    const int64_t tid = texture_id_.load(std::memory_order_acquire);
    if (tid < 0) {
      return;
    }

    std::shared_ptr<NesiumGpuTexture> gpu_texture;
    std::shared_ptr<NesiumTexture> cpu_texture;
    {
      std::lock_guard<std::mutex> lk(texture_state_mu_);
      gpu_texture = gpu_texture_;
      cpu_texture = cpu_texture_;
    }

    if (gpu_texture) {
      // Resize Source if frame size changed
      if (src_width_ != static_cast<int>(width) ||
          src_height_ != static_cast<int>(height)) {
        gpu_texture->ResizeSource(static_cast<int>(width),
                                  static_cast<int>(height));
        src_width_ = static_cast<int>(width);
        src_height_ = static_cast<int>(height);
      }

      // GPU path: map, copy, unmap, commit
      auto [dst, pitch] = gpu_texture->MapWriteBuffer();
      if (dst) {
        nesium_copy_frame(bufferIndex, dst, pitch,
                          static_cast<uint32_t>(gpu_texture->height()));
        int idx_to_present = gpu_texture->UnmapAndCommit();

        if (native_window_ && idx_to_present >= 0) {
          native_window_->PresentTexture(
              gpu_texture->GetTexture(idx_to_present),
              gpu_texture->was_shader_applied());
        }
      }
    } else if (cpu_texture) {
      if (cpu_texture->width() != static_cast<int>(width) ||
          cpu_texture->height() != static_cast<int>(height)) {
        cpu_texture->Resize(static_cast<int>(width), static_cast<int>(height));
        src_width_ = static_cast<int>(width);
        src_height_ = static_cast<int>(height);
      }

      // CPU fallback path
      auto [dst, write_index] = cpu_texture->acquireWritableBuffer();
      nesium_copy_frame(bufferIndex, dst,
                        static_cast<uint32_t>(cpu_texture->stride()),
                        static_cast<uint32_t>(cpu_texture->height()));
      cpu_texture->commitLatestReady(write_index);
    }

    // Notify Flutter that the texture has a new frame.
    texture_registrar_->MarkTextureFrameAvailable(tid);
  }

public:
  void UpdateOverlayPos() {
    if (!native_window_ || !parent_hwnd_)
      return;

    native_window_->Resize(overlay_x_, overlay_y_, overlay_w_, overlay_h_);
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

  std::mutex texture_state_mu_;

  std::atomic<bool> shutting_down_{false};

  HWND parent_hwnd_ = nullptr;
  std::unique_ptr<NesiumNativeWindow> native_window_;

  // Overlay state for synchronization
  int overlay_x_ = 0;
  int overlay_y_ = 0;
  int overlay_w_ = 0;
  int overlay_h_ = 0;
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

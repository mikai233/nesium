#pragma once

#include <atomic>
#include <cstdint>
#include <d3d11.h>
#include <dxgi.h>
#include <memory>
#include <mutex>
#include <utility>
#include <windows.h>
#include <wrl/client.h>

// Flutter texture registrar header for FlutterDesktopGpuSurfaceDescriptor
#include "flutter/texture_registrar.h"

using Microsoft::WRL::ComPtr;

/// D3D11 GPU texture for low-overhead frame presentation to Flutter.
///
/// Uses double-buffered D3D11 textures with DXGI shared handles so Flutter's
/// compositor can directly sample the GPU texture. Note that if the producer
/// renders frames on CPU, an upload/copy to GPU is still required.
class NesiumGpuTexture {
public:
  /// Create a new GPU texture with the given dimensions.
  /// Returns nullptr if D3D11 initialization fails.
  static std::shared_ptr<NesiumGpuTexture> Create(int width, int height,
                                                  IDXGIAdapter *adapter = nullptr);

  ~NesiumGpuTexture();

  /// Get the back buffer for writing. The returned pointer is valid until
  /// the next call to CommitFrame().
  /// Returns {mapped_data, row_pitch} or {nullptr, 0} on failure.
  std::pair<uint8_t *, uint32_t> MapWriteBuffer();

  /// Unmap the write buffer and make it available for Flutter to read.
  void UnmapAndCommit();

  /// Recreate underlying textures with a new size.
  void Resize(int width, int height);

  /// Get the Flutter GPU surface descriptor for the current front buffer.
  /// This is called by Flutter's texture callback.
  const FlutterDesktopGpuSurfaceDescriptor *GetGpuSurface(size_t width,
                                                          size_t height);

  int width() const { return width_; }
  int height() const { return height_; }
  bool is_valid() const { return device_ != nullptr; }

private:
  class ScopedHandle {
  public:
    ScopedHandle() = default;
    explicit ScopedHandle(HANDLE handle) noexcept : handle_(handle) {}

    ~ScopedHandle() { reset(); }

    ScopedHandle(const ScopedHandle &) = delete;
    ScopedHandle &operator=(const ScopedHandle &) = delete;

    ScopedHandle(ScopedHandle &&other) noexcept
        : handle_(std::exchange(other.handle_, nullptr)) {}

    ScopedHandle &operator=(ScopedHandle &&other) noexcept {
      if (this != &other) {
        reset(std::exchange(other.handle_, nullptr));
      }
      return *this;
    }

    HANDLE get() const noexcept { return handle_; }
    explicit operator bool() const noexcept { return handle_ != nullptr; }

    void reset(HANDLE handle = nullptr) noexcept {
      if (handle_ && handle_ != INVALID_HANDLE_VALUE) {
        ::CloseHandle(handle_);
      }
      handle_ = handle;
    }

  private:
    HANDLE handle_ = nullptr;
  };

  NesiumGpuTexture(int width, int height);
  bool Initialize(IDXGIAdapter *adapter);
  bool CreateBuffersLocked();

  int width_;
  int height_;

  ComPtr<ID3D11Device> device_;
  ComPtr<ID3D11DeviceContext> context_;

  // Double-buffered textures: one for writing, one for Flutter to read.
  // staging_textures_: CPU-writable staging resources.
  // gpu_textures_: GPU-readable shared resources (opened by Flutter via handle).
  static constexpr int kBufferCount = 2;
  ComPtr<ID3D11Texture2D> staging_textures_[kBufferCount];
  ComPtr<ID3D11Texture2D> gpu_textures_[kBufferCount];
  ScopedHandle shared_handles_[kBufferCount];

  std::atomic<int> write_index_{0};
  std::atomic<int> read_index_{0};
  bool is_mapped_ = false;

  // Use unique_ptr to avoid incomplete type issue with forward declaration
  std::unique_ptr<FlutterDesktopGpuSurfaceDescriptor> descriptor_;
  std::mutex mu_;
};

#include "nesium_gpu_texture.h"
#include "nesium_rust_ffi.h"

#include <cstdio>
#include <cstring>

// Include Flutter header for FlutterDesktopGpuSurfaceDescriptor definition
#include "flutter/texture_registrar.h"

#pragma comment(lib, "d3d11.lib")
#pragma comment(lib, "dxgi.lib")

namespace {

void LogHResult(const char *step, HRESULT hr) {
  char buffer[256] = {};
  sprintf_s(buffer, "[NesiumGpuTexture] %s failed (hr=0x%08lX)\n", step,
            static_cast<unsigned long>(hr));
  ::OutputDebugStringA(buffer);
}

void LogHResultIndexed(const char *step, int index, HRESULT hr) {
  char buffer[256] = {};
  sprintf_s(buffer, "[NesiumGpuTexture] %s[%d] failed (hr=0x%08lX)\n", step,
            index, static_cast<unsigned long>(hr));
  ::OutputDebugStringA(buffer);
}

} // namespace

std::shared_ptr<NesiumGpuTexture>
NesiumGpuTexture::Create(int src_width, int src_height, int dst_width,
                         int dst_height, IDXGIAdapter *adapter) {
  auto texture = std::shared_ptr<NesiumGpuTexture>(
      new NesiumGpuTexture(src_width, src_height, dst_width, dst_height));
  if (!texture->Initialize(adapter)) {
    return {};
  }
  return texture;
}

NesiumGpuTexture::NesiumGpuTexture(int src_width, int src_height, int dst_width,
                                   int dst_height)
    : src_width_(src_width), src_height_(src_height), dst_width_(dst_width),
      dst_height_(dst_height) {
  // Allocate descriptor on heap
  descriptor_ = std::make_unique<FlutterDesktopGpuSurfaceDescriptor>();
  std::memset(descriptor_.get(), 0, sizeof(FlutterDesktopGpuSurfaceDescriptor));
  descriptor_->struct_size = sizeof(FlutterDesktopGpuSurfaceDescriptor);
}

NesiumGpuTexture::~NesiumGpuTexture() {}

bool NesiumGpuTexture::Initialize(IDXGIAdapter *adapter) {
  // Create D3D11 device
  D3D_FEATURE_LEVEL feature_levels[] = {
      D3D_FEATURE_LEVEL_11_1,
      D3D_FEATURE_LEVEL_11_0,
      D3D_FEATURE_LEVEL_10_1,
      D3D_FEATURE_LEVEL_10_0,
  };

  UINT flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT;
#ifdef _DEBUG
  flags |= D3D11_CREATE_DEVICE_DEBUG;
#endif

  D3D_FEATURE_LEVEL created_level;

  auto create_device = [&](UINT device_flags) -> HRESULT {
    if (adapter) {
      // When an adapter is provided, D3D_DRIVER_TYPE must be UNKNOWN.
      return D3D11CreateDevice(adapter, D3D_DRIVER_TYPE_UNKNOWN, nullptr,
                               device_flags, feature_levels,
                               ARRAYSIZE(feature_levels), D3D11_SDK_VERSION,
                               &device_, &created_level, &context_);
    }

    return D3D11CreateDevice(nullptr, D3D_DRIVER_TYPE_HARDWARE, nullptr,
                             device_flags, feature_levels,
                             ARRAYSIZE(feature_levels), D3D11_SDK_VERSION,
                             &device_, &created_level, &context_);
  };

  HRESULT hr = create_device(flags);
  if (FAILED(hr)) {
    // Fallback: try without debug layer.
#ifdef _DEBUG
    LogHResult("D3D11CreateDevice(with debug layer)", hr);
#endif
    hr = create_device(D3D11_CREATE_DEVICE_BGRA_SUPPORT);
  }

  if (FAILED(hr)) {
    LogHResult("D3D11CreateDevice", hr);
    return false;
  }

  std::lock_guard<std::mutex> lk(mu_);
  return CreateBuffersLocked();
}

bool NesiumGpuTexture::CreateBuffersLocked() {
  if (!device_) {
    return false;
  }

  // Reset previous resources.
  shader_texture_.Reset();
  for (int i = 0; i < kBufferCount; ++i) {
    staging_textures_[i].Reset();
    gpu_textures_[i].Reset();
    shared_handles_[i].reset();
  }

  HRESULT hr = S_OK;

  // Create double-buffered textures.
  for (int i = 0; i < kBufferCount; ++i) {
    // Staging texture: CPU writable (Source Size)
    D3D11_TEXTURE2D_DESC staging_desc = {};
    staging_desc.Width = src_width_;
    staging_desc.Height = src_height_;
    staging_desc.MipLevels = 1;
    staging_desc.ArraySize = 1;
    staging_desc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
    staging_desc.SampleDesc.Count = 1;
    staging_desc.Usage = D3D11_USAGE_STAGING;
    staging_desc.CPUAccessFlags = D3D11_CPU_ACCESS_WRITE;

    hr =
        device_->CreateTexture2D(&staging_desc, nullptr, &staging_textures_[i]);
    if (FAILED(hr)) {
      LogHResultIndexed("CreateTexture2D(staging)", i, hr);
      return false;
    }

    // GPU texture: shared with Flutter (Destination Size)
    // Must have RENDER_TARGET for ANGLE to bind it as a renderable surface
    D3D11_TEXTURE2D_DESC gpu_desc = {};
    gpu_desc.Width = dst_width_;
    gpu_desc.Height = dst_height_;
    gpu_desc.MipLevels = 1;
    gpu_desc.ArraySize = 1;
    gpu_desc.Format =
        DXGI_FORMAT_B8G8R8A8_UNORM; // Reverted to BGRA for D2D compatibility
    gpu_desc.SampleDesc.Count = 1;
    gpu_desc.Usage = D3D11_USAGE_DEFAULT;
    gpu_desc.BindFlags = D3D11_BIND_RENDER_TARGET | D3D11_BIND_SHADER_RESOURCE;
    gpu_desc.MiscFlags = D3D11_RESOURCE_MISC_SHARED;

    hr = device_->CreateTexture2D(&gpu_desc, nullptr, &gpu_textures_[i]);
    if (FAILED(hr)) {
      LogHResultIndexed("CreateTexture2D(shared gpu)", i, hr);
      return false;
    }

    // Get DXGI shared handle for Flutter
    ComPtr<IDXGIResource> dxgi_resource;
    hr = gpu_textures_[i].As(&dxgi_resource);
    if (FAILED(hr)) {
      LogHResultIndexed("QueryInterface(IDXGIResource)", i, hr);
      return false;
    }

    HANDLE shared_handle = nullptr;
    hr = dxgi_resource->GetSharedHandle(&shared_handle);
    if (FAILED(hr)) {
      LogHResultIndexed("GetSharedHandle", i, hr);
      return false;
    }
    shared_handles_[i].reset(shared_handle);
  }

  // Create intermediate shader texture (Source Size)
  D3D11_TEXTURE2D_DESC shader_desc = {};
  shader_desc.Width = src_width_;
  shader_desc.Height = src_height_;
  shader_desc.MipLevels = 1;
  shader_desc.ArraySize = 1;
  shader_desc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
  shader_desc.SampleDesc.Count = 1;
  shader_desc.Usage = D3D11_USAGE_DEFAULT;
  shader_desc.BindFlags = D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_RENDER_TARGET;

  hr = device_->CreateTexture2D(&shader_desc, nullptr, &shader_texture_);
  if (FAILED(hr)) {
    LogHResult("CreateTexture2D(shader input)", hr);
    return false;
  }

  // Pass device and context to Rust for librashader
  nesium_set_d3d11_device(device_.Get(), context_.Get());

  write_index_.store(0, std::memory_order_release);
  read_index_.store(0, std::memory_order_release);
  is_mapped_ = false;

  return true;
}

std::pair<uint8_t *, uint32_t> NesiumGpuTexture::MapWriteBuffer() {
  if (is_mapped_ || !context_) {
    return {nullptr, 0};
  }

  int idx = write_index_.load(std::memory_order_acquire);

  D3D11_MAPPED_SUBRESOURCE mapped = {};
  HRESULT hr = context_->Map(staging_textures_[idx].Get(), 0, D3D11_MAP_WRITE,
                             0, &mapped);

  if (FAILED(hr)) {
#ifdef _DEBUG
    LogHResult("Map(staging)", hr);
#endif
    return {nullptr, 0};
  }

  is_mapped_ = true;
  return {static_cast<uint8_t *>(mapped.pData),
          static_cast<uint32_t>(mapped.RowPitch)};
}

void NesiumGpuTexture::UnmapAndCommit() {
  std::lock_guard<std::mutex> lk(mu_);
  if (!is_mapped_ || !context_) {
    return;
  }

  int idx = write_index_.load(std::memory_order_acquire);

  // Unmap staging texture
  context_->Unmap(staging_textures_[idx].Get(), 0);
  is_mapped_ = false;

  // Copy staging -> Shader input texture
  if (shader_texture_) {
    context_->CopyResource(shader_texture_.Get(), staging_textures_[idx].Get());
  }

  // Try apply shader: intermediate(shader_texture) -> shared
  // output(gpu_textures)
  bool applied = false;
  if (shader_texture_ && gpu_textures_[idx] && src_width_ > 0 &&
      src_height_ > 0 && dst_width_ > 0 && dst_height_ > 0) {
    applied =
        nesium_apply_shader(shader_texture_.Get(), gpu_textures_[idx].Get(),
                            src_width_, src_height_, dst_width_, dst_height_);
  }

  if (!applied && gpu_textures_[idx]) {
    // Safety Fallback: Directly copy the source to the shared texture if the
    // shader-based blit fails. Note that this only works correctly if source
    // and destination dimensions match.
    if (src_width_ == dst_width_ && src_height_ == dst_height_) {
      if (shader_texture_) {
        context_->CopyResource(gpu_textures_[idx].Get(), shader_texture_.Get());
      } else {
        context_->CopyResource(gpu_textures_[idx].Get(),
                               staging_textures_[idx].Get());
      }
    } else {
      // Dimensions mismatch and shader blit failed. Scaling/conversion cannot
      // be performed via BitBlt. Frame will remain blank.
    }
  }

  // Flush to ensure copy/shader-render is complete before Flutter reads
  context_->Flush();

  // Swap buffers: the written buffer becomes readable
  read_index_.store(idx, std::memory_order_release);
  write_index_.store(1 - idx, std::memory_order_release);
}

void NesiumGpuTexture::ResizeSource(int width, int height) {
  std::lock_guard<std::mutex> lk(mu_);
  if (width == src_width_ && height == src_height_) {
    return;
  }
  if (!device_) {
    return;
  }

  // Best-effort: if the worker resized mid-frame, unmap so we can recreate.
  if (is_mapped_ && context_) {
    int idx = write_index_.load(std::memory_order_acquire);
    context_->Unmap(staging_textures_[idx].Get(), 0);
    is_mapped_ = false;
  }

  src_width_ = width;
  src_height_ = height;
  CreateBuffersLocked();
}

void NesiumGpuTexture::ResizeOutput(int width, int height) {
  std::lock_guard<std::mutex> lk(mu_);
  if (width == dst_width_ && height == dst_height_) {
    return;
  }
  if (!device_) {
    return;
  }

  // Best-effort: if the worker resized mid-frame, unmap so we can recreate.
  if (is_mapped_ && context_) {
    int idx = write_index_.load(std::memory_order_acquire);
    context_->Unmap(staging_textures_[idx].Get(), 0);
    is_mapped_ = false;
  }

  dst_width_ = width;
  dst_height_ = height;
  CreateBuffersLocked();
}

const FlutterDesktopGpuSurfaceDescriptor *
NesiumGpuTexture::GetGpuSurface(size_t width, size_t height) {
  if (!descriptor_) {
    return nullptr;
  }

  std::lock_guard<std::mutex> lk(mu_);
  int idx = read_index_.load(std::memory_order_acquire);

  descriptor_->handle = shared_handles_[idx].get();
  descriptor_->width = dst_width_;
  descriptor_->height = dst_height_;
  descriptor_->visible_width = dst_width_;
  descriptor_->visible_height = dst_height_;
  descriptor_->format = kFlutterDesktopPixelFormatBGRA8888;
  // Release callback not needed for persistent shared handles
  descriptor_->release_context = nullptr;
  descriptor_->release_callback = nullptr;

  return descriptor_.get();
}

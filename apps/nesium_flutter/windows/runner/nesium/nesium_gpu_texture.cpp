#include "nesium_gpu_texture.h"
#include "nesium_rust_ffi.h"

#include <algorithm>
#include <cstdio>
#include <cstring>
#include <d3dcompiler.h>

// Include Flutter header for FlutterDesktopGpuSurfaceDescriptor definition
#include "flutter/texture_registrar.h"

#pragma comment(lib, "d3d11.lib")
#pragma comment(lib, "dxgi.lib")
#pragma comment(lib, "d3dcompiler.lib")

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

const char *kSwizzleShaderSource = R"(
Texture2D<float4> bgra_input : register(t0);
RWTexture2D<float4> rgba_output : register(u0);

[numthreads(16, 16, 1)]
void main(uint3 coord : SV_DispatchThreadID) {
    uint width, height;
    rgba_output.GetDimensions(width, height);
    if (coord.x >= width || coord.y >= height) return;

    float4 color = bgra_input[coord.xy];
    // D3D11 handles format conversion (BGRA -> float4) automatically.
    // We just write it to the RGBA output, letting the hardware map logical channels.
    rgba_output[coord.xy] = color;
}
)";

} // namespace

void NesiumGpuTexture::RetireOldBufferLocked(int index) {
  if (index < 0 || index >= kBufferCount) {
    return;
  }
  if (!gpu_textures_[index] && !shared_handles_[index]) {
    return;
  }

  RetiredBuffer retired;
  retired.handle = std::move(shared_handles_[index]);
  retired.texture = std::move(gpu_textures_[index]);
  retired.retire_at_ms = ::GetTickCount64() + 2000;
  retired_.push_back(std::move(retired));
}

void NesiumGpuTexture::CleanupRetiredLocked() {
  const uint64_t now = ::GetTickCount64();
  retired_.erase(std::remove_if(retired_.begin(), retired_.end(),
                                [&](const RetiredBuffer &b) {
                                  return b.retire_at_ms <= now;
                                }),
                 retired_.end());
}

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
  adapter_ = adapter;
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

  // Ensure device is thread-safe for our multi-threaded access
  ComPtr<ID3D10Multithread> mt;
  if (SUCCEEDED(device_.As(&mt))) {
    mt->SetMultithreadProtected(TRUE);
  }

  std::lock_guard<std::mutex> lk(mu_);
  return CreateBuffersLocked();
}

bool NesiumGpuTexture::EnsureDeviceLocked() {
  if (!device_ || !context_) {
    return RecreateDeviceLocked();
  }

  HRESULT reason = device_->GetDeviceRemovedReason();
  if (reason == S_OK) {
    return true;
  }

#ifdef _DEBUG
  LogHResult("GetDeviceRemovedReason", reason);
#endif

  return RecreateDeviceLocked();
}

bool NesiumGpuTexture::RecreateDeviceLocked() {
  // Clear existing resources first; CreateBuffersLocked expects clean state.
  shader_input_bgra_.Reset();
  shader_input_rgba_.Reset();
  swizzle_srv_.Reset();
  swizzle_uav_.Reset();
  swizzle_cs_blob_.Reset();
  swizzle_shader_.Reset();

  for (int i = 0; i < kBufferCount; ++i) {
    staging_textures_[i].Reset();
    gpu_textures_[i].Reset();
    gpu_queries_[i].Reset();
    shared_handles_[i].reset();
    query_pending_[i] = false;
  }

  context_.Reset();
  device_.Reset();

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
    if (adapter_) {
      return D3D11CreateDevice(adapter_.Get(), D3D_DRIVER_TYPE_UNKNOWN, nullptr,
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
#ifdef _DEBUG
    LogHResult("D3D11CreateDevice(recreate, with debug layer)", hr);
#endif
    hr = create_device(D3D11_CREATE_DEVICE_BGRA_SUPPORT);
  }

  if (FAILED(hr)) {
    LogHResult("D3D11CreateDevice(recreate)", hr);
    return false;
  }

  ComPtr<ID3D10Multithread> mt;
  if (SUCCEEDED(device_.As(&mt))) {
    mt->SetMultithreadProtected(TRUE);
  }

  return CreateBuffersLocked();
}

bool NesiumGpuTexture::CreateBuffersLocked() {
  if (!device_) {
    return false;
  }

  CleanupRetiredLocked();

  // Reset previous resources.
  shader_input_bgra_.Reset();
  shader_input_rgba_.Reset();
  swizzle_srv_.Reset();
  swizzle_uav_.Reset();
  // Keep swizzle_cs_blob_ and swizzle_shader_ across resizes; only textures and
  // their views depend on dimensions. Recreate the shader only on device reset.

  for (int i = 0; i < kBufferCount; ++i) {
    RetireOldBufferLocked(i);
    staging_textures_[i].Reset();
    gpu_queries_[i].Reset();
    shared_handles_[i].reset();
    query_pending_[i] = false;
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
    // Pure BGRA pipeline: Staging is now BGRA (matches Core output)
    staging_desc.Format = DXGI_FORMAT_B8G8R8A8_UNORM;
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

    // Create GPU synchronization query (Event)
    D3D11_QUERY_DESC query_desc = {};
    query_desc.Query = D3D11_QUERY_EVENT;
    hr = device_->CreateQuery(&query_desc, &gpu_queries_[i]);
    if (FAILED(hr)) {
      LogHResultIndexed("CreateQuery(Event)", i, hr);
      return false;
    }
  }

  // Create intermediate shader texture (Source Size)
  // 1. BGRA Texture (Target of CPU upload)
  D3D11_TEXTURE2D_DESC bgra_desc = {};
  bgra_desc.Width = src_width_;
  bgra_desc.Height = src_height_;
  bgra_desc.MipLevels = 1;
  bgra_desc.ArraySize = 1;
  bgra_desc.Format = DXGI_FORMAT_B8G8R8A8_UNORM;
  bgra_desc.SampleDesc.Count = 1;
  bgra_desc.Usage = D3D11_USAGE_DEFAULT;
  bgra_desc.BindFlags = D3D11_BIND_SHADER_RESOURCE;

  hr = device_->CreateTexture2D(&bgra_desc, nullptr, &shader_input_bgra_);
  if (FAILED(hr)) {
    LogHResult("CreateTexture2D(shader_input_bgra)", hr);
    return false;
  }

  // 2. RGBA Texture (Target of GPU swizzle, source for librashader)
  D3D11_TEXTURE2D_DESC rgba_desc = bgra_desc;
  rgba_desc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
  rgba_desc.BindFlags = D3D11_BIND_SHADER_RESOURCE |
                        D3D11_BIND_UNORDERED_ACCESS | D3D11_BIND_RENDER_TARGET;

  hr = device_->CreateTexture2D(&rgba_desc, nullptr, &shader_input_rgba_);
  if (FAILED(hr)) {
    LogHResult("CreateTexture2D(shader_input_rgba)", hr);
    return false;
  }

  // 3. Create SRV for BGRA input
  hr = device_->CreateShaderResourceView(shader_input_bgra_.Get(), nullptr,
                                         &swizzle_srv_);
  if (FAILED(hr)) {
    LogHResult("CreateShaderResourceView(swizzle_srv)", hr);
    return false;
  }

  // 4. Create UAV for RGBA output
  hr = device_->CreateUnorderedAccessView(shader_input_rgba_.Get(), nullptr,
                                          &swizzle_uav_);
  if (FAILED(hr)) {
    LogHResult("CreateUnorderedAccessView(swizzle_uav)", hr);
    return false;
  }

  // 5. Compile (once) and create Compute Shader (per device)
  if (!swizzle_cs_blob_) {
    ComPtr<ID3DBlob> error_msg;
    hr = D3DCompile(kSwizzleShaderSource, strlen(kSwizzleShaderSource), nullptr,
                    nullptr, nullptr, "main", "cs_5_0", 0, 0, &swizzle_cs_blob_,
                    &error_msg);
    if (FAILED(hr)) {
      if (error_msg) {
        OutputDebugStringA((char *)error_msg->GetBufferPointer());
      }
      LogHResult("D3DCompile(SwizzleCS)", hr);
      return false;
    }
  }

  if (!swizzle_shader_) {
    hr = device_->CreateComputeShader(swizzle_cs_blob_->GetBufferPointer(),
                                      swizzle_cs_blob_->GetBufferSize(),
                                      nullptr, &swizzle_shader_);
    if (FAILED(hr)) {
      LogHResult("CreateComputeShader(SwizzleCS)", hr);
      return false;
    }
  }

  write_index_.store(0, std::memory_order_release);
  read_index_.store(0, std::memory_order_release);
  is_mapped_.store(false, std::memory_order_release);

  return true;
}

std::pair<uint8_t *, uint32_t> NesiumGpuTexture::MapWriteBuffer() {
  ComPtr<ID3D11DeviceContext> local_context;
  ComPtr<ID3D11Texture2D> local_staging;
  int idx = -1;

  {
    std::lock_guard<std::mutex> lk(mu_);
    if (!EnsureDeviceLocked()) {
      return {nullptr, 0};
    }

    if (is_mapped_.load(std::memory_order_acquire) || !context_) {
      return {nullptr, 0};
    }

    idx = write_index_.load(std::memory_order_acquire);

    // If the previous GPU work that referenced this buffer hasn't completed
    // yet, avoid blocking on Map. Poll the query and skip this frame if it is
    // still pending.
    if (idx >= 0 && idx < kBufferCount) {
      // Opportunistically advance read_index_ for Flutter when queries complete.
      for (int i = 0; i < kBufferCount; i++) {
        if (!query_pending_[i] || !gpu_queries_[i])
          continue;
        BOOL done = FALSE;
        HRESULT hr =
            context_->GetData(gpu_queries_[i].Get(), &done, sizeof(done),
                              D3D11_ASYNC_GETDATA_DONOTFLUSH);
        if (hr == S_OK) {
          query_pending_[i] = false;
          read_index_.store(i, std::memory_order_release);
        }
      }

      if (query_pending_[idx] && gpu_queries_[idx]) {
        BOOL done = FALSE;
        HRESULT hr =
            context_->GetData(gpu_queries_[idx].Get(), &done, sizeof(done),
                              D3D11_ASYNC_GETDATA_DONOTFLUSH);
        if (hr != S_OK) {
          return {nullptr, 0};
        }
        query_pending_[idx] = false;
      }
    }
    local_context = context_;
    local_staging = staging_textures_[idx];
  }

  if (!local_context || !local_staging) {
    return {nullptr, 0};
  }

  D3D11_MAPPED_SUBRESOURCE mapped = {};
  HRESULT hr =
      local_context->Map(local_staging.Get(), 0, D3D11_MAP_WRITE, 0, &mapped);

  if (FAILED(hr)) {
#ifdef _DEBUG
    LogHResult("Map(staging)", hr);
#endif
    return {nullptr, 0};
  }

  is_mapped_.store(true, std::memory_order_release);
  return {static_cast<uint8_t *>(mapped.pData),
          static_cast<uint32_t>(mapped.RowPitch)};
}

int NesiumGpuTexture::UnmapAndCommit() {
  ComPtr<ID3D11DeviceContext> local_context;
  ComPtr<ID3D11Texture2D> local_staging;
  ComPtr<ID3D11Texture2D> local_gpu_tex;
  ComPtr<ID3D11Query> local_query;
  int idx = -1;

  {
    std::lock_guard<std::mutex> lk(mu_);
    if (!EnsureDeviceLocked()) {
      is_mapped_.store(false, std::memory_order_release);
      return -1;
    }
    if (!is_mapped_.load(std::memory_order_acquire) || !context_) {
      return -1;
    }
    idx = write_index_.load(std::memory_order_acquire);
    local_context = context_;
    local_staging = staging_textures_[idx];
    local_gpu_tex = gpu_textures_[idx];
    local_query = gpu_queries_[idx];
  }

  // Unmap staging texture (doesn't require global lock)
  local_context->Unmap(local_staging.Get(), 0);
  is_mapped_.store(false, std::memory_order_release);

  // Copy/Shader processing
  {
    std::lock_guard<std::mutex> lk(
        mu_); // Temporarily take lock for shader textures/dims
    if (!EnsureDeviceLocked()) {
      return -1;
    }
    if (shader_input_bgra_) {
      local_context->CopyResource(shader_input_bgra_.Get(),
                                  local_staging.Get());
    }

    if (swizzle_shader_) {
      local_context->CSSetShader(swizzle_shader_.Get(), nullptr, 0);
      ID3D11ShaderResourceView *srvs[] = {swizzle_srv_.Get()};
      local_context->CSSetShaderResources(0, 1, srvs);
      ID3D11UnorderedAccessView *uavs[] = {swizzle_uav_.Get()};
      local_context->CSSetUnorderedAccessViews(0, 1, uavs, nullptr);
      local_context->Dispatch((src_width_ + 15) / 16, (src_height_ + 15) / 16,
                              1);
      local_context->CSSetShader(nullptr, nullptr, 0);
      ID3D11ShaderResourceView *null_srvs[] = {nullptr};
      local_context->CSSetShaderResources(0, 1, null_srvs);
      ID3D11UnorderedAccessView *null_uavs[] = {nullptr};
      local_context->CSSetUnorderedAccessViews(0, 1, null_uavs, nullptr);
    }

    bool applied = false;
    if (shader_input_rgba_ && local_gpu_tex && src_width_ > 0 &&
        src_height_ > 0 && dst_width_ > 0 && dst_height_ > 0) {
      applied =
          nesium_apply_shader(device_.Get(), local_context.Get(),
                              shader_input_rgba_.Get(), local_gpu_tex.Get(),
                              src_width_, src_height_, dst_width_, dst_height_);
    }

    if (!applied && local_gpu_tex) {
      if (src_width_ == dst_width_ && src_height_ == dst_height_) {
        if (shader_input_bgra_) {
          local_context->CopyResource(local_gpu_tex.Get(),
                                      shader_input_bgra_.Get());
        } else {
          local_context->CopyResource(local_gpu_tex.Get(), local_staging.Get());
        }
      }
    }
    was_shader_applied_.store(applied, std::memory_order_release);
  }

  // --- No Lock Held during GPU Sync ---
  if (local_query) {
    local_context->End(local_query.Get());
  }
  local_context->Flush();

  // Poll queries without blocking. We'll only advance the shared-texture
  // read_index_ once the GPU signals completion, preventing cross-device races
  // while keeping the app responsive during window resizing.
  {
    std::lock_guard<std::mutex> lk(mu_);
    if (!EnsureDeviceLocked()) {
      write_index_.store(1 - idx, std::memory_order_release);
      return -1;
    }

    if (idx >= 0 && idx < kBufferCount) {
      query_pending_[idx] = (local_query != nullptr);
    }

    // Always publish the latest committed buffer to Flutter. We still keep the
    // query state to reduce Map stalls during interactive resizing, but do not
    // gate presentation on query completion (which can otherwise result in
    // persistent black output when toggling the native overlay).
    read_index_.store(idx, std::memory_order_release);

    const int other = 1 - idx;
    auto poll = [&](int i) {
      if (i < 0 || i >= kBufferCount)
        return;
      if (!query_pending_[i] || !gpu_queries_[i] || !context_)
        return;
      BOOL done = FALSE;
      HRESULT hr = context_->GetData(gpu_queries_[i].Get(), &done, sizeof(done),
                                     D3D11_ASYNC_GETDATA_DONOTFLUSH);
      if (hr == S_OK) {
        query_pending_[i] = false;
        // read_index_ is already set to the latest; keep it that way.
      }
    };

    // Prefer the most recently committed buffer if both are ready.
    poll(other);
    poll(idx);
  }

  // Swap indices
  {
    std::lock_guard<std::mutex> lk(mu_);
    // Double buffering: next write is the other one
    int next_write = 1 - idx;
    write_index_.store(next_write, std::memory_order_release);
  }
  return idx;
}

void NesiumGpuTexture::ResizeSource(int width, int height) {
  std::lock_guard<std::mutex> lk(mu_);
  if (!EnsureDeviceLocked()) {
    return;
  }
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
  if (!EnsureDeviceLocked()) {
    return;
  }
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
  if (!EnsureDeviceLocked()) {
    return nullptr;
  }
  CleanupRetiredLocked();
  int idx = read_index_.load(std::memory_order_acquire);

  HANDLE handle = shared_handles_[idx].get();
  if (!handle || handle == INVALID_HANDLE_VALUE) {
    return nullptr;
  }

  descriptor_->handle = handle;
  descriptor_->width = dst_width_;
  descriptor_->height = dst_height_;
  descriptor_->visible_width = dst_width_;
  descriptor_->visible_height = dst_height_;
  descriptor_->format = kFlutterDesktopPixelFormatBGRA8888;
  descriptor_->release_context = nullptr;
  descriptor_->release_callback = nullptr;

  return descriptor_.get();
}

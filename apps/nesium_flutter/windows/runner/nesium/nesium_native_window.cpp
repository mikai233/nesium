#include "nesium_native_window.h"
#include <cstdio>
#include <d3dcompiler.h>
#include <iostream>

#pragma comment(lib, "d3dcompiler.lib")

namespace {
LRESULT CALLBACK GameWindowProc(HWND hwnd, UINT msg, WPARAM wparam,
                                LPARAM lparam) {
  if (msg == WM_NCCALCSIZE)
    return 0; // Remove non-client area
  if (msg == WM_ERASEBKGND)
    return 1; // Don't erase background
  return DefWindowProc(hwnd, msg, wparam, lparam);
}

const wchar_t *kClassName = L"NesiumGameOverlay";

const char *kVertexShaderSource = R"(
struct VS_OUTPUT {
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD0;
};
VS_OUTPUT main(uint id : SV_VertexID) {
    VS_OUTPUT vout;
    vout.tex = float2((id << 1) & 2, id & 2);
    vout.pos = float4(vout.tex * float2(2, -2) + float2(-1, 1), 0, 1);
    return vout;
}
)";

const char *kPixelShaderSource = R"(
Texture2D tex : register(t0);
SamplerState sam : register(s0);
float4 main(float4 pos : SV_POSITION, float2 uv : TEXCOORD0) : SV_TARGET {
    return tex.Sample(sam, uv);
}
)";
} // namespace

std::unique_ptr<NesiumNativeWindow>
NesiumNativeWindow::Create(HWND parent_hwnd, ID3D11Device *device) {
  static bool class_registered = false;
  if (!class_registered) {
    WNDCLASSEXW wc = {sizeof(WNDCLASSEXW)};
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = GameWindowProc;
    wc.hInstance = GetModuleHandle(nullptr);
    wc.lpszClassName = kClassName;
    wc.hCursor = LoadCursor(nullptr, IDC_ARROW);
    wc.hbrBackground = (HBRUSH)GetStockObject(BLACK_BRUSH);
    RegisterClassExW(&wc);
    class_registered = true;
  }

  // Create as CHILD window (embedded in Flutter view)
  HWND hwnd = CreateWindowExW(
      0, kClassName, L"Game", WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS, 0, 0, 1,
      1, parent_hwnd, nullptr, GetModuleHandle(nullptr), nullptr);

  if (!hwnd) {
    OutputDebugStringA("[Nesium] CreateWindowExW FAILED\n");
    return nullptr;
  }

  auto window = std::unique_ptr<NesiumNativeWindow>(
      new NesiumNativeWindow(hwnd, parent_hwnd, device));
  if (!window->CreateSwapChain()) {
    OutputDebugStringA("[Nesium] CreateSwapChain FAILED\n");
    return nullptr;
  }

  if (!window->CreateResources()) {
    OutputDebugStringA("[Nesium] CreateResources FAILED\n");
    return nullptr;
  }
  OutputDebugStringA("[Nesium] NesiumNativeWindow::Create SUCCESS\n");
  return window;
}

NesiumNativeWindow::NesiumNativeWindow(HWND hwnd, HWND parent_hwnd,
                                       ID3D11Device *device)
    : hwnd_(hwnd), parent_hwnd_(parent_hwnd), device_(device) {
  device_->GetImmediateContext(&context_);
}

NesiumNativeWindow::~NesiumNativeWindow() {
  if (hwnd_)
    DestroyWindow(hwnd_);
}

bool NesiumNativeWindow::CreateSwapChain() {
  ComPtr<IDXGIDevice> dxgi_device;
  if (FAILED(device_.As(&dxgi_device)))
    return false;

  ComPtr<IDXGIAdapter> adapter;
  if (FAILED(dxgi_device->GetAdapter(&adapter)))
    return false;

  ComPtr<IDXGIFactory2> factory;
  if (FAILED(adapter->GetParent(IID_PPV_ARGS(&factory))))
    return false;

  RECT rect;
  GetClientRect(hwnd_, &rect);
  width_ = rect.right - rect.left;
  height_ = rect.bottom - rect.top;

  // Use fixed size for SwapChain (source size)
  // and let DXGI handle the stretch to the window size.
  // Use the actual window size for backbuffer to enable custom scaling
  DXGI_SWAP_CHAIN_DESC1 desc = {};
  desc.Width = width_ > 0 ? width_ : 256;
  desc.Height = height_ > 0 ? height_ : 240;
  desc.Format = DXGI_FORMAT_B8G8R8A8_UNORM;
  desc.SampleDesc.Count = 1;
  desc.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
  desc.BufferCount = 2;
  desc.SwapEffect = DXGI_SWAP_EFFECT_FLIP_DISCARD;
  desc.Scaling = DXGI_SCALING_STRETCH;

  HRESULT hr = factory->CreateSwapChainForHwnd(device_.Get(), hwnd_, &desc,
                                               nullptr, nullptr, &swap_chain_);
  if (FAILED(hr)) {
    return false;
  }

  // Create RTV for the backdrop to render into
  ComPtr<ID3D11Texture2D> back_buffer;
  swap_chain_->GetBuffer(0, IID_PPV_ARGS(&back_buffer));
  device_->CreateRenderTargetView(back_buffer.Get(), nullptr, &rtv_);

  ClearToBlack();
  return true;
}

bool NesiumNativeWindow::CreateResources() {
  ComPtr<ID3DBlob> vs_blob;
  ComPtr<ID3DBlob> err_blob;
  HRESULT hr =
      D3DCompile(kVertexShaderSource, strlen(kVertexShaderSource), nullptr,
                 nullptr, nullptr, "main", "vs_5_0", 0, 0, &vs_blob, &err_blob);
  if (FAILED(hr)) {
    if (err_blob)
      OutputDebugStringA((char *)err_blob->GetBufferPointer());
    return false;
  }
  device_->CreateVertexShader(vs_blob->GetBufferPointer(),
                              vs_blob->GetBufferSize(), nullptr,
                              &vertex_shader_);

  ComPtr<ID3DBlob> ps_blob;
  hr =
      D3DCompile(kPixelShaderSource, strlen(kPixelShaderSource), nullptr,
                 nullptr, nullptr, "main", "ps_5_0", 0, 0, &ps_blob, &err_blob);
  if (FAILED(hr)) {
    if (err_blob)
      OutputDebugStringA((char *)err_blob->GetBufferPointer());
    return false;
  }
  device_->CreatePixelShader(ps_blob->GetBufferPointer(),
                             ps_blob->GetBufferSize(), nullptr, &pixel_shader_);

  D3D11_SAMPLER_DESC samp_desc = {};
  samp_desc.Filter = D3D11_FILTER_MIN_MAG_MIP_POINT;
  samp_desc.AddressU = D3D11_TEXTURE_ADDRESS_CLAMP;
  samp_desc.AddressV = D3D11_TEXTURE_ADDRESS_CLAMP;
  samp_desc.AddressW = D3D11_TEXTURE_ADDRESS_CLAMP;
  samp_desc.ComparisonFunc = D3D11_COMPARISON_NEVER;
  samp_desc.MinLOD = 0;
  samp_desc.MaxLOD = D3D11_FLOAT32_MAX;
  device_->CreateSamplerState(&samp_desc, &point_sampler_);

  samp_desc.Filter = D3D11_FILTER_MIN_MAG_MIP_LINEAR;
  device_->CreateSamplerState(&samp_desc, &linear_sampler_);

  return true;
}

void NesiumNativeWindow::Resize(int x, int y, int width, int height) {
  // Input x, y, width, height are PHYSICAL pixels relative to parent_hwnd_
  // (the Flutter View). Because we are now a direct child of it, we can use
  // them directly.
  int px = x;
  int py = y;
  int pw = width;
  int ph = height;

  char buf[256];
  sprintf_s(buf, "[Nesium] ResizeOverlay: view_relative(%d,%d) size(%dx%d)\n",
            px, py, pw, ph);
  OutputDebugStringA(buf);

  // Use a temporary lock for state changes
  {
    std::lock_guard<std::mutex> lk(mu_);
    // Z-Order: Place at TOP of the child list within the parent window
    // properties: SWP_NOACTIVATE prevents stealing focus.
    SetWindowPos(hwnd_, HWND_TOP, px, py, pw, ph,
                 SWP_NOACTIVATE | SWP_SHOWWINDOW);

    if (width_ != pw || height_ != ph) {
      width_ = pw;
      height_ = ph;

      if (swap_chain_) {
        // Proper cleanup before resizing/recreating to avoid Error #297
        // Lock ensures no one is presenting while we clear state.
        context_->ClearState();
        context_->Flush();
        rtv_.Reset();

        // Use the window size for backbuffer
        HRESULT hr =
            swap_chain_->ResizeBuffers(0, pw, ph, DXGI_FORMAT_UNKNOWN, 0);
        if (FAILED(hr)) {
          char err[128];
          sprintf_s(err, "[Nesium] ResizeBuffers FAILED (hr=0x%08lX)\n", hr);
          OutputDebugStringA(err);
          swap_chain_.Reset();
          CreateSwapChain();
        } else {
          ComPtr<ID3D11Texture2D> back_buffer;
          swap_chain_->GetBuffer(0, IID_PPV_ARGS(&back_buffer));
          device_->CreateRenderTargetView(back_buffer.Get(), nullptr, &rtv_);
          ClearToBlack();
        }
      } else {
        CreateSwapChain();
      }
    }
  }
}

void NesiumNativeWindow::SetVisible(bool visible) {
  ShowWindow(hwnd_, visible ? SW_SHOW : SW_HIDE);
}

bool NesiumNativeWindow::PresentTexture(ID3D11Texture2D *src_texture,
                                        bool use_linear) {
  // Lock BOTH the context usage and the swap_chain state
  std::lock_guard<std::mutex> lk(mu_);

  if (!swap_chain_ || !rtv_ || !src_texture || !context_ || !vertex_shader_ ||
      !pixel_shader_ || !point_sampler_ || !linear_sampler_)
    return false;

  // Create SRV for the source texture on the fly
  ComPtr<ID3D11ShaderResourceView> srv;
  if (FAILED(device_->CreateShaderResourceView(src_texture, nullptr, &srv)))
    return false;

  // Setup pipeline for selected sampler
  float clear_color[4] = {0, 0, 0, 1};
  context_->ClearRenderTargetView(rtv_.Get(), clear_color);

  D3D11_VIEWPORT vp = {0, 0, (float)width_, (float)height_, 0, 1};
  context_->RSSetViewports(1, &vp);
  context_->OMSetRenderTargets(1, rtv_.GetAddressOf(), nullptr);

  context_->IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
  context_->IASetInputLayout(nullptr);

  context_->VSSetShader(vertex_shader_.Get(), nullptr, 0);
  context_->PSSetShader(pixel_shader_.Get(), nullptr, 0);
  context_->PSSetShaderResources(0, 1, srv.GetAddressOf());

  ID3D11SamplerState *samplers[] = {use_linear ? linear_sampler_.Get()
                                               : point_sampler_.Get()};
  context_->PSSetSamplers(0, 1, samplers);

  // Draw full-screen quad (vertex-less)
  context_->Draw(3, 0);

  // Clean up SRV slots
  ID3D11ShaderResourceView *null_srv = nullptr;
  context_->PSSetShaderResources(0, 1, &null_srv);

  // Present with V-Sync.
  swap_chain_->Present(1, 0);
  return true;
}

void NesiumNativeWindow::ClearToBlack() {
  if (!context_ || !rtv_ || !swap_chain_)
    return;
  float clear_color[4] = {0, 0, 0, 1};
  context_->ClearRenderTargetView(rtv_.Get(), clear_color);
  swap_chain_->Present(0, 0);
}

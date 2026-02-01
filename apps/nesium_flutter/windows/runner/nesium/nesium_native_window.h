#pragma once

#include <d3d11.h>
#include <dxgi1_2.h>
#include <memory>
#include <mutex>
#include <windows.h>
#include <wrl/client.h>

using Microsoft::WRL::ComPtr;

class NesiumNativeWindow {
public:
  static std::unique_ptr<NesiumNativeWindow> Create(HWND parent_hwnd,
                                                    ID3D11Device *device);
  ~NesiumNativeWindow();

  // Must be called on the window's owning thread (Flutter UI thread).
  void SetRect(int x, int y, int width, int height);

  // Resizes the swapchain/backbuffer. Does not touch HWND geometry.
  // Safe to call from the render thread (guarded externally by d3d_context_mu_).
  void ResizeSwapChain(int width, int height);

  void Resize(int x, int y, int width, int height);
  void SetVisible(bool visible);

  // Presents a texture to this window's swapchain
  bool PresentTexture(ID3D11Texture2D *src_texture, bool use_linear);

  HWND hwnd() const { return hwnd_; }
  ID3D11Device *GetDevice() const { return device_.Get(); }

private:
  NesiumNativeWindow(HWND hwnd, HWND parent_hwnd, ID3D11Device *device);
  bool CreateSwapChain();

  HWND hwnd_ = nullptr;
  HWND parent_hwnd_ = nullptr;
  ComPtr<ID3D11Device> device_;
  ComPtr<ID3D11DeviceContext> context_;
  ComPtr<IDXGISwapChain1> swap_chain_;
  ComPtr<ID3D11RenderTargetView> rtv_;
  ComPtr<ID3D11VertexShader> vertex_shader_;
  ComPtr<ID3D11PixelShader> pixel_shader_;
  ComPtr<ID3D11SamplerState> point_sampler_;
  ComPtr<ID3D11SamplerState> linear_sampler_;

  int width_ = 0;
  int height_ = 0;
  std::mutex mu_;

  bool CreateResources();
  void ClearToBlack();
};

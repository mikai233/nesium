#include "flutter_window.h"

#include <optional>

#include "flutter/generated_plugin_registrant.h"

#include "nesium/nesium_aux_texture_plugin.h"
#include "nesium/nesium_texture_plugin.h"
#include <desktop_multi_window/desktop_multi_window_plugin.h>

FlutterWindow::FlutterWindow(const flutter::DartProject &project)
    : project_(project) {}

FlutterWindow::~FlutterWindow() {}

bool FlutterWindow::OnCreate() {
  if (!Win32Window::OnCreate()) {
    return false;
  }

  RECT frame = GetClientArea();

  // The size here must match the window dimensions to avoid unnecessary surface
  // creation / destruction in the startup path.
  flutter_controller_ = std::make_unique<flutter::FlutterViewController>(
      frame.right - frame.left, frame.bottom - frame.top, project_);
  // Ensure that basic setup of the controller was successful.
  if (!flutter_controller_->engine() || !flutter_controller_->view()) {
    return false;
  }
  RegisterPlugins(flutter_controller_->engine());
  NesiumTexturePluginRegisterWithRegistrar(
      flutter_controller_->engine()->GetRegistrarForPlugin(
          "NesiumTexturePlugin"));
  NesiumAuxTexturePluginRegisterWithRegistrar(
      flutter_controller_->engine()->GetRegistrarForPlugin(
          "NesiumAuxTexturePlugin"));
  SetChildContent(flutter_controller_->view()->GetNativeWindow());

  flutter_controller_->engine()->SetNextFrameCallback([&]() { this->Show(); });

  // Flutter can complete the first frame before the "show window" callback is
  // registered. The following call ensures a frame is pending to ensure the
  // window is shown. It is a no-op if the first frame hasn't completed yet.
  flutter_controller_->ForceRedraw();

  // Register a callback for secondary windows created by the
  // desktop_multi_window plugin.
  DesktopMultiWindowSetWindowCreatedCallback([](void *controller_ptr) {
    auto *controller =
        static_cast<flutter::FlutterViewController *>(controller_ptr);

    // 1. Register generated plugins (multi_window, file_selector, etc.) for the
    // new window's engine.
    RegisterPlugins(controller->engine());

    // 2. Register our custom Nesium-specific auxiliary texture plugin.
    // Each window (engine) must have its own plugin instance to manage its
    // local textures.
    NesiumAuxTexturePluginRegisterWithRegistrar(
        controller->engine()->GetRegistrarForPlugin("NesiumAuxTexturePlugin"));
  });

  return true;
}

void FlutterWindow::OnDestroy() {
  if (flutter_controller_) {
    flutter_controller_ = nullptr;
  }

  Win32Window::OnDestroy();
}

LRESULT
FlutterWindow::MessageHandler(HWND hwnd, UINT const message,
                              WPARAM const wparam,
                              LPARAM const lparam) noexcept {
  // Give Flutter, including plugins, an opportunity to handle window messages.
  if (flutter_controller_) {
    std::optional<LRESULT> result =
        flutter_controller_->HandleTopLevelWindowProc(hwnd, message, wparam,
                                                      lparam);
    if (result) {
      return *result;
    }
  }

  switch (message) {
  case WM_FONTCHANGE:
    flutter_controller_->engine()->ReloadSystemFonts();
    break;
  }

  return Win32Window::MessageHandler(hwnd, message, wparam, lparam);
}

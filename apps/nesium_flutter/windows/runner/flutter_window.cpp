#include "flutter_window.h"

#include <optional>

#include "flutter/generated_plugin_registrant.h"

#include "nesium/nesium_texture_plugin.h"
#include "utils.h"
#include <desktop_multi_window/desktop_multi_window_plugin.h>
#include <flutter/method_channel.h>
#include <flutter/standard_method_codec.h>

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
    auto messenger = controller->engine()->messenger();

    // Use shared_ptr instead of unique_ptr to allow lambda to be copyable
    // (required by MSVC 14.44+ for SetMethodCallHandler)
    auto channel =
        std::make_shared<flutter::MethodChannel<flutter::EncodableValue>>(
            messenger, "nesium/window",
            &flutter::StandardMethodCodec::GetInstance());

    channel->SetMethodCallHandler(
        [controller, channel](const auto &call, auto result) {
          if (call.method_name() == "setWindowTitle") {
            if (std::holds_alternative<std::string>(*call.arguments())) {
              std::string title = std::get<std::string>(*call.arguments());
              HWND hwnd = controller->view()->GetNativeWindow();
              SetWindowTextW(hwnd, Utf16FromUtf8(title).c_str());
              result->Success();
            } else {
              result->Error("INVALID_ARGUMENT", "Title must be a string");
            }
          } else {
            result->NotImplemented();
          }
        });
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

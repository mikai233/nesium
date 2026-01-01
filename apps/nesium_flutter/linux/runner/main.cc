#include "my_application.h"
#include <X11/Xlib.h>
#include <glib.h>
#include <iostream>
#include <string>

static int XErrorHandlerImpl(Display *display, XErrorEvent *event) {
  // Filter out BadAccess errors from GLX (related to context cleanup race
  // conditions) Request code 148 is typically GLX.
  if (event->error_code == BadAccess) {
    // Log but don't crash
    std::cerr << "Warning: Ignored X11 BadAccess error (likely GLX context "
                 "cleanup race)"
              << std::endl;
    return 0;
  }

  char error_text[1024];
  XGetErrorText(display, event->error_code, error_text, sizeof(error_text));
  std::cerr << "X Error: " << error_text
            << " (Request: " << (int)event->request_code
            << ", Minor: " << (int)event->minor_code << ")" << std::endl;

  return 0; // Return 0 to indicate the error is handled (don't exit)
}

// GLib log handler to suppress fatal EGL/OpenGL warnings during multi-window
// cleanup. These warnings are caused by race conditions in GTK/Flutter's
// OpenGL context cleanup and are not indicative of actual application errors.
static void GLibLogHandler(const gchar *log_domain, GLogLevelFlags log_level,
                           const gchar *message, gpointer user_data) {
  if (message != nullptr) {
    std::string msg(message);

    // Check if this is an EGL/OpenGL related warning
    bool is_egl_warning =
        (msg.find("eglMakeCurrent") != std::string::npos ||
         msg.find("cleanup compositor shaders") != std::string::npos ||
         msg.find("RemoveWindow") != std::string::npos ||
         msg.find("egl") != std::string::npos ||
         msg.find("EGL") != std::string::npos ||
         msg.find("OpenGL") != std::string::npos);

    // Suppress all WARNING level messages that are EGL-related
    // This prevents GTK from treating them as fatal errors
    if (is_egl_warning &&
        (log_level & (G_LOG_LEVEL_WARNING | G_LOG_LEVEL_CRITICAL))) {
      std::cerr << "[Suppressed " << (log_domain ? log_domain : "GLib")
                << " warning]: " << message << std::endl;
      return; // Do NOT call default handler
    }
  }

  // For other messages, use default handler
  g_log_default_handler(log_domain, log_level, message, user_data);
}

int main(int argc, char **argv) {
  // Enable X11 multi-threading support (required for multi-window apps)
  XInitThreads();

  // Install custom X11 error handler to prevent crashes from GLX errors
  XSetErrorHandler(XErrorHandlerImpl);

  // Install GLib log handler to suppress fatal EGL warnings
  // CRITICAL: Must register for ALL log domains to catch all GDK warnings
  // Register for specific domains
  g_log_set_handler("Gdk", G_LOG_LEVEL_MASK, GLibLogHandler, nullptr);
  g_log_set_handler("Gtk", G_LOG_LEVEL_MASK, GLibLogHandler, nullptr);
  g_log_set_handler("GLib", G_LOG_LEVEL_MASK, GLibLogHandler, nullptr);
  g_log_set_handler("GLib-GObject", G_LOG_LEVEL_MASK, GLibLogHandler, nullptr);
  // Also register for default domain (nullptr)
  g_log_set_handler(nullptr, G_LOG_LEVEL_MASK, GLibLogHandler, nullptr);

  g_autoptr(MyApplication) app = my_application_new();
  return g_application_run(G_APPLICATION(app), argc, argv);
}

import Cocoa
import FlutterMacOS
import desktop_multi_window

class MainFlutterWindow: NSWindow {
  private var nesiumManager: NesiumTextureManager?

  // A native splash overlay to mask the brief background transitions
  // (transparent -> black -> Flutter first frame) during startup.
  private var splashView: NSView?

  override func awakeFromNib() {
    let flutterViewController = FlutterViewController()
    let windowFrame = self.frame
    self.contentViewController = flutterViewController
    self.setFrame(windowFrame, display: true)

    // --- Native splash overlay ---
    //
    // Flutter's macOS embedder may briefly show intermediate background colors
    // while the engine boots and the first frame is rendered. We place a native
    // view on top of the content view to avoid visible flicker, then remove it
    // after Flutter notifies us that the first frame is ready.
    if let contentView = self.contentView {
      let splash = NSView(frame: contentView.bounds)
      splash.autoresizingMask = [.width, .height]
      splash.wantsLayer = true
      splash.layer?.backgroundColor = NSColor.black.cgColor

      contentView.addSubview(splash)
      self.splashView = splash

      // Fallback: if Flutter never calls `hideSplash` (channel not registered, crash before
      // first frame, etc.), do not leave the app unusable under a permanent overlay.
      // We auto-hide after a short timeout.
      DispatchQueue.main.asyncAfter(deadline: .now() + 2.0) { [weak self] in
        guard let self = self else { return }
        guard self.splashView != nil else { return }
        self.hideSplashAnimated()
      }
    }

    RegisterGeneratedPlugins(registry: flutterViewController)

    // Ensure plugins are registered for newly created secondary windows.
    FlutterMultiWindowPlugin.setOnWindowCreatedCallback { controller in
      RegisterGeneratedPlugins(registry: controller)
    }

    // Set up a MethodChannel and NesiumTextureManager to bridge the NES
    // renderer into a Flutter Texture widget on macOS.
    //
    // On macOS, texture registration is exposed via FlutterPluginRegistrar,
    // not directly on FlutterEngine. We create a registrar with a unique
    // plugin key and use its `textures` and `messenger` properties.
    let registrar = flutterViewController.registrar(forPlugin: "NesiumTexturePlugin")
    let textures = registrar.textures
    let messenger = registrar.messenger

    let channel = FlutterMethodChannel(name: "nesium", binaryMessenger: messenger)

    // Splash control channel. Flutter calls `hideSplash` after the first frame.
    let splashChannel = FlutterMethodChannel(name: "app/splash", binaryMessenger: messenger)
    splashChannel.setMethodCallHandler { [weak self] call, result in
      guard let self = self else {
        result(nil)
        return
      }

      if call.method == "hideSplash" {
        self.hideSplashAnimated()
        result(nil)
      } else {
        result(FlutterMethodNotImplemented)
      }
    }

    let nesiumManager = NesiumTextureManager(textureRegistry: textures)
    self.nesiumManager = nesiumManager

    channel.setMethodCallHandler { call, result in
      nesiumManager.handle(call: call, result: result)
    }

    super.awakeFromNib()
  }

  private func hideSplashAnimated() {
    guard let splashView = self.splashView else { return }

    // Fade out quickly, then remove from the view hierarchy.
    NSAnimationContext.runAnimationGroup(
      { ctx in
        ctx.duration = 0.18
        splashView.animator().alphaValue = 0
      },
      completionHandler: {
        splashView.removeFromSuperview()
      })

    self.splashView = nil
  }
}

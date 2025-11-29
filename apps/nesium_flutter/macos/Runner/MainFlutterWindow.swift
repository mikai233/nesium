import Cocoa
import FlutterMacOS
import desktop_multi_window

class MainFlutterWindow: NSWindow {
  private var nesiumManager: NesiumTextureManager?

  override func awakeFromNib() {
    let flutterViewController = FlutterViewController()
    let windowFrame = self.frame
    self.contentViewController = flutterViewController
    self.setFrame(windowFrame, display: true)

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
    let nesiumManager = NesiumTextureManager(textureRegistry: textures)
    self.nesiumManager = nesiumManager

    channel.setMethodCallHandler { call, result in
      nesiumManager.handle(call: call, result: result)
    }

    super.awakeFromNib()
  }
}

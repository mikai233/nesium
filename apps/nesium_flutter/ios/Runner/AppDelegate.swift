import Flutter
import UIKit

@main
@objc class AppDelegate: FlutterAppDelegate {
  private var nesiumManager: NesiumTextureManager?
  private var auxManager: NesiumAuxTextureManager?

  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    GeneratedPluginRegistrant.register(with: self)

    if let controller = window?.rootViewController as? FlutterViewController,
       let registrar = controller.registrar(forPlugin: "NesiumTexturePlugin") {
      let textures = registrar.textures()
      let messenger = registrar.messenger()
      let channel = FlutterMethodChannel(name: "nesium", binaryMessenger: messenger)

      let manager = NesiumTextureManager(textureRegistry: textures)
      nesiumManager = manager

      channel.setMethodCallHandler { call, result in
        manager.handle(call: call, result: result)
      }
      
      // --- Auxiliary Texture Manager ---
      // Separate channel for debug/tool textures (Tilemap, Pattern, etc.)
      let auxChannel = FlutterMethodChannel(name: "nesium_aux", binaryMessenger: messenger)
      let auxManager = NesiumAuxTextureManager(textureRegistry: textures)
      self.auxManager = auxManager
      auxChannel.setMethodCallHandler { call, result in
        auxManager.handle(call: call, result: result)
      }
    }

    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }
}

import Flutter
import UIKit

@main
@objc class AppDelegate: FlutterAppDelegate {
  private var nesiumManager: NesiumTextureManager?

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
    }

    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }
}

//
//  NesiumTextureManager.swift
//  Runner
//
//  Created by 时光回忆 on 2025/11/28.
//
//
//  NesiumTextureManager wires a NesiumTexture (Flutter external texture)
//  to the Flutter MethodChannel and to the Rust-based framebuffer.
//
//  The actual pixel data is produced by the `nesium-flutter` Rust crate,
//  which runs a background render loop and notifies macOS via a C callback
//  whenever a new frame is available. This manager:
//    1. Creates and registers the Flutter external texture.
//    2. Registers itself as a frame consumer with the Rust runtime.
//    3. Copies the latest frame into the CVPixelBuffer and notifies Flutter.
//

import Cocoa
import FlutterMacOS
import CoreVideo

/// Manages a NesiumTexture instance and exposes it to Flutter via a method
/// channel. It also owns the render loop that updates the pixel buffer and
/// notifies Flutter when a new frame is available.
final class NesiumTextureManager: NesiumFrameConsumer {
    private let textureRegistry: FlutterTextureRegistry

    private var texture: NesiumTexture?
    private var textureId: Int64?

    // MARK: - Initialization

    init(textureRegistry: FlutterTextureRegistry) {
        self.textureRegistry = textureRegistry
        // Register this manager as the consumer of frames produced by the Rust runtime.
        nesiumRegisterFrameCallback(for: self)
        // Optional: if the runtime is not started from Dart/FRB yet, you can
        // start it here for testing. In a production setup you may prefer to
        // start the runtime from Dart instead.
        nesium_runtime_start()
    }

    // MARK: - MethodChannel entry point

    /// Handles incoming MethodChannel calls from Dart.
    ///
    /// Expected methods:
    /// - `createNesTexture`: creates the NES texture, starts the render loop,
    ///   and returns the textureId to Flutter.
    func handle(call: FlutterMethodCall, result: @escaping FlutterResult) {
        switch call.method {
        case "createNesTexture":
            createNesTexture(result: result)
        default:
            result(FlutterMethodNotImplemented)
        }
    }

    // MARK: - Texture & render-loop setup

    private func createNesTexture(result: @escaping FlutterResult) {
        // NES resolution; keep this in sync with the Rust core.
        let width = 256
        let height = 240

        // Create the texture that will back the Flutter Texture widget.
        let tex = NesiumTexture(width: width, height: height)
        let id = textureRegistry.register(tex)

        self.texture = tex
        self.textureId = id

        // Return the texture ID to Dart so that it can construct a Texture widget.
        result(id)
    }

    // MARK: - NesiumFrameConsumer

    /// Called by the Rust runtime (via NesiumRustBridge) whenever a new frame is
    /// available in one of the internal frame buffers.
    ///
    /// The manager is responsible for copying the Rust buffer into the
    /// CVPixelBuffer backing the Flutter texture and then notifying Flutter.
    func nesiumOnFrameReady(bufferIndex: UInt32, width: Int, height: Int, pitch: Int) {
        guard let texture = self.texture,
              let textureId = self.textureId
        else {
            return
        }

        texture.withWritablePixelBuffer { pixelBuffer, _ in
            CVPixelBufferLockBaseAddress(pixelBuffer, [])
            defer { CVPixelBufferUnlockBaseAddress(pixelBuffer, []) }

            guard let baseAddress = CVPixelBufferGetBaseAddress(pixelBuffer) else {
                return
            }

            let dstBytesPerRow = CVPixelBufferGetBytesPerRow(pixelBuffer)
            let dstHeight = CVPixelBufferGetHeight(pixelBuffer)

            // Ask Rust to copy the contents of the selected frame buffer into the
            // CVPixelBuffer's backing memory.
            nesium_copy_frame(
                bufferIndex,
                baseAddress.assumingMemoryBound(to: UInt8.self),
                UInt32(dstBytesPerRow),
                UInt32(dstHeight)
            )
        }

        // Notify Flutter that the external texture has been updated.
        if Thread.isMainThread {
            textureRegistry.textureFrameAvailable(textureId)
        } else {
            DispatchQueue.main.async { [textureRegistry] in
                textureRegistry.textureFrameAvailable(textureId)
            }
        }
    }
}
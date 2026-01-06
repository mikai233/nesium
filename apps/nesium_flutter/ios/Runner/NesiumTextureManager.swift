import CoreVideo
import Flutter
import UIKit

/// Bridges Rust framebuffer updates into a Flutter external texture on iOS.
final class NesiumTextureManager: NesiumFrameConsumer {
    private let textureRegistry: FlutterTextureRegistry

    private var texture: NesiumTexture?
    private var textureId: Int64 = -1

    private let frameCopyQueue = DispatchQueue(label: "Nesium.FrameCopy", qos: .userInteractive)

    init(textureRegistry: FlutterTextureRegistry) {
        self.textureRegistry = textureRegistry
        nesiumRegisterFrameCallback(for: self)
        nesium_runtime_start()
    }

    func handle(call: FlutterMethodCall, result: @escaping FlutterResult) {
        switch call.method {
        case "createNesTexture":
            createNesTexture(result: result)
        case "disposeNesTexture":
            disposeNesTexture(result: result)
        default:
            result(FlutterMethodNotImplemented)
        }
    }

    private func createNesTexture(result: @escaping FlutterResult) {
        nesiumRegisterFrameCallback(for: self)

        let width = 256
        let height = 240

        let tex = NesiumTexture(width: width, height: height)
        let id = textureRegistry.register(tex)

        frameCopyQueue.sync {
            self.texture = tex
            self.textureId = id
        }

        result(id)
    }

    private func disposeNesTexture(result: @escaping FlutterResult) {
        nesium_set_frame_ready_callback(nil, nil)

        var tid: Int64 = -1
        frameCopyQueue.sync {
            tid = self.textureId
            self.textureId = -1
            self.texture = nil
        }

        if tid >= 0 { textureRegistry.unregisterTexture(tid) }

        result(nil)
    }

    func nesiumOnFrameReady(bufferIndex: UInt32, width: Int, height: Int, pitch: Int) {
        // Execute frame copy synchronously to minimize latency.
        // The copy operation is lightweight (~60KB memcpy) and can run on the Rust callback thread.
        frameCopyQueue.sync { [weak self] in
            self?.copyLatestFrame()
        }
    }



    private func copyLatestFrame() {
        guard let texture = self.texture else { return }
        let tid = self.textureId
        guard tid >= 0 else { return }

        guard let (pixelBuffer, writeIndex) = texture.acquireWritablePixelBuffer() else {
            return
        }

        CVPixelBufferLockBaseAddress(pixelBuffer, [])
        guard let baseAddress = CVPixelBufferGetBaseAddress(pixelBuffer) else {
            CVPixelBufferUnlockBaseAddress(pixelBuffer, [])
            return
        }

        let dstBytesPerRow = CVPixelBufferGetBytesPerRow(pixelBuffer)
        let dstHeight = CVPixelBufferGetHeight(pixelBuffer)

        nesium_copy_frame(
            0, // `bufferIndex` is informational; Rust copy API uses a safe front-copy internally.
            baseAddress.assumingMemoryBound(to: UInt8.self),
            UInt32(dstBytesPerRow),
            UInt32(dstHeight)
        )

        CVPixelBufferUnlockBaseAddress(pixelBuffer, [])

        texture.commitLatestReady(writeIndex)

        DispatchQueue.main.async { [weak self] in
            guard let self else { return }
            guard self.textureId == tid else { return }
            self.textureRegistry.textureFrameAvailable(tid)
        }
    }
}

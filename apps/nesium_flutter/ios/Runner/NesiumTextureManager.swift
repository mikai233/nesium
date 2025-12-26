import CoreVideo
import Flutter
import UIKit

/// Bridges Rust framebuffer updates into a Flutter external texture on iOS.
final class NesiumTextureManager: NesiumFrameConsumer {
    private let textureRegistry: FlutterTextureRegistry

    private var texture: NesiumTexture?
    private var textureId: Int64 = -1

    private let stateLock = NSLock()
    private var copyInFlight = false
    private var copyPending = false

    // Copying into CVPixelBuffer can be expensive; keep it off the Rust callback thread.
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
        }

        stateLock.lock()
        textureId = id
        copyInFlight = false
        copyPending = false
        stateLock.unlock()

        result(id)
    }

    private func disposeNesTexture(result: @escaping FlutterResult) {
        nesium_set_frame_ready_callback(nil, nil)

        let tid: Int64
        stateLock.lock()
        tid = textureId
        textureId = -1
        copyInFlight = false
        copyPending = false
        stateLock.unlock()
        if tid >= 0 { textureRegistry.unregisterTexture(tid) }

        frameCopyQueue.sync {
            self.texture = nil
        }

        result(nil)
    }

    func nesiumOnFrameReady(bufferIndex: UInt32, width: Int, height: Int, pitch: Int) {
        stateLock.lock()
        if copyInFlight {
            copyPending = true
            stateLock.unlock()
            return
        }
        copyInFlight = true
        stateLock.unlock()

        frameCopyQueue.async { [weak self] in
            self?.drainPendingCopies()
        }
    }

    private func drainPendingCopies() {
        while true {
            copyLatestFrame()

            stateLock.lock()
            if copyPending {
                copyPending = false
                stateLock.unlock()
                continue
            }
            copyInFlight = false
            stateLock.unlock()
            return
        }
    }

    private func copyLatestFrame() {
        guard let texture = self.texture else { return }

        let tid: Int64
        stateLock.lock()
        tid = textureId
        stateLock.unlock()
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
            self.stateLock.lock()
            let stillSame = self.textureId == tid
            self.stateLock.unlock()
            guard stillSame else { return }
            self.textureRegistry.textureFrameAvailable(tid)
        }
    }
}

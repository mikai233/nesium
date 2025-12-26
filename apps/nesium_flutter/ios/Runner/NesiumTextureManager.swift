import CoreVideo
import Flutter
import QuartzCore
import UIKit

/// Bridges Rust framebuffer updates into a Flutter external texture on iOS.
final class NesiumTextureManager: NesiumFrameConsumer {
    private let textureRegistry: FlutterTextureRegistry

    private var texture: NesiumTexture?
    private var textureId: Int64 = -1

    private let stateLock = NSLock()
    private var pendingRustBufferIndex: UInt32?
    private var copyScheduled = false
    private var frameDirty = false
    private var displayLink: CADisplayLink?

    private let frameCopyQueue = DispatchQueue(label: "Nesium.FrameCopy", qos: .userInitiated)

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

        startDisplayLinkIfNeeded()
        result(id)
    }

    private func disposeNesTexture(result: @escaping FlutterResult) {
        nesium_set_frame_ready_callback(nil, nil)

        if textureId >= 0 {
            textureRegistry.unregisterTexture(textureId)
        }

        frameCopyQueue.sync {
            self.texture = nil
            self.textureId = -1
        }

        stateLock.lock()
        pendingRustBufferIndex = nil
        copyScheduled = false
        frameDirty = false
        stateLock.unlock()

        if let link = displayLink {
            link.invalidate()
            displayLink = nil
        }

        result(nil)
    }

    func nesiumOnFrameReady(bufferIndex: UInt32, width: Int, height: Int, pitch: Int) {
        stateLock.lock()
        pendingRustBufferIndex = bufferIndex
        let shouldSchedule = !copyScheduled
        if shouldSchedule {
            copyScheduled = true
        }
        stateLock.unlock()

        guard shouldSchedule else { return }
        frameCopyQueue.async { [weak self] in
            self?.drainPendingFrames()
        }
    }

    private func drainPendingFrames() {
        var bufferIndex: UInt32?
        var texture: NesiumTexture?

        stateLock.lock()
        bufferIndex = pendingRustBufferIndex
        pendingRustBufferIndex = nil
        texture = self.texture
        stateLock.unlock()

        guard let bufferIndex, let texture else {
            stateLock.lock()
            copyScheduled = false
            stateLock.unlock()
            return
        }

        guard let (pixelBuffer, writeIndex) = texture.acquireWritablePixelBuffer() else {
            stateLock.lock()
            copyScheduled = false
            stateLock.unlock()
            return
        }

        CVPixelBufferLockBaseAddress(pixelBuffer, [])
        guard let baseAddress = CVPixelBufferGetBaseAddress(pixelBuffer) else {
            CVPixelBufferUnlockBaseAddress(pixelBuffer, [])
            stateLock.lock()
            copyScheduled = false
            stateLock.unlock()
            return
        }

        let dstBytesPerRow = CVPixelBufferGetBytesPerRow(pixelBuffer)
        let dstHeight = CVPixelBufferGetHeight(pixelBuffer)

        nesium_copy_frame(
            bufferIndex,
            baseAddress.assumingMemoryBound(to: UInt8.self),
            UInt32(dstBytesPerRow),
            UInt32(dstHeight)
        )

        CVPixelBufferUnlockBaseAddress(pixelBuffer, [])

        texture.commitLatestReady(writeIndex)

        var scheduleAgain = false
        stateLock.lock()
        frameDirty = true
        copyScheduled = false
        if pendingRustBufferIndex != nil {
            copyScheduled = true
            scheduleAgain = true
        }
        stateLock.unlock()

        if scheduleAgain {
            frameCopyQueue.async { [weak self] in
                self?.drainPendingFrames()
            }
        }
    }

    private func startDisplayLinkIfNeeded() {
        guard displayLink == nil else { return }
        let link = CADisplayLink(target: self, selector: #selector(onDisplayLink))
        link.add(to: .main, forMode: .common)
        displayLink = link
    }

    @objc private func onDisplayLink() {
        stateLock.lock()
        let shouldNotify = frameDirty
        frameDirty = false
        let tid = textureId
        stateLock.unlock()

        guard shouldNotify, tid >= 0 else { return }
        textureRegistry.textureFrameAvailable(tid)
    }
}

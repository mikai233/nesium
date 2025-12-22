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
//    2. Receives frame-ready callbacks from the Rust runtime on a background thread.
//    3. Coalesces updates (latest-only) and copies pixels on a dedicated serial queue.
//    4. Notifies Flutter on the main thread via `textureFrameAvailable`.
//

import Cocoa
import FlutterMacOS
import CoreVideo
import Atomics
import QuartzCore

/// Manages a NesiumTexture instance and exposes it to Flutter via a method
/// channel. It also owns the render loop that updates the pixel buffer and
/// notifies Flutter when a new frame is available.
final class NesiumTextureManager: NesiumFrameConsumer {
    private let textureRegistry: FlutterTextureRegistry

    // `texture` is only accessed on `frameCopyQueue`.
    private var texture: NesiumTexture?

    // Shared cross-thread state.
    //
    // - Rust callback thread publishes the latest produced framebuffer index.
    // - `frameCopyQueue` drains at most one pending index at a time (latest-only).
    // - Main thread is the only place we call `textureFrameAvailable`.
    //
    // `pendingRustBufferIndex` uses `UInt32.max` as the empty sentinel.
    private let textureId = ManagedAtomic<Int64>(-1)
    private let pendingRustBufferIndex = ManagedAtomic<UInt32>(UInt32.max)
    private let copyScheduled = ManagedAtomic<Bool>(false)
    private let frameDirty = ManagedAtomic<Bool>(false)
    private let notifyScheduled = ManagedAtomic<Bool>(false)
    private var displayLink: CVDisplayLink?

    private let frameCopyQueue = DispatchQueue(label: "Nesium.FrameCopy", qos: .userInitiated)

    // MARK: - Initialization

    init(textureRegistry: FlutterTextureRegistry) {
        self.textureRegistry = textureRegistry
        // Register this manager as the consumer of frames produced by the Rust runtime.
        nesiumRegisterFrameCallback(for: self)
        // Start the runtime here only if you don't already start it from Dart.
        // If Dart owns runtime lifecycle, remove this call and start/stop from Dart instead.
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

        // `texture` is only accessed from `frameCopyQueue`, so publish it there.
        frameCopyQueue.sync {
            self.texture = tex
            self.textureId.store(id, ordering: .releasing)
        }

        startVsyncPumpIfNeeded()

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
        // Publish the latest ready framebuffer index (latest-only). We overwrite any previous pending index.
        pendingRustBufferIndex.store(bufferIndex, ordering: .releasing)

        // Ensure only one drain is scheduled at a time; the drain loop will pick up the latest pending index.
        let scheduled = copyScheduled.compareExchange(
            expected: false,
            desired: true,
            ordering: .acquiringAndReleasing
        ).exchanged

        guard scheduled else { return }

        frameCopyQueue.async { [weak self] in
            self?.drainPendingFrames()
        }
    }

    private func drainPendingFrames() {
        let empty = UInt32.max

        let bufferIndex = pendingRustBufferIndex.exchange(empty, ordering: .acquiringAndReleasing)
        guard bufferIndex != empty else {
            copyScheduled.store(false, ordering: .releasing)
            return
        }

        guard let texture = self.texture else {
            copyScheduled.store(false, ordering: .releasing)
            return
        }

        guard let (pixelBuffer, writeIndex) = texture.acquireWritablePixelBuffer() else {
            copyScheduled.store(false, ordering: .releasing)
            return
        }

        CVPixelBufferLockBaseAddress(pixelBuffer, [])
        guard let baseAddress = CVPixelBufferGetBaseAddress(pixelBuffer) else {
            CVPixelBufferUnlockBaseAddress(pixelBuffer, [])
            copyScheduled.store(false, ordering: .releasing)
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
        frameDirty.store(true, ordering: .releasing)

        // Unschedule after one copy; if a frame arrived concurrently, schedule another drain.
        copyScheduled.store(false, ordering: .releasing)
        let hasNew = pendingRustBufferIndex.load(ordering: .acquiring) != empty
        if hasNew {
            let scheduled = copyScheduled.compareExchange(
                expected: false,
                desired: true,
                ordering: .acquiringAndReleasing
            ).exchanged
            if scheduled {
                frameCopyQueue.async { [weak self] in
                    self?.drainPendingFrames()
                }
            }
        }
    }

    private func startVsyncPumpIfNeeded() {
        guard displayLink == nil else { return }

        var link: CVDisplayLink?
        CVDisplayLinkCreateWithActiveCGDisplays(&link)
        guard let dl = link else { return }

        displayLink = dl

        let callback: CVDisplayLinkOutputCallback = { _, _, _, _, _, userInfo in
            guard let userInfo else { return kCVReturnSuccess }
            let mgr = Unmanaged<NesiumTextureManager>.fromOpaque(userInfo).takeUnretainedValue()

            let shouldNotify = mgr.frameDirty.exchange(false, ordering: .acquiringAndReleasing)
            guard shouldNotify else { return kCVReturnSuccess }

            let tid = mgr.textureId.load(ordering: .acquiring)
            guard tid >= 0 else { return kCVReturnSuccess }

            let shouldEnqueue = mgr.notifyScheduled.compareExchange(
                expected: false,
                desired: true,
                ordering: .acquiringAndReleasing
            ).exchanged
            guard shouldEnqueue else { return kCVReturnSuccess }

            DispatchQueue.main.async { [weak mgr] in
                guard let mgr else { return }
                mgr.notifyScheduled.store(false, ordering: .releasing)
                mgr.textureRegistry.textureFrameAvailable(tid)
            }

            return kCVReturnSuccess
        }

        CVDisplayLinkSetOutputCallback(dl, callback, Unmanaged.passUnretained(self).toOpaque())
        CVDisplayLinkStart(dl)
    }

    deinit {
        if let dl = displayLink {
            CVDisplayLinkStop(dl)
        }
    }
}

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
import Metal

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
    private let frameDirty = ManagedAtomic<Bool>(false)
    private let notifyScheduled = ManagedAtomic<Bool>(false)
    private var displayLink: CVDisplayLink?

    private let frameCopyQueue = DispatchQueue(label: "Nesium.FrameCopy", qos: .userInitiated)
    
    // Size requested by Flutter (via setPresentBufferSize)
    // Used as the output size for shaders to match the physical window/widget texture size.
    private var requestedWidth: Int = 0
    private var requestedHeight: Int = 0

    // Metal resources for shaders
    private let device: MTLDevice?
    private let commandQueue: MTLCommandQueue?
    private var inputTexture: MTLTexture?
    private var textureCache: CVMetalTextureCache?
    
    // Reusable buffer for frame copy to avoid per-frame allocation
    private var stagingBuffer: [UInt8] = []

    // MARK: - Initialization

    init(textureRegistry: FlutterTextureRegistry) {
        self.textureRegistry = textureRegistry
        self.device = MTLCreateSystemDefaultDevice()
        self.commandQueue = device?.makeCommandQueue()
        
        if let device = self.device {
            CVMetalTextureCacheCreate(kCFAllocatorDefault, nil, device, nil, &self.textureCache)
        }
        
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
    /// - `disposeNesTexture`: unregisters the texture and clears pending state.
    func handle(call: FlutterMethodCall, result: @escaping FlutterResult) {
        switch call.method {
        case "createNesTexture":
            createNesTexture(result: result)
        case "setPresentBufferSize":
            setPresentBufferSize(call: call, result: result)
        case "disposeNesTexture":
            disposeNesTexture(result: result)

        default:
            result(FlutterMethodNotImplemented)
        }
    }

    // MARK: - Texture & render-loop setup

    private func createNesTexture(result: @escaping FlutterResult) {
        let existing = textureId.load(ordering: .acquiring)
        if existing >= 0 {
            result(existing)
            return
        }

        nesiumRegisterFrameCallback(for: self)

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

    private func disposeNesTexture(result: @escaping FlutterResult) {
        nesium_set_frame_ready_callback(nil, nil)

        let id = textureId.load(ordering: .acquiring)
        if id >= 0 {
            textureRegistry.unregisterTexture(id)
        }

        frameCopyQueue.sync {
            self.texture = nil
            self.textureId.store(-1, ordering: .releasing)
            self.inputTexture = nil
        }

        frameDirty.store(false, ordering: .releasing)
        notifyScheduled.store(false, ordering: .releasing)

        if let dl = displayLink {
            CVDisplayLinkStop(dl)
            displayLink = nil
        }

        result(nil)
    }

    private func setPresentBufferSize(call: FlutterMethodCall, result: @escaping FlutterResult) {
        guard let args = call.arguments as? [String: Any] else {
            result(FlutterError(code: "BAD_ARGS", message: "Missing arguments", details: nil))
            return
        }
        guard let width = args["width"] as? Int, let height = args["height"] as? Int else {
            result(FlutterError(code: "BAD_ARGS", message: "Missing width/height", details: nil))
            return
        }
        if width <= 0 || height <= 0 {
            result(FlutterError(code: "BAD_ARGS", message: "width/height must be > 0", details: nil))
            return
        }
        frameCopyQueue.sync {
            self.requestedWidth = width
            self.requestedHeight = height
            self.texture?.resize(width: width, height: height)
        }
        result(nil)
    }

    // MARK: - NesiumFrameConsumer

    /// Called by the Rust runtime (via NesiumRustBridge) whenever a new frame is
    /// available in one of the internal frame buffers.
    ///
    /// The manager is responsible for copying the Rust buffer into the
    /// CVPixelBuffer backing the Flutter texture and then notifying Flutter.
    func nesiumOnFrameReady(bufferIndex: UInt32, width: Int, height: Int, pitch: Int) {
        // Execute frame copy synchronously to minimize latency.
        // The copy operation is lightweight (~60KB memcpy) and can run on the Rust callback thread.
        frameCopyQueue.sync { [weak self] in
            self?.copyFrame(bufferIndex: bufferIndex, width: width, height: height)
        }
    }

    private func copyFrame(bufferIndex: UInt32, width: Int, height: Int) {
        guard let texture = self.texture else { return }
        
        // Determine output size:
        // If Flutter requested a specific logical/physical size (via setPresentBufferSize), use it.
        // Otherwise, fall back to the NES native resolution (width/height).
        let renderWidth = requestedWidth > 0 ? requestedWidth : width
        let renderHeight = requestedHeight > 0 ? requestedHeight : height
        
        texture.resize(width: renderWidth, height: renderHeight)

        guard let (pixelBuffer, writeIndex) = texture.acquireWritablePixelBuffer() else {
            return
        }

        // Ensure input Metal texture is created and sized correctly
        if inputTexture == nil || inputTexture?.width != width || inputTexture?.height != height {
            let desc = MTLTextureDescriptor.texture2DDescriptor(pixelFormat: .bgra8Unorm, width: width, height: height, mipmapped: false)
            desc.usage = [.shaderRead, .shaderWrite]
            inputTexture = device?.makeTexture(descriptor: desc)
        }

        guard let inputTexture = self.inputTexture else { return }

        // Copy raw frame data from Rust to Metal input texture
        let region = MTLRegionMake2D(0, 0, width, height)
        let rowBytes = width * 4 // BGRA8888
        
        // Use a staging buffer to copy frame data from Rust via FFI.
        // This intermediate buffer is necessary because `nesium_copy_frame` writes to a raw pointer,
        // which we then upload to the Metal texture.
        let neededSize = rowBytes * height
        if stagingBuffer.count < neededSize {
            stagingBuffer = [UInt8](repeating: 0, count: neededSize)
        }
        
        stagingBuffer.withUnsafeMutableBytes { ptr in
            nesium_copy_frame(
                bufferIndex,
                ptr.baseAddress?.assumingMemoryBound(to: UInt8.self),
                UInt32(rowBytes),
                UInt32(height)
            )
        }
        inputTexture.replace(region: region, mipmapLevel: 0, withBytes: stagingBuffer, bytesPerRow: rowBytes)

        // Now wrap the output CVPixelBuffer in a Metal texture for librashader
        var shaderApplied = false
        
        if let textureCache = self.textureCache,
           let device = self.device,
           let commandQueue = self.commandQueue {
            
            var cvMetalTexture: CVMetalTexture?
            CVMetalTextureCacheCreateTextureFromImage(
                kCFAllocatorDefault,
                textureCache,
                pixelBuffer,
                nil,
                .bgra8Unorm,
                CVPixelBufferGetWidth(pixelBuffer),
                CVPixelBufferGetHeight(pixelBuffer),
                0,
                &cvMetalTexture
            )
            
            if let cvMetalTexture = cvMetalTexture,
               let outputMetalTexture = CVMetalTextureGetTexture(cvMetalTexture),
               let commandBuffer = commandQueue.makeCommandBuffer() {
                
                let success = nesium_apply_shader_metal(
                    Unmanaged.passUnretained(device).toOpaque(),
                    Unmanaged.passUnretained(commandQueue).toOpaque(),
                    Unmanaged.passUnretained(commandBuffer).toOpaque(),
                    Unmanaged.passUnretained(inputTexture).toOpaque(),
                    Unmanaged.passUnretained(outputMetalTexture).toOpaque(),
                    UInt32(width),
                    UInt32(height),
                    UInt32(renderWidth),
                    UInt32(renderHeight)
                )
                
                if success {
                    commandBuffer.commit()
                    shaderApplied = true
                }
            }
        }
        
        if !shaderApplied {
            // Fallback: simple copy if shader failed or resources were missing
            CVPixelBufferLockBaseAddress(pixelBuffer, [])
            if let baseAddress = CVPixelBufferGetBaseAddress(pixelBuffer) {
                let dstBytesPerRow = CVPixelBufferGetBytesPerRow(pixelBuffer)
                let dstHeight = CVPixelBufferGetHeight(pixelBuffer)
                nesium_copy_frame(
                    bufferIndex,
                    baseAddress.assumingMemoryBound(to: UInt8.self),
                    UInt32(dstBytesPerRow),
                    UInt32(dstHeight)
                )
            }
            CVPixelBufferUnlockBaseAddress(pixelBuffer, [])
        }

        texture.commitLatestReady(writeIndex)
        frameDirty.store(true, ordering: .releasing)
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

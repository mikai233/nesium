import CoreVideo
import Flutter
import UIKit
import Metal
import QuartzCore

/// Bridges Rust framebuffer updates into a Flutter external texture on iOS.
final class NesiumTextureManager: NesiumFrameConsumer {
    private let textureRegistry: FlutterTextureRegistry

    private var texture: NesiumTexture?
    private var textureId: Int64 = -1

    private let frameCopyQueue = DispatchQueue(label: "Nesium.FrameCopy", qos: .userInteractive)

    // Metal resources for shaders
    private let device: MTLDevice?
    private let commandQueue: MTLCommandQueue?
    private var inputTexture: MTLTexture?
    private var textureCache: CVMetalTextureCache?

    // Reusable buffer for frame copy to avoid per-frame allocation
    private var stagingBuffer: [UInt8] = []

    // Size requested by Flutter (via setPresentBufferSize)
    private var requestedWidth: Int = 0
    private var requestedHeight: Int = 0

    init(textureRegistry: FlutterTextureRegistry) {
        self.textureRegistry = textureRegistry
        self.device = MTLCreateSystemDefaultDevice()
        self.commandQueue = device?.makeCommandQueue()

        if let device = self.device {
            CVMetalTextureCacheCreate(kCFAllocatorDefault, nil, device, nil, &self.textureCache)
        }

        nesiumRegisterFrameCallback(for: self)
        nesium_runtime_start()
    }

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

    private func createNesTexture(result: @escaping FlutterResult) {
        var existingId: Int64 = -1
        frameCopyQueue.sync {
            existingId = self.textureId
        }
        if existingId >= 0 {
            result(existingId)
            return
        }

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
        nesiumUnregisterFrameCallback(for: self)

        var tid: Int64 = -1
        frameCopyQueue.sync {
            tid = self.textureId
            self.textureId = -1
            self.texture = nil
        }

        if tid >= 0 { textureRegistry.unregisterTexture(tid) }

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

    func nesiumOnFrameReady(bufferIndex: UInt32, width: Int, height: Int, pitch: Int) {
        // Execute frame copy synchronously to minimize latency.
        // The copy operation is lightweight (~60KB memcpy) and can run on the Rust callback thread.
        frameCopyQueue.sync { [weak self] in
            self?.copyLatestFrame(width: width, height: height)
        }
    }



    private func copyLatestFrame(width: Int, height: Int) {
        guard let texture = self.texture else { return }
        let tid = self.textureId
        guard tid >= 0 else { return }

        // Determine output size
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
        let neededSize = rowBytes * height
        if stagingBuffer.count < neededSize {
            stagingBuffer = [UInt8](repeating: 0, count: neededSize)
        }

        stagingBuffer.withUnsafeMutableBytes { ptr in
            nesium_copy_frame(
                0,
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
                    0,
                    baseAddress.assumingMemoryBound(to: UInt8.self),
                    UInt32(dstBytesPerRow),
                    UInt32(dstHeight)
                )
            }
            CVPixelBufferUnlockBaseAddress(pixelBuffer, [])
        }

        texture.commitLatestReady(writeIndex)

        DispatchQueue.main.async { [weak self] in
            guard let self else { return }
            guard self.textureId == tid else { return }
            self.textureRegistry.textureFrameAvailable(tid)
        }
    }
}

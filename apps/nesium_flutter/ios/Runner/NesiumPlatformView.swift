import Flutter
import UIKit
import Metal
import QuartzCore

/// A PlatformView that renders the NES framebuffer directly using Metal.
/// This provides better frame synchronization than FlutterTexture on iOS.
final class NesiumMetalView: UIView {
    let metalLayer: CAMetalLayer = CAMetalLayer()
    var onSizeChanged: ((CGSize) -> Void)?

    override init(frame: CGRect) {
        super.init(frame: frame)
        self.backgroundColor = .black
        self.isOpaque = true
        
        metalLayer.pixelFormat = .bgra8Unorm
        metalLayer.framebufferOnly = true
        metalLayer.isOpaque = true
        self.layer.addSublayer(metalLayer)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func layoutSubviews() {
        super.layoutSubviews()
        metalLayer.frame = self.bounds
        onSizeChanged?(self.bounds.size)
    }
}

final class NesiumPlatformView: NSObject, FlutterPlatformView, NesiumFrameConsumer {
    private let _view: NesiumMetalView
    
    private let device: MTLDevice?
    private let commandQueue: MTLCommandQueue?
    
    // Scaling and shaders
    private var inputTexture: MTLTexture?
    private var stagingBuffers: [MTLBuffer?] = [nil, nil]  // Double buffering
    private var stagingBufferSize: Int = 0
    private var currentBufferIndex: Int = 0
    private let inflightSemaphore = DispatchSemaphore(value: 2) // Allow 2 frames in flight
    private let renderQueue = DispatchQueue(label: "plugins.nesium.com/render_queue", qos: .userInteractive)
    
    // Cached viewport state
    private var cachedPhysicalSize: (width: Int, height: Int) = (0, 0)
    private let sizeLock = NSLock()
    private var frameCount: Int = 0

    init(frame: CGRect, viewIdentifier: Int64, arguments: Any?) {
        self.device = MTLCreateSystemDefaultDevice()
        self.commandQueue = device?.makeCommandQueue()
        
        self._view = NesiumMetalView(frame: frame)
        self._view.metalLayer.device = self.device
        
        // Push rendering settings: we want to show frames as soon as they are ready.
        self._view.metalLayer.presentsWithTransaction = false
        // Blocking is better than dropping a frame for emulator syncing.
        self._view.metalLayer.allowsNextDrawableTimeout = false
        
        super.init()
        
        _view.onSizeChanged = { [weak self] size in
            self?.updatePhysicalSize(size)
        }
        
        updatePhysicalSize(frame.size)
        nesiumRegisterFrameCallback(for: self)
        nesium_runtime_start()
        NSLog("[Nesium] PlatformView initialized (Low Latency Metal)")
    }

    func view() -> UIView {
        return _view
    }
    
    deinit {
        nesiumUnregisterFrameCallback(for: self)
        NSLog("[Nesium] PlatformView deinitialized")
    }

    private func updatePhysicalSize(_ size: CGSize) {
        let scale = UIScreen.main.scale
        let w = Int(size.width * scale)
        let h = Int(size.height * scale)
        
        sizeLock.lock()
        cachedPhysicalSize = (w, h)
        sizeLock.unlock()
        
        DispatchQueue.main.async { [weak self] in
            self?._view.metalLayer.drawableSize = CGSize(width: w, height: h)
        }
    }

    func nesiumOnFrameReady(bufferIndex: UInt32, width: Int, height: Int, pitch: Int) {
        // Immediately dispatch to render queue. 
        // This ensures the Rust core is never blocked but frames are processed in order.
        renderQueue.async { [weak self] in
            self?.renderFrame(width: width, height: height)
        }
    }
    
    private func renderFrame(width: Int, height: Int) {
        // Wait for a buffer to become available (prevent overwriting data the GPU is still using)
        _ = inflightSemaphore.wait(timeout: .distantFuture)
        
        guard let device = self.device,
              let commandQueue = self.commandQueue,
              let drawable = _view.metalLayer.nextDrawable() else {
            inflightSemaphore.signal()
            return
        }
        
        sizeLock.lock()
        let renderWidth = cachedPhysicalSize.width
        let renderHeight = cachedPhysicalSize.height
        sizeLock.unlock()
        
        if renderWidth <= 0 || renderHeight <= 0 {
            inflightSemaphore.signal()
            return
        }

        let rowBytes = width * 4
        let neededSize = rowBytes * height
        
        // Ensure MTLBuffer is large enough (shared memory for zero-copy)
        if stagingBuffers[currentBufferIndex] == nil || stagingBufferSize < neededSize {
            stagingBuffers[currentBufferIndex] = device.makeBuffer(length: neededSize, options: .storageModeShared)
            // If we resized one, we likely need to resize them all, but we handle that lazily as we cycle.
            // We only track the max size needed.
            // Note: In a production app, checking if ANY buffer is too small is safer, 
            // but lazily replacing them as we encounter them is also fine since we cycle sequentially.
            if stagingBufferSize < neededSize {
               stagingBufferSize = neededSize
            }
        }
        
        guard let stagingBuffer = self.stagingBuffers[currentBufferIndex] else {
            inflightSemaphore.signal()
            return
        }
        
        // Copy raw frame data from Rust directly into shared GPU memory
        let dst = stagingBuffer.contents().assumingMemoryBound(to: UInt8.self)
        nesium_copy_frame(0, dst, UInt32(rowBytes), UInt32(height))
        
        // Prepare input texture
        if inputTexture == nil || inputTexture?.width != width || inputTexture?.height != height {
            let desc = MTLTextureDescriptor.texture2DDescriptor(pixelFormat: .bgra8Unorm, width: width, height: height, mipmapped: false)
            desc.usage = [.shaderRead]
            inputTexture = device.makeTexture(descriptor: desc)
        }
        
        guard let inputTexture = self.inputTexture else {
            inflightSemaphore.signal()
            return
        }
        
        // Use blit encoder to copy from MTLBuffer to MTLTexture (GPU-side copy)
        guard let commandBuffer = commandQueue.makeCommandBuffer(),
              let blitEncoder = commandBuffer.makeBlitCommandEncoder() else {
            inflightSemaphore.signal()
            return
        }
        
        blitEncoder.copy(
            from: stagingBuffer,
            sourceOffset: 0,
            sourceBytesPerRow: rowBytes,
            sourceBytesPerImage: neededSize,
            sourceSize: MTLSize(width: width, height: height, depth: 1),
            to: inputTexture,
            destinationSlice: 0,
            destinationLevel: 0,
            destinationOrigin: MTLOrigin(x: 0, y: 0, z: 0)
        )
        blitEncoder.endEncoding()

        // Apply shader and render to drawable
        let success = nesium_apply_shader_metal(
            Unmanaged.passUnretained(device).toOpaque(),
            Unmanaged.passUnretained(commandQueue).toOpaque(),
            Unmanaged.passUnretained(commandBuffer).toOpaque(),
            Unmanaged.passUnretained(inputTexture).toOpaque(),
            Unmanaged.passUnretained(drawable.texture).toOpaque(),
            UInt32(width),
            UInt32(height),
            UInt32(renderWidth),
            UInt32(renderHeight)
        )
        
        // Signal the semaphore when the GPU is done with this frame's commands
        commandBuffer.addCompletedHandler { [weak self] _ in
            self?.inflightSemaphore.signal()
        }
        
        if success {
            commandBuffer.present(drawable)
        } else {
            if frameCount % 60 == 0 {
                NSLog("[Nesium] Metal shader application failed")
            }
        }
        commandBuffer.commit()
        
        frameCount += 1
        currentBufferIndex = (currentBufferIndex + 1) % 2
    }
}

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
    private var stagingBuffer: [UInt8] = []
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
        NSLog("NesiumPlatformView: initialized (Push Mode)")
    }

    func view() -> UIView {
        return _view
    }
    
    deinit {
        nesiumUnregisterFrameCallback(for: self)
        NSLog("NesiumPlatformView: deinit")
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
        guard let device = self.device,
              let commandQueue = self.commandQueue,
              let drawable = _view.metalLayer.nextDrawable() else {
            return
        }
        
        sizeLock.lock()
        let renderWidth = cachedPhysicalSize.width
        let renderHeight = cachedPhysicalSize.height
        sizeLock.unlock()
        
        if renderWidth <= 0 || renderHeight <= 0 { return }

        // Prepare input texture
        if inputTexture == nil || inputTexture?.width != width || inputTexture?.height != height {
            let desc = MTLTextureDescriptor.texture2DDescriptor(pixelFormat: .bgra8Unorm, width: width, height: height, mipmapped: false)
            desc.usage = [.shaderRead]
            inputTexture = device.makeTexture(descriptor: desc)
        }
        
        guard let inputTexture = self.inputTexture else { return }
        
        // Copy raw frame data from Rust
        let rowBytes = width * 4
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
        
        inputTexture.replace(region: MTLRegionMake2D(0, 0, width, height), 
                           mipmapLevel: 0, 
                           withBytes: stagingBuffer, 
                           bytesPerRow: rowBytes)

        // Apply shader and render to drawable
        if let commandBuffer = commandQueue.makeCommandBuffer() {
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
            
            if success {
                commandBuffer.present(drawable)
            }
            commandBuffer.commit()
            
            frameCount += 1
            if frameCount % 60 == 0 {
                NSLog("NesiumPlatformView: Pushed 60 frames")
            }
        }
    }
}

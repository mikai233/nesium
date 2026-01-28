import Flutter
import UIKit
import Metal
import QuartzCore

/// A PlatformView that renders the NES framebuffer directly using Metal.
/// This provides better frame synchronization than FlutterTexture on iOS.
final class NesiumPlatformView: NSObject, FlutterPlatformView, NesiumFrameConsumer {
    private let view: UIView
    private let metalLayer: CAMetalLayer
    
    private let device: MTLDevice?
    private let commandQueue: MTLCommandQueue?
    
    // Scaling and shaders
    private var inputTexture: MTLTexture?
    private var stagingBuffer: [UInt8] = []
    private let frameCopyQueue = DispatchQueue(label: "Nesium.IOS.PlatformView", qos: .userInteractive)
    
    // Viewport state
    private var viewWidth: Int = 0
    private var viewHeight: Int = 0

    init(frame: CGRect, viewIdentifier: Int64, arguments: Any?) {
        self.device = MTLCreateSystemDefaultDevice()
        self.commandQueue = device?.makeCommandQueue()
        
        self.view = UIView(frame: frame)
        self.view.backgroundColor = .black
        
        self.metalLayer = CAMetalLayer()
        self.metalLayer.device = self.device
        self.metalLayer.pixelFormat = .bgra8Unorm
        self.metalLayer.framebufferOnly = true
        self.metalLayer.frame = view.bounds
        self.view.layer.addSublayer(self.metalLayer)
        
        super.init()
        
        // Listen for frame updates from Rust
        nesiumRegisterFrameCallback(for: self)
    }

    func view() -> UIView {
        return view
    }
    
    deinit {
        nesium_set_frame_ready_callback(nil, nil)
    }

    func nesiumOnFrameReady(bufferIndex: UInt32, width: Int, height: Int, pitch: Int) {
        // Run rendering on a dedicated queue to avoid blocking Rust core
        frameCopyQueue.async { [weak self] in
            self?.renderFrame(width: width, height: height)
        }
    }
    
    private func renderFrame(width: Int, height: Int) {
        guard let device = self.device,
              let commandQueue = self.commandQueue,
              let drawable = metalLayer.nextDrawable() else {
            return
        }
        
        // Match view size if needed
        let currentBounds = DispatchQueue.main.sync { view.bounds }
        let renderWidth = Int(currentBounds.width * UIScreen.main.scale)
        let renderHeight = Int(currentBounds.height * UIScreen.main.scale)
        
        if metalLayer.drawableSize.width != CGFloat(renderWidth) || 
           metalLayer.drawableSize.height != CGFloat(renderHeight) {
            DispatchQueue.main.async {
                self.metalLayer.drawableSize = CGSize(width: renderWidth, height: renderHeight)
                self.metalLayer.frame = self.view.bounds
            }
        }

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
                commandBuffer.commit()
            }
        }
    }
}

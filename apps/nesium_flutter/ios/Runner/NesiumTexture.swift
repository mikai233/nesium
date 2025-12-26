import CoreVideo
import Flutter

/// A FlutterTexture backed by a pair of CVPixelBuffers (double buffering).
final class NesiumTexture: NSObject, FlutterTexture {
    let width: Int
    let height: Int

    private var pixelBuffers: [CVPixelBuffer] = []
    private let lock = NSLock()
    private var latestReadyIndex = 0

    init(width: Int, height: Int) {
        self.width = width
        self.height = height
        super.init()

        if let pb0 = NesiumTexture.makePixelBuffer(width: width, height: height),
           let pb1 = NesiumTexture.makePixelBuffer(width: width, height: height) {
            self.pixelBuffers = [pb0, pb1]
        } else {
            NSLog("NesiumTexture(iOS): failed to create CVPixelBuffer(s) for %dx%d", width, height)
        }
    }

    private static func makePixelBuffer(width: Int, height: Int) -> CVPixelBuffer? {
        var pb: CVPixelBuffer?
        let attrs: [CFString: Any] = [
            kCVPixelBufferCGImageCompatibilityKey: true,
            kCVPixelBufferCGBitmapContextCompatibilityKey: true,
            kCVPixelBufferMetalCompatibilityKey: true,
            kCVPixelBufferIOSurfacePropertiesKey: [:] as CFDictionary,
        ]

        let status = CVPixelBufferCreate(
            kCFAllocatorDefault,
            width,
            height,
            kCVPixelFormatType_32BGRA,
            attrs as CFDictionary,
            &pb
        )

        guard status == kCVReturnSuccess else {
            return nil
        }

        return pb
    }

    func acquireWritablePixelBuffer() -> (CVPixelBuffer, Int)? {
        lock.lock()
        defer { lock.unlock() }
        guard pixelBuffers.count == 2 else { return nil }

        let nextIndex = 1 - latestReadyIndex
        return (pixelBuffers[nextIndex], nextIndex)
    }

    func commitLatestReady(_ index: Int) {
        lock.lock()
        latestReadyIndex = index
        lock.unlock()
    }

    func copyPixelBuffer() -> Unmanaged<CVPixelBuffer>? {
        lock.lock()
        defer { lock.unlock() }
        guard pixelBuffers.count == 2 else { return nil }

        let pb = pixelBuffers[latestReadyIndex]
        return Unmanaged.passRetained(pb)
    }
}

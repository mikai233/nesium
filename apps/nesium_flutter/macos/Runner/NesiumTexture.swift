//
//  NesiumTexture.swift
//  Runner
//
//  Created by 时光回忆 on 2025/11/28.
//

import Cocoa
import FlutterMacOS
import CoreVideo
import Atomics

/// A FlutterTexture implementation backed by a CVPixelBuffer.
///
/// NesiumTexture owns a pair of pixel buffers (double buffering) that NES frames
/// are copied into. Flutter will call `copyPixelBuffer()` whenever it needs to
/// composite the current frame into the UI.
final class NesiumTexture: NSObject, FlutterTexture {
    let width: Int
    let height: Int

    /// Double buffering: we hold 2 buffers.
    private var pixelBuffers: [CVPixelBuffer] = []

    /// Atomic index indicating which buffer contains the latest fully written frame.
    /// 0 or 1.
    private let latestReadyIndex = ManagedAtomic<Int32>(0)

    init(width: Int, height: Int) {
        self.width = width
        self.height = height
        super.init()
        
        if let pb0 = NesiumTexture.makePixelBuffer(width: width, height: height),
           let pb1 = NesiumTexture.makePixelBuffer(width: width, height: height) {
            self.pixelBuffers = [pb0, pb1]
        } else {
            NSLog("NesiumTexture: failed to create CVPixelBuffer(s) for %dx%d", width, height)
        }
    }

    /// Creates a CVPixelBuffer suitable for use as a Flutter external texture.
    private static func makePixelBuffer(width: Int, height: Int) -> CVPixelBuffer? {
        var pb: CVPixelBuffer?
        let attrs: [CFString: Any] = [
            kCVPixelBufferCGImageCompatibilityKey: true,
            kCVPixelBufferCGBitmapContextCompatibilityKey: true,
            // Required so that Flutter can wrap this CVPixelBuffer in a Metal texture.
            kCVPixelBufferMetalCompatibilityKey: true,
            // Required on macOS to back the pixel buffer with an IOSurface, which
            // is what the Flutter engine expects for external textures.
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

    // MARK: - Writer Interface

    /// Executes the given closure with a writable pixel buffer.
    ///
    /// This method identifies the back buffer (the one not currently marked as latest),
    /// yields it to the closure for writing, and then atomically updates `latestReadyIndex`
    /// to point to this new buffer.
    func acquireWritablePixelBuffer() -> (CVPixelBuffer, Int)? {
        guard pixelBuffers.count == 2 else { return nil }

        let current = Int(latestReadyIndex.load(ordering: .acquiring))
        let nextIndex = 1 - current
        return (pixelBuffers[nextIndex], nextIndex)
    }

    func commitLatestReady(_ index: Int) {
        latestReadyIndex.store(Int32(index), ordering: .releasing)
    }

    // MARK: - FlutterTexture

    /// Called by Flutter to obtain the current frame for this texture.
    func copyPixelBuffer() -> Unmanaged<CVPixelBuffer>? {
        guard pixelBuffers.count == 2 else {
            return nil
        }
        
        // Acquire latest ready index
        let idx = Int(latestReadyIndex.load(ordering: .acquiring))
        let pb = pixelBuffers[idx]
        
        return Unmanaged.passRetained(pb)
    }
}

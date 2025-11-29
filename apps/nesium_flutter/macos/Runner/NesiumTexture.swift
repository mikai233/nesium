//
//  NesiumTexture.swift
//  Runner
//
//  Created by 时光回忆 on 2025/11/28.
//

import Cocoa
import FlutterMacOS
import CoreVideo

/// A FlutterTexture implementation backed by a CVPixelBuffer.
///
/// NesiumTexture owns a single pixel buffer that NES frames are copied into.
/// Flutter will call `copyPixelBuffer()` whenever it needs to composite the
/// current frame into the UI. The actual pixel data is updated externally
/// (e.g. by NesiumTextureManager) before calling
/// `textureRegistry.textureFrameAvailable(textureId)`.
final class NesiumTexture: NSObject, FlutterTexture {
    let width: Int
    let height: Int

    /// The underlying pixel buffer that holds the current NES frame.
    ///
    /// This buffer is expected to use a 32-bit BGRA pixel format
    /// (kCVPixelFormatType_32BGRA) to match the emulator's ColorFormat,
    /// so that we can memcpy the raw bytes directly into it.
    private(set) var pixelBuffer: CVPixelBuffer?

    init(width: Int, height: Int) {
        self.width = width
        self.height = height
        super.init()
        self.pixelBuffer = NesiumTexture.makePixelBuffer(width: width, height: height)
    }

    /// Creates a CVPixelBuffer suitable for use as a Flutter external texture.
    ///
    /// The pixel format must match the bytes written by the emulator. If the
    /// NES core uses ColorFormat::Bgra8888, we should use BGRA here so the
    /// memory layout is [B, G, R, A] per pixel.
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
            NSLog("NesiumTexture: failed to create CVPixelBuffer (status=\(status))")
            return nil
        }

        return pb
    }

    // MARK: - FlutterTexture

    /// Called by Flutter to obtain the current frame for this texture.
    ///
    /// We simply return the retained CVPixelBuffer that is being updated by
    /// the NES rendering loop. Flutter will release it when it is done.
    func copyPixelBuffer() -> Unmanaged<CVPixelBuffer>? {
        guard let pb = pixelBuffer else {
            return nil
        }
        return Unmanaged.passRetained(pb)
    }
}

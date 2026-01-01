//
//  NesiumAuxTextureManager.swift
//  Runner (iOS)
//
//  Manages auxiliary Flutter textures (Tilemap, Pattern, etc.)
//  completely separate from the main NES screen rendering.
//

import UIKit
import Flutter
import CoreVideo

// MARK: - C ABI declarations

@_silgen_name("nesium_aux_create")
func nesium_aux_create(_ id: UInt32, _ width: UInt32, _ height: UInt32)

@_silgen_name("nesium_aux_destroy")
func nesium_aux_destroy(_ id: UInt32)

@_silgen_name("nesium_aux_copy")
func nesium_aux_copy(_ id: UInt32, _ dst: UnsafeMutablePointer<UInt8>, _ dstPitch: UInt32, _ dstHeight: UInt32) -> Int

// MARK: - Auxiliary Texture Entry

/// Represents one auxiliary texture registered with Flutter.
final class AuxTextureEntry: NSObject, FlutterTexture {
    let id: UInt32
    let width: Int
    let height: Int
    
    private var pixelBuffers: [CVPixelBuffer] = []
    private let indexLock = NSLock()
    private var latestReadyIndex: Int32 = 0
    
    init(id: UInt32, width: Int, height: Int) {
        self.id = id
        self.width = width
        self.height = height
        super.init()
        
        if let pb0 = Self.makePixelBuffer(width: width, height: height),
           let pb1 = Self.makePixelBuffer(width: width, height: height) {
            self.pixelBuffers = [pb0, pb1]
        } else {
            NSLog("AuxTextureEntry: failed to create CVPixelBuffer(s) for %dx%d", width, height)
        }
        
        // Create the Rust-side backing store.
        nesium_aux_create(id, UInt32(width), UInt32(height))
    }
    
    deinit {
        nesium_aux_destroy(id)
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
        
        return status == kCVReturnSuccess ? pb : nil
    }
    
    /// Copies from Rust buffer into the back CVPixelBuffer and commits.
    func updateFromRust() {
        guard pixelBuffers.count == 2 else { return }
        
        indexLock.lock()
        let current = Int(latestReadyIndex)
        indexLock.unlock()
        
        let nextIndex = 1 - current
        let pixelBuffer = pixelBuffers[nextIndex]
        
        CVPixelBufferLockBaseAddress(pixelBuffer, [])
        defer { CVPixelBufferUnlockBaseAddress(pixelBuffer, []) }
        
        guard let baseAddress = CVPixelBufferGetBaseAddress(pixelBuffer) else { return }
        
        let dstBytesPerRow = CVPixelBufferGetBytesPerRow(pixelBuffer)
        let dstHeight = CVPixelBufferGetHeight(pixelBuffer)
        
        _ = nesium_aux_copy(
            id,
            baseAddress.assumingMemoryBound(to: UInt8.self),
            UInt32(dstBytesPerRow),
            UInt32(dstHeight)
        )
        
        indexLock.lock()
        latestReadyIndex = Int32(nextIndex)
        indexLock.unlock()
    }
    
    // MARK: - FlutterTexture
    
    func copyPixelBuffer() -> Unmanaged<CVPixelBuffer>? {
        guard pixelBuffers.count == 2 else { return nil }
        
        indexLock.lock()
        let idx = Int(latestReadyIndex)
        indexLock.unlock()
        
        return Unmanaged.passRetained(pixelBuffers[idx])
    }
}

// MARK: - Manager

/// Manages all auxiliary textures and exposes them via MethodChannel.
final class NesiumAuxTextureManager {
    private let textureRegistry: FlutterTextureRegistry
    
    /// Map from aux texture ID to (FlutterTextureId, AuxTextureEntry).
    private var textures: [UInt32: (flutterTextureId: Int64, entry: AuxTextureEntry)] = [:]
    
    private var displayLink: CADisplayLink?
    private let updateQueue = DispatchQueue(label: "Nesium.AuxTexture", qos: .userInteractive)
    
    init(textureRegistry: FlutterTextureRegistry) {
        self.textureRegistry = textureRegistry
    }
    
    func handle(call: FlutterMethodCall, result: @escaping FlutterResult) {
        switch call.method {
        case "createAuxTexture":
            guard let args = call.arguments as? [String: Any],
                  let id = args["id"] as? Int,
                  let width = args["width"] as? Int,
                  let height = args["height"] as? Int else {
                result(FlutterError(code: "BAD_ARGS", message: "Missing id/width/height", details: nil))
                return
            }
            createAuxTexture(id: UInt32(id), width: width, height: height, result: result)
            
        case "disposeAuxTexture":
            guard let args = call.arguments as? [String: Any],
                  let id = args["id"] as? Int else {
                result(FlutterError(code: "BAD_ARGS", message: "Missing id", details: nil))
                return
            }
            disposeAuxTexture(id: UInt32(id), result: result)
            
        default:
            result(FlutterMethodNotImplemented)
        }
    }
    
    private func createAuxTexture(id: UInt32, width: Int, height: Int, result: @escaping FlutterResult) {
        // Clean up any existing texture with this ID.
        if let existing = textures[id] {
            textureRegistry.unregisterTexture(existing.flutterTextureId)
        }
        
        let entry = AuxTextureEntry(id: id, width: width, height: height)
        let flutterTextureId = textureRegistry.register(entry)
        textures[id] = (flutterTextureId, entry)
        
        startDisplayLinkIfNeeded()
        
        result(flutterTextureId)
    }
    
    private func disposeAuxTexture(id: UInt32, result: @escaping FlutterResult) {
        if let existing = textures.removeValue(forKey: id) {
            textureRegistry.unregisterTexture(existing.flutterTextureId)
        }
        
        if textures.isEmpty {
            stopDisplayLink()
        }
        
        result(nil)
    }
    
    // MARK: - Display Link
    
    private func startDisplayLinkIfNeeded() {
        guard displayLink == nil else { return }
        
        let link = CADisplayLink(target: self, selector: #selector(onDisplayLinkTick))
        link.add(to: .current, forMode: .common)
        displayLink = link
    }
    
    private func stopDisplayLink() {
        displayLink?.invalidate()
        displayLink = nil
    }
    
    @objc private func onDisplayLinkTick() {
        updateQueue.async { [weak self] in
            self?.updateAllTextures()
        }
    }
    
    private func updateAllTextures() {
        for (_, (flutterTextureId, entry)) in textures {
            entry.updateFromRust()
            DispatchQueue.main.async { [weak self] in
                self?.textureRegistry.textureFrameAvailable(flutterTextureId)
            }
        }
    }
    
    deinit {
        stopDisplayLink()
    }
}

//
//  NesiumRustBridge.swift
//  Runner
//
//  Created by 时光回忆 on 2025/11/28.
//

import Foundation
import FlutterMacOS

/// C-ABI callback signature used by the Rust runtime.
///
/// Rust will call this function pointer after finishing a frame, passing:
/// - bufferIndex: which of the internal frame buffers (0 or 1) now holds
///   the freshly rendered BGRA8888 image
/// - width / height: logical size in pixels
/// - pitch: number of bytes per row in the internal buffer
/// - userData: opaque pointer round-tripped from Swift (used to recover the owner)
typealias NesiumFrameReadyCallback = @convention(c) (
    UInt32,  // bufferIndex
    UInt32,  // width
    UInt32,  // height
    UInt32,  // pitch
    UnsafeMutableRawPointer? // userData
) -> Void

// MARK: - FFI entry points exposed by the nesium-flutter Rust crate

@_silgen_name("nesium_runtime_start")
func nesium_runtime_start()

@_silgen_name("nesium_set_frame_ready_callback")
func nesium_set_frame_ready_callback(
    _ cb: NesiumFrameReadyCallback?,
    _ userData: UnsafeMutableRawPointer?
)

@_silgen_name("nesium_copy_frame")
func nesium_copy_frame(
    _ bufferIndex: UInt32,
    _ dst: UnsafeMutablePointer<UInt8>?,
    _ dstPitch: UInt32,
    _ dstHeight: UInt32
)

// MARK: - High-level Swift bridge

/// Protocol that any Swift object can adopt if it wants to receive frame
/// notifications from the Rust runtime.
///
/// For example, `NesiumTextureManager` can conform to this protocol and use
/// `nesiumCopyFrame()` to blit the contents into a `CVPixelBuffer`.
protocol NesiumFrameConsumer: AnyObject {
    func nesiumOnFrameReady(bufferIndex: UInt32, width: Int, height: Int, pitch: Int)
}

/// A single global C callback that Rust will call from its render thread.
///
/// The `userData` pointer is an opaque handle that we set up when calling
/// `nesium_set_frame_ready_callback`. We treat it as an `AnyObject` that
/// conforms to `NesiumFrameConsumer`, then bounce the event back to the main
/// thread for actual UI / texture updates.
private let globalFrameReadyCallback: NesiumFrameReadyCallback = { bufferIndex, width, height, pitch, userData in
    guard let userData = userData else { return }
    
    let anyObject = Unmanaged<AnyObject>
        .fromOpaque(userData)
        .takeUnretainedValue()
    
    guard let consumer = anyObject as? NesiumFrameConsumer else {
        return
    }
    
    // The Rust callback is likely invoked on a background render thread.
    // Always hop back to the main queue before touching AppKit / Flutter.
    DispatchQueue.main.async {
        consumer.nesiumOnFrameReady(
            bufferIndex: bufferIndex,
            width: Int(width),
            height: Int(height),
            pitch: Int(pitch)
        )
    }
}

/// Registers the global frame-ready callback for a given consumer.
///
/// Typical usage:
///
/// ```swift
/// final class NesiumTextureManager: NesiumFrameConsumer {
///     init(registry: FlutterTextureRegistry) {
///         nesiumRegisterFrameCallback(for: self)
///     }
///
///     func nesiumOnFrameReady(bufferIndex: UInt32, width: Int, height: Int, pitch: Int) {
///         // Call `nesium_copy_frame(...)` here and update the Flutter texture.
///     }
/// }
/// ```
func nesiumRegisterFrameCallback(for consumer: NesiumFrameConsumer) {
    let anyObject = consumer as AnyObject
    let userData = UnsafeMutableRawPointer(
        Unmanaged.passUnretained(anyObject).toOpaque()
    )
    nesium_set_frame_ready_callback(globalFrameReadyCallback, userData)
}

import Foundation

/// C-ABI callback signature used by the Rust runtime.
typealias NesiumFrameReadyCallback = @convention(c) (
    UInt32,  // bufferIndex
    UInt32,  // width
    UInt32,  // height
    UInt32,  // pitch
    UnsafeMutableRawPointer?
) -> Void

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

@_silgen_name("nesium_apply_shader_metal")
func nesium_apply_shader_metal(
    _ device: UnsafeMutableRawPointer,
    _ commandQueue: UnsafeMutableRawPointer,
    _ commandBuffer: UnsafeMutableRawPointer,
    _ inputTex: UnsafeMutableRawPointer,
    _ outputTex: UnsafeMutableRawPointer,
    _ srcWidth: UInt32,
    _ srcHeight: UInt32,
    _ dstWidth: UInt32,
    _ dstHeight: UInt32
) -> Bool

protocol NesiumFrameConsumer: AnyObject {
    func nesiumOnFrameReady(bufferIndex: UInt32, width: Int, height: Int, pitch: Int)
}

private let globalFrameReadyCallback: NesiumFrameReadyCallback = { bufferIndex, width, height, pitch, userData in
    guard let userData = userData else { return }

    let anyObject = Unmanaged<AnyObject>
        .fromOpaque(userData)
        .takeUnretainedValue()

    guard let consumer = anyObject as? NesiumFrameConsumer else {
        return
    }

    consumer.nesiumOnFrameReady(
        bufferIndex: bufferIndex,
        width: Int(width),
        height: Int(height),
        pitch: Int(pitch)
    )
}

func nesiumRegisterFrameCallback(for consumer: NesiumFrameConsumer) {
    let anyObject = consumer as AnyObject
    let userData = UnsafeMutableRawPointer(
        Unmanaged.passUnretained(anyObject).toOpaque()
    )
    nesium_set_frame_ready_callback(globalFrameReadyCallback, userData)
}

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

/// A central coordinator that multiplexes the single Rust frame callback to multiple observers.
/// This allows both the legacy FlutterTexture manager and new PlatformViews to co-exist.
final class NesiumFrameCoordinator {
    static let shared = NesiumFrameCoordinator()
    
    private let consumers = NSHashTable<AnyObject>.weakObjects()
    private let lock = NSLock()

    private init() {
        let userData = UnsafeMutableRawPointer(Unmanaged.passUnretained(self).toOpaque())
        nesium_set_frame_ready_callback(globalFrameReadyCallback, userData)
    }

    func register(_ consumer: NesiumFrameConsumer) {
        lock.lock()
        defer { lock.unlock() }
        consumers.add(consumer)
    }

    func unregister(_ consumer: NesiumFrameConsumer) {
        lock.lock()
        defer { lock.unlock() }
        consumers.remove(consumer)
    }

    fileprivate func broadcast(bufferIndex: UInt32, width: Int, height: Int, pitch: Int) {
        lock.lock()
        let observers = consumers.allObjects.compactMap { $0 as? NesiumFrameConsumer }
        lock.unlock()

        for observer in observers {
            observer.nesiumOnFrameReady(
                bufferIndex: bufferIndex,
                width: width,
                height: height,
                pitch: pitch
            )
        }
    }
}

private let globalFrameReadyCallback: NesiumFrameReadyCallback = { bufferIndex, width, height, pitch, userData in
    guard let userData = userData else { return }

    let coordinator = Unmanaged<NesiumFrameCoordinator>
        .fromOpaque(userData)
        .takeUnretainedValue()

    coordinator.broadcast(
        bufferIndex: bufferIndex,
        width: Int(width),
        height: Int(height),
        pitch: Int(pitch)
    )
}

func nesiumRegisterFrameCallback(for consumer: NesiumFrameConsumer) {
    NesiumFrameCoordinator.shared.register(consumer)
}

func nesiumUnregisterFrameCallback(for consumer: NesiumFrameConsumer) {
    NesiumFrameCoordinator.shared.unregister(consumer)
}

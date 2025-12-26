package io.github.mikai233.nesium

import android.content.Context
import java.nio.ByteBuffer

/**
 * JNI bridge to the Rust core.
 *
 * The Rust side should export matching JNI symbols for this object, e.g.
 * `Java_io_github_mikai233_nesium_NesiumNative_nativeFrameSeq`.
 */
object NesiumNative {
    init {
        // Ensure the Rust shared library is loaded before calling any native methods.
        System.loadLibrary("nesium_flutter")
    }

    @JvmStatic
    external fun init_android_context(context: Context)

    @JvmStatic
    external fun nativeFrameSeq(): Long

    @JvmStatic
    external fun nativeBeginFrontCopy(): Int

    /**
     * Returns a direct ByteBuffer backed by the current plane memory.
     * The Rust side should create it via `NewDirectByteBuffer`.
     */
    @JvmStatic
    external fun nativePlaneBuffer(idx: Int): ByteBuffer

    @JvmStatic
    external fun nativeEndFrontCopy()

    @JvmStatic
    external fun nativeFrameWidth(): Int

    @JvmStatic
    external fun nativeFrameHeight(): Int

    @JvmStatic
    external fun nativeSetFrameSignalFd(fd: Int)
}

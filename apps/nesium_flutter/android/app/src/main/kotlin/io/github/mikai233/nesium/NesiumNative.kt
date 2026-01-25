package io.github.mikai233.nesium

import android.content.Context
import android.view.Surface
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
        nativeInitLogger()
    }

    @JvmStatic
    external fun nativeInitLogger()

    @JvmStatic
    external fun init_android_context(context: Context)

    /**
     * Selects the Android video backend.
     *
     * Must be called before [init_android_context] triggers runtime initialization.
     *
     * 0 = Upload (Kotlin GL uploader)
     * 1 = AHardwareBuffer swapchain + Rust EGL/GL renderer
     */
    @JvmStatic
    external fun nativeSetVideoBackend(mode: Int)

    /**
     * Enables/disables best-effort thread priority boost on Android.
     *
     * This affects the Rust runtime thread and (when running) the Rust renderer thread.
     */
    @JvmStatic
    external fun nativeSetHighPriority(enabled: Boolean)

    /**
     * Enables/disables the Rust-side librashader filter chain (AHB backend).
     */
    @JvmStatic
    external fun nativeSetShaderEnabled(enabled: Boolean)

    /**
     * Sets the shader preset path for the Rust-side librashader filter chain (AHB backend).
     *
     * The path must be readable by native code (typically under app-private storage).
     * Pass an empty string to clear.
     */
    @JvmStatic
    external fun nativeSetShaderPreset(path: String)

    /**
     * Starts the Rust-side EGL/GL renderer that presents into the given [Surface].
     *
     * Only used when `nativeSetVideoBackend(1)` was selected.
     */
    @JvmStatic
    external fun nativeStartRustRenderer(surface: Surface)

    /**
     * Stops the Rust-side EGL/GL renderer (if running).
     */
    @JvmStatic
    external fun nativeStopRustRenderer()

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

    // --- Auxiliary Texture API ---
    // Maps to Rust's nesium_aux_create/copy/destroy C ABI

    @JvmStatic
    external fun nesiumAuxCreate(id: Int, width: Int, height: Int)

    @JvmStatic
    external fun nesiumAuxCopy(id: Int, dst: ByteBuffer, dstPitch: Int, dstHeight: Int): Int

    @JvmStatic
    external fun nesiumAuxDestroy(id: Int)
}

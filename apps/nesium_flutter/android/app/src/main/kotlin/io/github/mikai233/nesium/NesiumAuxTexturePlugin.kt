package io.github.mikai233.nesium

import android.os.Handler
import android.os.HandlerThread
import android.util.Log
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import io.flutter.view.TextureRegistry
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

/**
 * Manages auxiliary textures for debugger views (Tilemap, Pattern, etc.) on Android.
 *
 * Uses SurfaceTexture-based external textures similar to the main NES texture,
 * but receives data from the Rust aux_texture module.
 */
class NesiumAuxTexturePlugin(private val flutterEngine: FlutterEngine) {

    companion object {
        private const val TAG = "NesiumAuxTexture"
        private const val CHANNEL_NAME = "nesium_aux"
        private const val UPDATE_INTERVAL_MS = 16L // ~60Hz
        private const val DISPOSE_TIMEOUT_MS = 1000L
    }

    /** Unified texture state that tracks both entry and pause status */
    private data class TextureState(
        val entry: AuxTextureEntry,
        var paused: Boolean = false
    )

    private val textures = mutableMapOf<Int, TextureState>()
    private val texturesLock = Any()
    private val updateThread = HandlerThread("NesiumAuxTexture").apply { start() }
    private val updateHandler = Handler(updateThread.looper)

    private val updateRunnable = object : Runnable {
        override fun run() {
            synchronized(texturesLock) {
                textures.forEach { (_, state) ->
                    if (!state.paused) {
                        state.entry.updateFromRust()
                    }
                }
            }
            updateHandler.postDelayed(this, UPDATE_INTERVAL_MS)
        }
    }

    fun register() {
        val channel = MethodChannel(
            flutterEngine.dartExecutor.binaryMessenger,
            CHANNEL_NAME
        )

        channel.setMethodCallHandler { call, result ->
            when (call.method) {
                "createAuxTexture" -> handleCreateAuxTexture(call, result)
                "disposeAuxTexture" -> handleDisposeAuxTexture(call, result)
                "pauseAuxTexture" -> handlePauseAuxTexture(call, result)
                else -> result.notImplemented()
            }
        }
    }

    private fun handleCreateAuxTexture(call: MethodCall, result: MethodChannel.Result) {
        val id = call.requireArg<Int>("id", result) ?: return
        val width = call.requireArg<Int>("width", result) ?: return
        val height = call.requireArg<Int>("height", result) ?: return

        synchronized(texturesLock) {
            // Clean up existing texture with this ID
            textures[id]?.entry?.dispose()

            val textureEntry = flutterEngine.renderer.createSurfaceTexture()
            textureEntry.surfaceTexture().setDefaultBufferSize(width, height)

            val auxEntry = AuxTextureEntry(id, width, height, textureEntry)
            textures[id] = TextureState(auxEntry)

            // Create Rust-side backing store
            NesiumNative.nesiumAuxCreate(id, width, height)

            // Start update loop if this is the first texture
            if (textures.size == 1) {
                updateHandler.post(updateRunnable)
            }

            result.success(textureEntry.id())
        }
    }

    private fun handleDisposeAuxTexture(call: MethodCall, result: MethodChannel.Result) {
        val id = call.requireArg<Int>("id", result) ?: return

        // Remove from textures map immediately to prevent new updates
        val state: TextureState?
        synchronized(texturesLock) {
            state = textures.remove(id)

            // Stop update loop if no textures remain
            if (textures.isEmpty()) {
                updateHandler.removeCallbacks(updateRunnable)
            }
        }

        // Dispose renderer on the update thread to ensure no race with updateFromRust()
        // But keep textureEntry.release() on UI thread (required by Flutter)
        if (state != null) {
            val latch = CountDownLatch(1)
            updateHandler.post {
                state.entry.disposeRenderer()
                latch.countDown()
            }

            // Wait for renderer dispose to complete
            val completed = try {
                latch.await(DISPOSE_TIMEOUT_MS, TimeUnit.MILLISECONDS)
            } catch (_: InterruptedException) {
                Thread.currentThread().interrupt()
                false
            }

            if (!completed) {
                Log.w(TAG, "Dispose timeout for texture $id, forcing texture release")
            }

            // Now release Flutter texture on UI thread (we're already on UI thread)
            state.entry.releaseTexture()
        }

        result.success(null)
    }

    private fun handlePauseAuxTexture(call: MethodCall, result: MethodChannel.Result) {
        val id = call.requireArg<Int>("id", result) ?: return

        synchronized(texturesLock) {
            textures[id]?.paused = true
        }
        result.success(null)
    }

    fun dispose() {
        updateHandler.removeCallbacks(updateRunnable)
        updateThread.quitSafely()

        synchronized(texturesLock) {
            textures.values.forEach { it.entry.dispose() }
            textures.clear()
        }
    }

    /** Extension to extract required argument with error handling */
    private inline fun <reified T> MethodCall.requireArg(
        name: String,
        result: MethodChannel.Result
    ): T? {
        val value = argument<T>(name)
        if (value == null) {
            result.error("BAD_ARGS", "Missing required argument: $name", null)
        }
        return value
    }

    private class AuxTextureEntry(
        private val id: Int,
        width: Int,
        height: Int,
        private val textureEntry: TextureRegistry.SurfaceTextureEntry
    ) {
        private val renderer = NesAuxRenderer(textureEntry, width, height, id)

        fun updateFromRust() {
            renderer.updateFromRust()
        }

        /** Dispose EGL resources - can be called from any thread */
        fun disposeRenderer() {
            NesiumNative.nesiumAuxDestroy(id)
            renderer.dispose()
        }

        /** Release Flutter texture - MUST be called on UI thread */
        fun releaseTexture() {
            textureEntry.release()
        }

        /** Full dispose - only safe if called from UI thread after stopping updates */
        fun dispose() {
            disposeRenderer()
            releaseTexture()
        }
    }
}

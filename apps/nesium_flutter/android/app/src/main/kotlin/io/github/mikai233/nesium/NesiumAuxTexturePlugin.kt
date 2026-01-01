package io.github.mikai233.nesium

import android.os.Handler
import android.os.HandlerThread
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel
import io.flutter.view.TextureRegistry

/**
 * Manages auxiliary textures for debugger views (Tilemap, Pattern, etc.) on Android.
 *
 * Uses SurfaceTexture-based external textures similar to the main NES texture,
 * but receives data from the Rust aux_texture module.
 */
class NesiumAuxTexturePlugin(private val flutterEngine: FlutterEngine) {
    
    private val textures = mutableMapOf<Int, AuxTextureEntry>()
    private val updateThread = HandlerThread("NesiumAuxTexture").apply { start() }
    private val updateHandler = Handler(updateThread.looper)
    
    private val updateRunnable = object : Runnable {
        override fun run() {
            synchronized(textures) {
                textures.values.forEach { it.updateFromRust() }
            }
            updateHandler.postDelayed(this, 16) // ~60Hz
        }
    }
    
    fun register() {
        val channel = MethodChannel(
            flutterEngine.dartExecutor.binaryMessenger,
            "nesium_aux"
        )
        
        channel.setMethodCallHandler { call, result ->
            when (call.method) {
                "createAuxTexture" -> {
                    val id = call.argument<Int>("id")
                    val width = call.argument<Int>("width")
                    val height = call.argument<Int>("height")
                    
                    if (id == null || width == null || height == null) {
                        result.error("BAD_ARGS", "Missing id/width/height", null)
                        return@setMethodCallHandler
                    }
                    
                    createAuxTexture(id, width, height, result)
                }
                
                "disposeAuxTexture" -> {
                    val id = call.argument<Int>("id")
                    if (id == null) {
                        result.error("BAD_ARGS", "Missing id", null)
                        return@setMethodCallHandler
                    }
                    
                    disposeAuxTexture(id, result)
                }
                
                else -> result.notImplemented()
            }
        }
    }
    
    private fun createAuxTexture(id: Int, width: Int, height: Int, result: MethodChannel.Result) {
        synchronized(textures) {
            // Clean up existing texture with this ID
            textures[id]?.dispose()
            
            val entry = flutterEngine.renderer.createSurfaceTexture()
            entry.surfaceTexture().setDefaultBufferSize(width, height)
            
            val auxEntry = AuxTextureEntry(id, width, height, entry)
            textures[id] = auxEntry
            
            // Create Rust-side backing store
            NesiumNative.nesiumAuxCreate(id, width, height)
            
            // Start update loop if this is the first texture
            if (textures.size == 1) {
                updateHandler.post(updateRunnable)
            }
            
            result.success(entry.id())
        }
    }
    
    private fun disposeAuxTexture(id: Int, result: MethodChannel.Result) {
        synchronized(textures) {
            textures.remove(id)?.dispose()
            
            // Stop update loop if no textures remain
            if (textures.isEmpty()) {
                updateHandler.removeCallbacks(updateRunnable)
            }
        }
        
        result.success(null)
    }
    
    fun dispose() {
        updateHandler.removeCallbacks(updateRunnable)
        updateThread.quitSafely()
        
        synchronized(textures) {
            textures.values.forEach { it.dispose() }
            textures.clear()
        }
    }
    
    private inner class AuxTextureEntry(
        private val id: Int,
        private val width: Int,
        private val height: Int,
        private val textureEntry: TextureRegistry.SurfaceTextureEntry
    ) {
        private val renderer = NesAuxRenderer(textureEntry, width, height, id)
        
        fun updateFromRust() {
            // Delegate to GL renderer - zero Bitmap allocation!
            renderer.updateFromRust()
        }
        
        fun dispose() {
            NesiumNative.nesiumAuxDestroy(id)
            renderer.dispose()
            textureEntry.release()
        }
    }
}

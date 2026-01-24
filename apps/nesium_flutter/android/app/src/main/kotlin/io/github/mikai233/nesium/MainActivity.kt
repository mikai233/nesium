package io.github.mikai233.nesium

import android.os.Bundle
import android.view.Surface

import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel
import androidx.core.content.edit
import io.flutter.view.TextureRegistry


class MainActivity : FlutterActivity() {
    private val channel = "nesium"
    private var videoBackend: NesiumVideoBackend = NesiumVideoBackend.Hardware
    private var highPriority: Boolean = false
    private var auxPlugin: NesiumAuxTexturePlugin? = null

    // Main NES Flutter texture (optional on Android; SurfaceView is the default path).
    private var nesTextureEntry: TextureRegistry.SurfaceTextureEntry? = null
    private var nesTextureSurface: Surface? = null
    private var nesTextureUploadRenderer: NesRenderer? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val prefs = getSharedPreferences("nesium", MODE_PRIVATE)
        videoBackend = NesiumVideoBackend.fromMode(prefs.getInt("video_backend", 1))
        highPriority = prefs.getBoolean("high_priority", false)
        NesiumAndroidVideoBackend.set(videoBackend.mode)
        NesiumAndroidHighPriority.set(highPriority)
        NesiumNative.nativeSetVideoBackend(videoBackend.mode)
        NesiumNative.nativeSetHighPriority(highPriority)
        // Pass a stable application context to Rust.
        NesiumNative.init_android_context(applicationContext)
    }

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)

        flutterEngine.platformViewsController.registry.registerViewFactory(
            "nesium_game_view",
            NesiumGameViewFactory(),
        )

        // Register auxiliary texture plugin
        val auxPlugin = NesiumAuxTexturePlugin(flutterEngine)
        auxPlugin.register()
        this.auxPlugin = auxPlugin

        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, channel)
            .setMethodCallHandler { call, result ->
                when (call.method) {
                    "createNesTexture" -> {
                        val width = call.argument<Int>("width") ?: 256
                        val height = call.argument<Int>("height") ?: 240
                        if (width <= 0 || height <= 0) {
                            result.error("bad_args", "width/height must be > 0", null)
                            return@setMethodCallHandler
                        }
                        result.success(createOrReuseNesTexture(flutterEngine, width, height))
                    }

                    "setPresentBufferSize" -> {
                        val width = call.argument<Int>("width")
                        val height = call.argument<Int>("height")
                        if (width == null || height == null || width <= 0 || height <= 0) {
                            result.error("bad_args", "width/height must be provided and > 0", null)
                            return@setMethodCallHandler
                        }
                        nesTextureEntry?.surfaceTexture()?.setDefaultBufferSize(width, height)
                        result.success(null)
                    }

                    "disposeNesTexture" -> {
                        disposeNesTexture()
                        result.success(null)
                    }

                    "setVideoBackend" -> {
                        val mode = call.argument<Int>("mode")
                        if (mode == null || (mode != 0 && mode != 1)) {
                            result.error("bad_args", "mode must be 0 or 1", null)
                            return@setMethodCallHandler
                        }
                        // Persist preference; it will take effect on next cold start.
                        val prefs = getSharedPreferences("nesium", MODE_PRIVATE)
                        prefs.edit { putInt("video_backend", mode) }
                        result.success(null)
                    }

                    "setAndroidHighPriority" -> {
                        val enabled = call.argument<Boolean>("enabled")
                        if (enabled == null) {
                            result.error("bad_args", "enabled must be a bool", null)
                            return@setMethodCallHandler
                        }
                        val prefs = getSharedPreferences("nesium", MODE_PRIVATE)
                        prefs.edit { putBoolean("high_priority", enabled) }
                        highPriority = enabled
                        NesiumAndroidHighPriority.set(enabled)
                        NesiumNative.nativeSetHighPriority(enabled)
                        result.success(null)
                    }

                    // Android SurfaceView buffer size (PlatformView path).
                    "setAndroidSurfaceSize" -> {
                        val width = call.argument<Int>("width")
                        val height = call.argument<Int>("height")
                        if (width == null || height == null) {
                            result.error("bad_args", "width/height must be provided", null)
                            return@setMethodCallHandler
                        }
                        NesiumGameView.setSurfaceSize(width, height)
                        result.success(null)
                    }

                    else -> result.notImplemented()
                }
            }
    }

    override fun onDestroy() {
        // Best-effort: if a renderer thread is still running when the Activity is being torn down,
        // stop it to avoid leaks/crashes.
        NesiumNative.nativeStopRustRenderer()
        disposeNesTexture()
        auxPlugin?.dispose()
        auxPlugin = null
        super.onDestroy()
    }

    private fun createOrReuseNesTexture(
        flutterEngine: FlutterEngine,
        width: Int,
        height: Int,
    ): Long {
        val existing = nesTextureEntry
        if (existing != null) {
            // Best-effort: update buffer size to requested dimensions.
            existing.surfaceTexture().setDefaultBufferSize(width, height)
            return existing.id()
        }

        val entry = flutterEngine.renderer.createSurfaceTexture()
        entry.surfaceTexture().setDefaultBufferSize(width, height)

        val surface = Surface(entry.surfaceTexture())
        nesTextureEntry = entry
        nesTextureSurface = surface

        when (videoBackend) {
            NesiumVideoBackend.Upload -> {
                nesTextureUploadRenderer = NesRenderer(
                    surface = surface,
                    releaseSurface = false,
                    highPriorityEnabled = highPriority,
                )
            }

            NesiumVideoBackend.Hardware -> {
                NesiumNative.nativeStartRustRenderer(surface)
            }
        }

        return entry.id()
    }

    private fun disposeNesTexture() {
        // Upload backend uses Kotlin GL thread; stop it first.
        nesTextureUploadRenderer?.dispose(waitForShutdown = true)
        nesTextureUploadRenderer = null

        // Hardware backend uses Rust renderer.
        NesiumNative.nativeStopRustRenderer()

        nesTextureSurface?.release()
        nesTextureSurface = null

        nesTextureEntry?.release()
        nesTextureEntry = null
    }
}

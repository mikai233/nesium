package io.github.mikai233.nesium

import android.content.pm.ApplicationInfo
import android.os.Bundle
import android.view.Surface

import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel
import io.flutter.view.TextureRegistry
import androidx.core.content.edit


class MainActivity : FlutterActivity() {
    private val channel = "nesium"
    private var renderer: NesRenderer? = null
    private var rustRendererSurface: Surface? = null
    private var rustTextureEntry: TextureRegistry.SurfaceTextureEntry? = null
    private var videoBackend: Int = 1 // default to hardware (Scheme B)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val prefs = getSharedPreferences("nesium", MODE_PRIVATE)
        videoBackend = prefs.getInt("video_backend", 1)
        NesiumNative.nativeSetVideoBackend(videoBackend)
        // Pass a stable application context to Rust.
        NesiumNative.init_android_context(applicationContext)
    }

    private fun disposeTextureInternal() {
        renderer?.dispose(waitForShutdown = true)
        renderer = null
        NesiumNative.nativeStopRustRenderer()
        rustRendererSurface?.release()
        rustRendererSurface = null
        rustTextureEntry?.release()
        rustTextureEntry = null
    }

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)

        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, channel)
            .setMethodCallHandler { call, result ->
                when (call.method) {
                    "createNesTexture" -> {
                        // Replace any existing texture.
                        disposeTextureInternal()

                        // SurfaceTexture-based external texture.
                        val entry = flutterEngine.renderer.createSurfaceTexture()
                        entry.surfaceTexture().setDefaultBufferSize(256, 240)
                        val profilingEnabled =
                            (applicationInfo.flags and ApplicationInfo.FLAG_DEBUGGABLE) != 0

                        if (videoBackend == 1) {
                            // Scheme B: Rust renders directly into the SurfaceTexture via EGL.
                            rustTextureEntry = entry
                            rustRendererSurface = Surface(entry.surfaceTexture())
                            NesiumNative.nativeStartRustRenderer(rustRendererSurface!!)
                        } else {
                            // Scheme A: Kotlin uploads the planes into a GL texture.
                            renderer = NesRenderer(
                                flutterEngine = flutterEngine,
                                textureEntry = entry,
                                profilingEnabled = profilingEnabled,
                            )
                        }

                        result.success(entry.id())
                    }

                    "disposeNesTexture" -> {
                        disposeTextureInternal()
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

                    else -> result.notImplemented()
                }
            }
    }

    override fun onDestroy() {
        disposeTextureInternal()
        super.onDestroy()
    }
}

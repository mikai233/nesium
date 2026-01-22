package io.github.mikai233.nesium

import android.os.Bundle

import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel
import androidx.core.content.edit


class MainActivity : FlutterActivity() {
    private val channel = "nesium"
    private var videoBackend: NesiumVideoBackend = NesiumVideoBackend.Hardware
    private var highPriority: Boolean = false
    private var auxPlugin: NesiumAuxTexturePlugin? = null

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

                    else -> result.notImplemented()
                }
            }
    }

    override fun onDestroy() {
        // Best-effort: if a renderer thread is still running when the Activity is being torn down,
        // stop it to avoid leaks/crashes.
        NesiumNative.nativeStopRustRenderer()
        auxPlugin?.dispose()
        auxPlugin = null
        super.onDestroy()
    }
}

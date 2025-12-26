package io.github.mikai233.nesium

import android.content.pm.ApplicationInfo
import android.os.Bundle

import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel


class MainActivity : FlutterActivity() {
    private val channel = "nesium"
    private var renderer: NesRenderer? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // Pass a stable application context to Rust.
        NesiumNative.init_android_context(applicationContext)
    }

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)

        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, channel)
            .setMethodCallHandler { call, result ->
                when (call.method) {
                    "createNesTexture" -> {
                        // SurfaceTexture-based external texture.
                        val entry = flutterEngine.renderer.createSurfaceTexture()
                        entry.surfaceTexture().setDefaultBufferSize(256, 240)

                        renderer?.dispose(waitForShutdown = true)
                        renderer = NesRenderer(
                            flutterEngine = flutterEngine,
                            textureEntry = entry,
                            profilingEnabled = (applicationInfo.flags and ApplicationInfo.FLAG_DEBUGGABLE) != 0,
                        )

                        result.success(entry.id())
                    }

                    "disposeNesTexture" -> {
                        renderer?.dispose(waitForShutdown = true)
                        renderer = null
                        result.success(null)
                    }

                    else -> result.notImplemented()
                }
            }
    }

    override fun onDestroy() {
        renderer?.dispose(waitForShutdown = true)
        renderer = null
        super.onDestroy()
    }
}

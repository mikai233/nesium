package io.github.mikai233.nesium

import android.content.Context
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.view.View
import io.flutter.plugin.platform.PlatformView

/**
 * Native SurfaceView-backed renderer for experimenting with a more "native" presentation path
 * (bypassing Flutter external textures).
 *
 * This view starts/stops the Rust renderer based on surface lifecycle.
 */
class NesiumGameView(context: Context) : PlatformView, SurfaceHolder.Callback {
    private val surfaceView: SurfaceView = SurfaceView(context)
    private var uploadRenderer: NesRenderer? = null

    init {
        // Note: do NOT force the underlying buffer to 256x240.
        // If we do, Android will scale the small Surface buffer to the view size in the system
        // compositor (typically bilinear), which looks blurry/dirty for pixel art.
        // We want the Surface buffer to match the view size so scaling happens in our renderer
        // (nearest-neighbor).
        surfaceView.holder.addCallback(this)
        surfaceView.keepScreenOn = true
    }

    override fun getView(): View = surfaceView

    override fun dispose() {
        surfaceView.holder.removeCallback(this)
        stopRenderer()
    }

    override fun surfaceCreated(holder: SurfaceHolder) {
        stopRenderer()

        val backend = NesiumAndroidVideoBackend.get(surfaceView.context)
        when (backend) {
            NesiumVideoBackend.Upload -> {
                // Scheme A: Kotlin uploads planes into a GL texture and presents to the SurfaceView.
                val highPriorityEnabled = NesiumAndroidHighPriority.get(surfaceView.context)
                uploadRenderer = NesRenderer(
                    surface = holder.surface,
                    releaseSurface = false,
                    highPriorityEnabled = highPriorityEnabled,
                )
            }

            NesiumVideoBackend.Hardware -> {
                // Scheme B: Rust renders directly into the SurfaceView via EGL.
                NesiumNative.nativeStartRustRenderer(holder.surface)
            }
        }
    }

    override fun surfaceDestroyed(holder: SurfaceHolder) {
        stopRenderer()
    }

    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {}

    private fun stopRenderer() {
        uploadRenderer?.dispose(waitForShutdown = true)
        uploadRenderer = null
        NesiumNative.nativeStopRustRenderer()
    }
}

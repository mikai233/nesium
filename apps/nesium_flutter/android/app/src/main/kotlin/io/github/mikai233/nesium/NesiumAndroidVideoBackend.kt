package io.github.mikai233.nesium

import android.content.Context

enum class NesiumVideoBackend(val mode: Int) {
    Upload(0),
    Hardware(1);

    companion object {
        fun fromMode(mode: Int): NesiumVideoBackend {
            return if (mode == 0) Upload else Hardware
        }
    }
}

/**
 * Process-stable Android video backend selection.
 *
 * The backend is chosen on cold start in [MainActivity.onCreate] and should not change until the
 * process restarts. This avoids accidentally switching render paths when the SurfaceView is
 * recreated (e.g. orientation changes) while the user has only updated persisted preferences.
 */
object NesiumAndroidVideoBackend {
    @Volatile
    private var backend: NesiumVideoBackend? = null

    fun set(mode: Int) {
        backend = NesiumVideoBackend.fromMode(mode)
    }

    fun get(context: Context): NesiumVideoBackend {
        val value = backend
        if (value != null) return value
        val prefs = context.getSharedPreferences("nesium", Context.MODE_PRIVATE)
        return NesiumVideoBackend.fromMode(prefs.getInt("video_backend", 1))
    }

    fun getMode(context: Context): Int = get(context).mode
}

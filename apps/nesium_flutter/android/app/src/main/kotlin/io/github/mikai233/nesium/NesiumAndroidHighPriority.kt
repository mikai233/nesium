package io.github.mikai233.nesium

import android.content.Context

object NesiumAndroidHighPriority {
    private const val PREFS_NAME = "nesium"
    private const val KEY = "high_priority"

    @Volatile
    private var enabled: Boolean? = null

    fun set(value: Boolean) {
        enabled = value
    }

    fun get(context: Context): Boolean {
        val cached = enabled
        if (cached != null) return cached
        val prefs = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        return prefs.getBoolean(KEY, false)
    }
}


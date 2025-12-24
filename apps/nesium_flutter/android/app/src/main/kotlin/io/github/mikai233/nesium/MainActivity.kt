package io.github.mikai233.nesium

import android.content.Context
import android.os.Bundle
import io.flutter.embedding.android.FlutterActivity

class MainActivity : FlutterActivity() {
    init {
        System.loadLibrary("nesium_flutter")
    }

    @Suppress("FunctionName")
    external fun init_android_context(context: Context)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        init_android_context(applicationContext)
    }
}

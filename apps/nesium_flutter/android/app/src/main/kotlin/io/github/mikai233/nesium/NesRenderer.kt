package io.github.mikai233.nesium

import android.os.Handler
import android.os.HandlerThread
import android.view.Surface
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.view.TextureRegistry
import android.opengl.EGL14
import android.opengl.GLES20
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.nio.FloatBuffer


/**
 * A dedicated GL thread that uploads the latest NES frame into a GL texture.
 *
 * This implementation uses an EGL window surface created from a Java `Surface` that wraps the
 * Flutter-provided `SurfaceTexture`. Each swap produces a new buffer that Flutter can consume.
 */
class NesRenderer(
    private val flutterEngine: FlutterEngine,
    private val textureEntry: TextureRegistry.SurfaceTextureEntry,
) {
    private val thread = HandlerThread("NesGLThread")
    private val handler: Handler

    private var eglDisplay = EGL14.EGL_NO_DISPLAY
    private var eglContext = EGL14.EGL_NO_CONTEXT
    private var eglSurface = EGL14.EGL_NO_SURFACE

    private var windowSurface: Surface? = null

    private var textureId = 0
    private var program = 0

    private var aPosition = -1
    private var aTexCoord = -1
    private var uTexture = -1
    private var uSwapRb = -1

    private var running = true
    private var lastSeq = -1L

    // Latest-only frame polling: lower values reduce latency but increase CPU usage.
    private val pollDelayMsWhenIdle = 2L

    private val vertexShaderCode = """
        attribute vec4 a_position;
        attribute vec2 a_tex_coord;
        varying vec2 v_tex_coord;
        void main() {
            gl_Position = a_position;
            v_tex_coord = a_tex_coord;
        }
    """.trimIndent()

    private val fragmentShaderCode = """
        precision mediump float;
        uniform sampler2D u_texture;
        uniform int u_swap_rb;
        varying vec2 v_tex_coord;
        void main() {
            vec4 c = texture2D(u_texture, v_tex_coord);
            if (u_swap_rb != 0) {
                c = vec4(c.b, c.g, c.r, c.a);
            }
            gl_FragColor = c;
        }
    """.trimIndent()

    private val vertexData: FloatBuffer = ByteBuffer.allocateDirect(16 * 4)
        .order(ByteOrder.nativeOrder())
        .asFloatBuffer()
        .apply {
            put(
                floatArrayOf(
                    // X,   Y,   U,   V
                    -1f, -1f, 0f, 1f, // Bottom-Left
                    1f, -1f, 1f, 1f,  // Bottom-Right
                    -1f, 1f, 0f, 0f,  // Top-Left
                    1f, 1f, 1f, 0f    // Top-Right
                )
            )
            position(0)
        }

    init {
        thread.start()
        handler = Handler(thread.looper)
        handler.post { initGLAndLoop() }
    }

    private fun initGLAndLoop() {
        // EGL display.
        eglDisplay = EGL14.eglGetDisplay(EGL14.EGL_DEFAULT_DISPLAY)
        val version = IntArray(2)
        if (!EGL14.eglInitialize(eglDisplay, version, 0, version, 1)) {
            throw RuntimeException("eglInitialize failed")
        }

        // Bind OpenGL ES API explicitly for better device compatibility.
        EGL14.eglBindAPI(EGL14.EGL_OPENGL_ES_API)

        // Choose an RGBA8888 ES2 config.
        val configAttribs = intArrayOf(
            EGL14.EGL_RENDERABLE_TYPE, EGL14.EGL_OPENGL_ES2_BIT,
            EGL14.EGL_RED_SIZE, 8,
            EGL14.EGL_GREEN_SIZE, 8,
            EGL14.EGL_BLUE_SIZE, 8,
            EGL14.EGL_ALPHA_SIZE, 8,
            EGL14.EGL_NONE
        )
        val configs = arrayOfNulls<android.opengl.EGLConfig>(1)
        val numConfigs = IntArray(1)
        if (!EGL14.eglChooseConfig(eglDisplay, configAttribs, 0, configs, 0, 1, numConfigs, 0)) {
            throw RuntimeException("eglChooseConfig failed")
        }
        val config = configs[0] ?: throw RuntimeException("No EGLConfig")

        // GLES2 context.
        val contextAttribs = intArrayOf(
            EGL14.EGL_CONTEXT_CLIENT_VERSION, 2,
            EGL14.EGL_NONE
        )
        eglContext =
            EGL14.eglCreateContext(eglDisplay, config, EGL14.EGL_NO_CONTEXT, contextAttribs, 0)
        if (eglContext == EGL14.EGL_NO_CONTEXT) {
            throw RuntimeException("eglCreateContext failed")
        }

        // Window surface backed by the Flutter SurfaceTexture.
        windowSurface = Surface(textureEntry.surfaceTexture())
        val surfaceAttribs = intArrayOf(EGL14.EGL_NONE)
        eglSurface =
            EGL14.eglCreateWindowSurface(eglDisplay, config, windowSurface, surfaceAttribs, 0)
        if (eglSurface == EGL14.EGL_NO_SURFACE) {
            throw RuntimeException("eglCreateWindowSurface failed")
        }

        if (!EGL14.eglMakeCurrent(eglDisplay, eglSurface, eglSurface, eglContext)) {
            throw RuntimeException("eglMakeCurrent failed")
        }

        // Prefer VSYNC to reduce jitter.
        EGL14.eglSwapInterval(eglDisplay, 1)

        // Compile and link program.
        val vertexShader = loadShader(GLES20.GL_VERTEX_SHADER, vertexShaderCode)
        val fragmentShader = loadShader(GLES20.GL_FRAGMENT_SHADER, fragmentShaderCode)
        program = GLES20.glCreateProgram()
        GLES20.glAttachShader(program, vertexShader)
        GLES20.glAttachShader(program, fragmentShader)
        GLES20.glLinkProgram(program)

        aPosition = GLES20.glGetAttribLocation(program, "a_position")
        aTexCoord = GLES20.glGetAttribLocation(program, "a_tex_coord")
        uTexture = GLES20.glGetUniformLocation(program, "u_texture")
        uSwapRb = GLES20.glGetUniformLocation(program, "u_swap_rb")

        // Texture storage.
        val textures = IntArray(1)
        GLES20.glGenTextures(1, textures, 0)
        textureId = textures[0]
        GLES20.glBindTexture(GLES20.GL_TEXTURE_2D, textureId)
        GLES20.glTexParameteri(
            GLES20.GL_TEXTURE_2D,
            GLES20.GL_TEXTURE_MIN_FILTER,
            GLES20.GL_NEAREST
        )
        GLES20.glTexParameteri(
            GLES20.GL_TEXTURE_2D,
            GLES20.GL_TEXTURE_MAG_FILTER,
            GLES20.GL_NEAREST
        )
        GLES20.glTexParameteri(
            GLES20.GL_TEXTURE_2D,
            GLES20.GL_TEXTURE_WRAP_S,
            GLES20.GL_CLAMP_TO_EDGE
        )
        GLES20.glTexParameteri(
            GLES20.GL_TEXTURE_2D,
            GLES20.GL_TEXTURE_WRAP_T,
            GLES20.GL_CLAMP_TO_EDGE
        )

        // Robust pixel upload alignment.
        GLES20.glPixelStorei(GLES20.GL_UNPACK_ALIGNMENT, 1)

        val w = NesiumNative.nativeFrameWidth()
        val h = NesiumNative.nativeFrameHeight()

        GLES20.glTexImage2D(
            GLES20.GL_TEXTURE_2D,
            0,
            GLES20.GL_RGBA,
            w,
            h,
            0,
            GLES20.GL_RGBA,
            GLES20.GL_UNSIGNED_BYTE,
            null
        )

        // Default viewport to the surface size.
        val surfaceW = IntArray(1)
        val surfaceH = IntArray(1)
        EGL14.eglQuerySurface(eglDisplay, eglSurface, EGL14.EGL_WIDTH, surfaceW, 0)
        EGL14.eglQuerySurface(eglDisplay, eglSurface, EGL14.EGL_HEIGHT, surfaceH, 0)
        GLES20.glViewport(0, 0, surfaceW[0], surfaceH[0])

        renderLoop()
    }

    private fun loadShader(type: Int, code: String): Int {
        val shader = GLES20.glCreateShader(type)
        GLES20.glShaderSource(shader, code)
        GLES20.glCompileShader(shader)
        return shader
    }

    private fun renderLoop() {
        if (!running) return

        val seq = NesiumNative.nativeFrameSeq()
        if (seq == lastSeq) {
            // No new frame: avoid spinning.
            handler.postDelayed({ renderLoop() }, pollDelayMsWhenIdle)
            return
        }

        lastSeq = seq

        // Decide swizzle based on the source format.
        // 0 = RGBA, 1 = BGRA
        val swapRb = NesiumNative.nativeColorFormat() == 1

        val idx = NesiumNative.nativeBeginFrontCopy()
        try {
            val buffer = NesiumNative.nativePlaneBuffer(idx)
            buffer.position(0)

            val w = NesiumNative.nativeFrameWidth()
            val h = NesiumNative.nativeFrameHeight()

            GLES20.glBindTexture(GLES20.GL_TEXTURE_2D, textureId)
            GLES20.glTexSubImage2D(
                GLES20.GL_TEXTURE_2D,
                0,
                0,
                0,
                w,
                h,
                GLES20.GL_RGBA,
                GLES20.GL_UNSIGNED_BYTE,
                buffer
            )

            // Draw.
            GLES20.glClearColor(0f, 0f, 0f, 1f)
            GLES20.glClear(GLES20.GL_COLOR_BUFFER_BIT)

            GLES20.glUseProgram(program)
            GLES20.glUniform1i(uTexture, 0)
            GLES20.glUniform1i(uSwapRb, if (swapRb) 1 else 0)

            vertexData.position(0)
            GLES20.glVertexAttribPointer(aPosition, 2, GLES20.GL_FLOAT, false, 16, vertexData)
            GLES20.glEnableVertexAttribArray(aPosition)

            vertexData.position(2)
            GLES20.glVertexAttribPointer(aTexCoord, 2, GLES20.GL_FLOAT, false, 16, vertexData)
            GLES20.glEnableVertexAttribArray(aTexCoord)

            GLES20.glDrawArrays(GLES20.GL_TRIANGLE_STRIP, 0, 4)

            EGL14.eglSwapBuffers(eglDisplay, eglSurface)

            // For SurfaceTexture-backed Flutter textures, swapping the EGL window surface enqueues
            // a new buffer to the underlying SurfaceTexture. The Flutter embedding registers an
            // internal OnFrameAvailableListener on that SurfaceTexture and will schedule a redraw.

            // Post the next frame ASAP to keep latency low.
            handler.post { renderLoop() }
        } finally {
            NesiumNative.nativeEndFrontCopy()
        }
    }

    fun dispose() {
        running = false
        handler.post {
            try {
                if (textureId != 0) {
                    GLES20.glDeleteTextures(1, intArrayOf(textureId), 0)
                }
                if (program != 0) {
                    GLES20.glDeleteProgram(program)
                }

                if (eglDisplay != EGL14.EGL_NO_DISPLAY && eglSurface != EGL14.EGL_NO_SURFACE) {
                    EGL14.eglDestroySurface(eglDisplay, eglSurface)
                }
                if (eglDisplay != EGL14.EGL_NO_DISPLAY && eglContext != EGL14.EGL_NO_CONTEXT) {
                    EGL14.eglDestroyContext(eglDisplay, eglContext)
                }
                if (eglDisplay != EGL14.EGL_NO_DISPLAY) {
                    EGL14.eglTerminate(eglDisplay)
                }
            } finally {
                // Release Java Surface wrapper explicitly.
                windowSurface?.release()
                windowSurface = null

                textureEntry.release()
                thread.quitSafely()
            }
        }
    }
}
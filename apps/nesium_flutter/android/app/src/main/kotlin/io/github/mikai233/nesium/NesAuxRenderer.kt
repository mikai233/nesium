package io.github.mikai233.nesium

import android.opengl.EGL14
import android.opengl.GLES20
import android.view.Surface
import io.flutter.view.TextureRegistry
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.nio.FloatBuffer

/**
 * High-performance GL-based renderer for auxiliary textures (Tilemap, Pattern, etc.)
 *
 * Similar to NesRenderer but simplified:
 * - No frame signal pipe (uses 60Hz polling from plugin)
 * - Dedicated EGL context per texture
 * - Direct glTexSubImage2D upload from Rust buffer
 */
class NesAuxRenderer(
    private val textureEntry: TextureRegistry.SurfaceTextureEntry,
    private val width: Int,
    private val height: Int,
    private val auxId: Int
) {
    private var eglDisplay = EGL14.EGL_NO_DISPLAY
    private var eglContext = EGL14.EGL_NO_CONTEXT
    private var eglSurface = EGL14.EGL_NO_SURFACE
    private var windowSurface: Surface? = null
    private val eglLock = Any()
    @Volatile private var disposed = false

    private var textureId = 0
    private var program = 0
    private var aPosition = -1
    private var aTexCoord = -1
    private var uTexture = -1

    private val buffer = ByteBuffer.allocateDirect(width * height * 4)

    companion object {
        private const val VERTEX_SHADER_CODE = """
            attribute vec4 a_position;
            attribute vec2 a_tex_coord;
            varying vec2 v_tex_coord;
            void main() {
                gl_Position = a_position;
                v_tex_coord = a_tex_coord;
            }
        """

        private const val FRAGMENT_SHADER_CODE = """
            precision mediump float;
            uniform sampler2D u_texture;
            varying vec2 v_tex_coord;
            void main() {
                gl_FragColor = texture2D(u_texture, v_tex_coord);
            }
        """
    }

    private val vertexData: FloatBuffer = ByteBuffer.allocateDirect(16 * 4)
        .order(ByteOrder.nativeOrder())
        .asFloatBuffer()
        .apply {
            put(floatArrayOf(
                // X,   Y,   U,   V
                -1f, -1f, 0f, 1f, // Bottom-Left
                 1f, -1f, 1f, 1f, // Bottom-Right
                -1f,  1f, 0f, 0f, // Top-Left
                 1f,  1f, 1f, 0f  // Top-Right
            ))
            position(0)
        }

    init {
        initGL()
    }

    private fun initGL() {
        // EGL display
        eglDisplay = EGL14.eglGetDisplay(EGL14.EGL_DEFAULT_DISPLAY)
        val version = IntArray(2)
        if (!EGL14.eglInitialize(eglDisplay, version, 0, version, 1)) {
            throw RuntimeException("eglInitialize failed")
        }

        EGL14.eglBindAPI(EGL14.EGL_OPENGL_ES_API)

        // Choose RGBA8888 ES2 config
        val configAttribs = intArrayOf(
            EGL14.EGL_RENDERABLE_TYPE, EGL14.EGL_OPENGL_ES2_BIT,
            EGL14.EGL_SURFACE_TYPE, EGL14.EGL_WINDOW_BIT,
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

        // GLES2 context
        val contextAttribs = intArrayOf(
            EGL14.EGL_CONTEXT_CLIENT_VERSION, 2,
            EGL14.EGL_NONE
        )
        eglContext = EGL14.eglCreateContext(eglDisplay, config, EGL14.EGL_NO_CONTEXT, contextAttribs, 0)
        if (eglContext == EGL14.EGL_NO_CONTEXT) {
            throw RuntimeException("eglCreateContext failed")
        }

        // Window surface backed by Flutter SurfaceTexture
        val st = textureEntry.surfaceTexture()
        // Ensure the producer buffer matches our GL viewport size
        st.setDefaultBufferSize(width, height)
        windowSurface = Surface(st)
        val surfaceAttribs = intArrayOf(EGL14.EGL_NONE)
        eglSurface = EGL14.eglCreateWindowSurface(eglDisplay, config, windowSurface, surfaceAttribs, 0)
        if (eglSurface == EGL14.EGL_NO_SURFACE) {
            throw RuntimeException("eglCreateWindowSurface failed")
        }

        if (!EGL14.eglMakeCurrent(eglDisplay, eglSurface, eglSurface, eglContext)) {
            throw RuntimeException("eglMakeCurrent failed")
        }

        // Disable dithering
        GLES20.glDisable(GLES20.GL_DITHER)

        // Compile shaders
        val vertexShader = loadShader(GLES20.GL_VERTEX_SHADER, VERTEX_SHADER_CODE)
        val fragmentShader = loadShader(GLES20.GL_FRAGMENT_SHADER, FRAGMENT_SHADER_CODE)
        program = GLES20.glCreateProgram()
        GLES20.glAttachShader(program, vertexShader)
        GLES20.glAttachShader(program, fragmentShader)
        GLES20.glLinkProgram(program)

        val linkStatus = IntArray(1)
        GLES20.glGetProgramiv(program, GLES20.GL_LINK_STATUS, linkStatus, 0)
        if (linkStatus[0] == 0) {
            val log = GLES20.glGetProgramInfoLog(program)
            GLES20.glDeleteProgram(program)
            GLES20.glDeleteShader(vertexShader)
            GLES20.glDeleteShader(fragmentShader)
            throw RuntimeException("Program link failed: $log")
        }

        // Shaders are now linked into program, delete them to free resources
        GLES20.glDeleteShader(vertexShader)
        GLES20.glDeleteShader(fragmentShader)

        aPosition = GLES20.glGetAttribLocation(program, "a_position")
        aTexCoord = GLES20.glGetAttribLocation(program, "a_tex_coord")
        uTexture = GLES20.glGetUniformLocation(program, "u_texture")

        // Create GL texture
        val textures = IntArray(1)
        GLES20.glGenTextures(1, textures, 0)
        textureId = textures[0]
        GLES20.glBindTexture(GLES20.GL_TEXTURE_2D, textureId)
        GLES20.glTexParameteri(GLES20.GL_TEXTURE_2D, GLES20.GL_TEXTURE_MIN_FILTER, GLES20.GL_NEAREST)
        GLES20.glTexParameteri(GLES20.GL_TEXTURE_2D, GLES20.GL_TEXTURE_MAG_FILTER, GLES20.GL_NEAREST)
        GLES20.glTexParameteri(GLES20.GL_TEXTURE_2D, GLES20.GL_TEXTURE_WRAP_S, GLES20.GL_CLAMP_TO_EDGE)
        GLES20.glTexParameteri(GLES20.GL_TEXTURE_2D, GLES20.GL_TEXTURE_WRAP_T, GLES20.GL_CLAMP_TO_EDGE)

        GLES20.glPixelStorei(GLES20.GL_UNPACK_ALIGNMENT, 1)

        // Allocate texture storage
        GLES20.glTexImage2D(
            GLES20.GL_TEXTURE_2D, 0, GLES20.GL_RGBA,
            width, height, 0,
            GLES20.GL_RGBA, GLES20.GL_UNSIGNED_BYTE, null
        )

        // Set viewport
        GLES20.glViewport(0, 0, width, height)

        unbind()
    }

    private fun unbind() {
        if (eglDisplay != EGL14.EGL_NO_DISPLAY) {
            EGL14.eglMakeCurrent(
                eglDisplay,
                EGL14.EGL_NO_SURFACE,
                EGL14.EGL_NO_SURFACE,
                EGL14.EGL_NO_CONTEXT
            )
        }
    }

    private fun loadShader(type: Int, code: String): Int {
        val shader = GLES20.glCreateShader(type)
        GLES20.glShaderSource(shader, code)
        GLES20.glCompileShader(shader)
        val status = IntArray(1)
        GLES20.glGetShaderiv(shader, GLES20.GL_COMPILE_STATUS, status, 0)
        if (status[0] == 0) {
            val log = GLES20.glGetShaderInfoLog(shader)
            GLES20.glDeleteShader(shader)
            throw RuntimeException("Shader compile failed: $log")
        }
        return shader
    }

    /**
     * Update texture from Rust buffer and render
     * This is called from the update thread at ~60Hz
     */
    fun updateFromRust() {
        synchronized(eglLock) {
            if (disposed) return
            if (eglDisplay == EGL14.EGL_NO_DISPLAY || eglContext == EGL14.EGL_NO_CONTEXT || eglSurface == EGL14.EGL_NO_SURFACE) return
            if (windowSurface?.isValid != true) return

            // Make context current (thread-safe)
            if (!EGL14.eglMakeCurrent(eglDisplay, eglSurface, eglSurface, eglContext)) {
                // Surface/context likely got torn down during UI switch
                return
            }

            try {
                // Copy from Rust buffer to local ByteBuffer
                buffer.position(0)
                val copied = NesiumNative.nesiumAuxCopy(auxId, buffer, width * 4, height)

                if (copied > 0) {
                    buffer.position(0)

                    // Upload directly to GPU texture
                    GLES20.glActiveTexture(GLES20.GL_TEXTURE0)
                    GLES20.glBindTexture(GLES20.GL_TEXTURE_2D, textureId)
                    GLES20.glTexSubImage2D(
                        GLES20.GL_TEXTURE_2D, 0, 0, 0,
                        width, height,
                        GLES20.GL_RGBA, GLES20.GL_UNSIGNED_BYTE,
                        buffer
                    )

                    // Render quad with texture
                    GLES20.glUseProgram(program)
                    GLES20.glUniform1i(uTexture, 0)

                    vertexData.position(0)
                    GLES20.glVertexAttribPointer(aPosition, 2, GLES20.GL_FLOAT, false, 16, vertexData)
                    GLES20.glEnableVertexAttribArray(aPosition)

                    vertexData.position(2)
                    GLES20.glVertexAttribPointer(aTexCoord, 2, GLES20.GL_FLOAT, false, 16, vertexData)
                    GLES20.glEnableVertexAttribArray(aTexCoord)

                    GLES20.glDrawArrays(GLES20.GL_TRIANGLE_STRIP, 0, 4)

                    // Swap buffers - this enqueues frame to Flutter's SurfaceTexture
                    EGL14.eglSwapBuffers(eglDisplay, eglSurface)
                }
            } finally {
                unbind()
            }
        }
    }

    fun dispose() {
        synchronized(eglLock) {
            if (disposed) return
            disposed = true

            // Make context current for cleanup
            if (eglDisplay != EGL14.EGL_NO_DISPLAY && 
                eglSurface != EGL14.EGL_NO_SURFACE && 
                eglContext != EGL14.EGL_NO_CONTEXT) {
                EGL14.eglMakeCurrent(eglDisplay, eglSurface, eglSurface, eglContext)
                
                // Wait for GPU to finish all pending operations
                GLES20.glFinish()
            }

            if (textureId != 0) {
                GLES20.glDeleteTextures(1, intArrayOf(textureId), 0)
                textureId = 0
            }
            if (program != 0) {
                GLES20.glDeleteProgram(program)
                program = 0
            }

            // Unbind context before destroying surface to prevent race
            if (eglDisplay != EGL14.EGL_NO_DISPLAY) {
                EGL14.eglMakeCurrent(
                    eglDisplay,
                    EGL14.EGL_NO_SURFACE,
                    EGL14.EGL_NO_SURFACE,
                    EGL14.EGL_NO_CONTEXT
                )
            }

            // Release window surface FIRST - this disconnects from SurfaceTexture
            windowSurface?.release()
            windowSurface = null

            // Now destroy EGL surface (safe since we've disconnected)
            if (eglDisplay != EGL14.EGL_NO_DISPLAY && eglSurface != EGL14.EGL_NO_SURFACE) {
                EGL14.eglDestroySurface(eglDisplay, eglSurface)
            }
            eglSurface = EGL14.EGL_NO_SURFACE

            if (eglDisplay != EGL14.EGL_NO_DISPLAY && eglContext != EGL14.EGL_NO_CONTEXT) {
                EGL14.eglDestroyContext(eglDisplay, eglContext)
            }
            eglContext = EGL14.EGL_NO_CONTEXT

            if (eglDisplay != EGL14.EGL_NO_DISPLAY) {
                EGL14.eglTerminate(eglDisplay)
            }
            eglDisplay = EGL14.EGL_NO_DISPLAY
        }
    }
}

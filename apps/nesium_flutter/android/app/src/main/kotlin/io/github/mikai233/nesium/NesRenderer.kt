package io.github.mikai233.nesium

import android.opengl.EGL14
import android.opengl.GLES20
import android.os.Build
import android.os.Handler
import android.os.HandlerThread
import android.os.Looper
import android.os.MessageQueue
import android.os.ParcelFileDescriptor
import android.os.Process
import android.system.ErrnoException
import android.system.Os
import android.system.OsConstants
import android.util.Log
import android.view.Surface
import java.io.FileDescriptor
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.nio.FloatBuffer
import java.util.concurrent.CountDownLatch


/**
 * A dedicated GL thread that uploads the latest NES frame into a GL texture.
 *
 * This implementation uses an EGL window surface created from a Java `Surface` that wraps the
 * Flutter-provided `SurfaceTexture` (external texture) or a native `SurfaceView`.
 */
class NesRenderer(
    private val surface: Surface,
    private val releaseSurface: Boolean,
    private val onDispose: (() -> Unit)? = null,
) {
    private companion object {
        private const val TAG = "NesRenderer"
    }

    private inline fun bestEffort(action: String, block: () -> Unit) {
        try {
            block()
        } catch (t: Throwable) {
            Log.d(TAG, "Best-effort operation failed: $action", t)
        }
    }

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
    private var running = true
    private var lastSeq = -1L

    // Cached native frame size (can change at runtime).
    private var frameW = 0
    private var frameH = 0

    // Cached DirectByteBuffers for the two persistent planes (double-buffered).
    private var planeBuffers: Array<ByteBuffer?> = arrayOfNulls(2)

    // Cached EGL surface size for viewport updates.
    private var surfaceW = -1
    private var surfaceH = -1

    // Frame-ready wakeup via a native-written pipe.
    private var frameSignalRead: ParcelFileDescriptor? = null
    private var frameSignalWrite: ParcelFileDescriptor? = null
    private var frameSignalFd: FileDescriptor? = null
    private var frameSignalNonBlocking = false

    @Volatile
    private var hasNewFrameSignal: Boolean = false

    @Volatile
    private var renderScheduled: Boolean = false

    // Safety fallback: if we ever miss signals on some devices, we still make progress.
    private val watchdogDelayMs = 250L

    // Prevent recursive re-initialization when EGL context is lost.
    private var recreatingEgl = false

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
        varying vec2 v_tex_coord;
        void main() {
            vec4 c = texture2D(u_texture, v_tex_coord);
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
        handler.post {
            bestEffort("registerRendererTid") {
                NesiumNative.nativeRegisterRendererTid(Process.myTid())
            }
            initGLAndLoop()
        }
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

        // Window surface backed by the provided Surface.
        windowSurface = surface
        val surfaceAttribs = intArrayOf(EGL14.EGL_NONE)
        eglSurface =
            EGL14.eglCreateWindowSurface(eglDisplay, config, windowSurface, surfaceAttribs, 0)
        if (eglSurface == EGL14.EGL_NO_SURFACE) {
            throw RuntimeException("eglCreateWindowSurface failed")
        }

        if (!EGL14.eglMakeCurrent(eglDisplay, eglSurface, eglSurface, eglContext)) {
            throw RuntimeException("eglMakeCurrent failed")
        }

        // Initialize the frame-ready wakeup pipe on the GL thread looper.
        ensureFrameSignalPipeInitialized()

        // Disable dithering for a tiny performance win and more deterministic output.
        GLES20.glDisable(GLES20.GL_DITHER)

        // Prefer VSYNC to reduce jitter.
        EGL14.eglSwapInterval(eglDisplay, 1)

        // Cache frame size and format.
        frameW = NesiumNative.nativeFrameWidth()
        frameH = NesiumNative.nativeFrameHeight()

        // Cache DirectByteBuffers for the two persistent planes.
        // The underlying memory is stable for the lifetime of the process.
        planeBuffers[0] = NesiumNative.nativePlaneBuffer(0)
        planeBuffers[1] = NesiumNative.nativePlaneBuffer(1)

        // Compile and link program.
        val vertexShader = loadShader(GLES20.GL_VERTEX_SHADER, vertexShaderCode)
        val fragmentShader = loadShader(GLES20.GL_FRAGMENT_SHADER, fragmentShaderCode)
        program = GLES20.glCreateProgram()
        GLES20.glAttachShader(program, vertexShader)
        GLES20.glAttachShader(program, fragmentShader)
        GLES20.glLinkProgram(program)
        val linkStatus = IntArray(1)
        GLES20.glGetProgramiv(program, GLES20.GL_LINK_STATUS, linkStatus, 0)
        if (linkStatus[0] == 0) {
            val log = GLES20.glGetProgramInfoLog(program)
            GLES20.glDeleteProgram(program)
            program = 0
            throw IllegalStateException("Program link failed: $log")
        }

        aPosition = GLES20.glGetAttribLocation(program, "a_position")
        aTexCoord = GLES20.glGetAttribLocation(program, "a_tex_coord")
        uTexture = GLES20.glGetUniformLocation(program, "u_texture")

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

        GLES20.glTexImage2D(
            GLES20.GL_TEXTURE_2D,
            0,
            GLES20.GL_RGBA,
            frameW,
            frameH,
            0,
            GLES20.GL_RGBA,
            GLES20.GL_UNSIGNED_BYTE,
            null
        )

        // Default viewport to the surface size.
        updateViewportIfNeeded()

        renderLoop()
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
            throw IllegalStateException("Shader compile failed: $log")
        }
        return shader
    }

    private fun ensureFrameSignalPipeInitialized() {
        if (frameSignalRead != null) return

        // Create a simple pipe: Rust writes to `write`, GL thread listens on `read`.
        val pipe = ParcelFileDescriptor.createPipe()
        frameSignalRead = pipe[0]
        frameSignalWrite = pipe[1]

        val readPfd = requireNotNull(frameSignalRead) {
            "Frame signal pipe: read ParcelFileDescriptor is unexpectedly null"
        }
        val writePfd = requireNotNull(frameSignalWrite) {
            "Frame signal pipe: write ParcelFileDescriptor is unexpectedly null"
        }

        frameSignalFd = readPfd.fileDescriptor
        val fd = requireNotNull(frameSignalFd) {
            "Frame signal pipe: failed to obtain read FileDescriptor"
        }

        // Make reads non-blocking so we can drain without risking a stall.
        frameSignalNonBlocking = false
        if (Build.VERSION.SDK_INT >= 30) {
            try {
                val flags = Os.fcntlInt(fd, OsConstants.F_GETFL, 0)
                Os.fcntlInt(fd, OsConstants.F_SETFL, flags or OsConstants.O_NONBLOCK)
                frameSignalNonBlocking = true
            } catch (e: ErrnoException) {
                throw IllegalStateException("Failed to set O_NONBLOCK on frame signal pipe", e)
            }
        }

        // Register an FD listener on THIS looper (the GL thread looper).
        val queue = Looper.myQueue()
        queue.addOnFileDescriptorEventListener(
            fd,
            MessageQueue.OnFileDescriptorEventListener.EVENT_INPUT,
        ) { _, events ->
            if ((events and MessageQueue.OnFileDescriptorEventListener.EVENT_INPUT) != 0) {
                drainFrameSignal()
                onFrameSignal()
            }
            MessageQueue.OnFileDescriptorEventListener.EVENT_INPUT
        }

        // Pass the write-end FD to native. Native should store it and write() a small token per frame.
        NesiumNative.nativeSetFrameSignalFd(writePfd.fd)
    }

    private fun drainFrameSignal() {
        val fd = frameSignalFd ?: return
        val buf = ByteArray(64)
        if (!frameSignalNonBlocking) {
            try {
                Os.read(fd, buf, 0, buf.size)
            } catch (_: ErrnoException) {
                return
            }
            return
        }
        while (true) {
            try {
                val n = Os.read(fd, buf, 0, buf.size)
                if (n <= 0) return
            } catch (e: ErrnoException) {
                // EAGAIN means we've drained all currently available bytes.
                if (e.errno == OsConstants.EAGAIN) return
                Log.w(TAG, "Frame signal pipe read failed (errno=${e.errno})", e)
                return
            }
        }
    }

    private fun onFrameSignal() {
        hasNewFrameSignal = true
        scheduleRender()
    }

    private fun scheduleRender() {
        if (!running) return
        if (renderScheduled) return
        renderScheduled = true
        // Post at front to reduce latency from the signal->render path.
        handler.postAtFrontOfQueue { renderLoop() }
    }

    private fun teardownFrameSignalPipe() {
        val fd = frameSignalFd
        if (fd != null) {
            bestEffort("removeOnFileDescriptorEventListener") {
                Looper.myQueue().removeOnFileDescriptorEventListener(fd)
            }
        }

        // Stop native writes before closing the pipe.
        NesiumNative.nativeSetFrameSignalFd(-1)

        bestEffort("frameSignalRead.close") {
            frameSignalRead?.close()
        }
        bestEffort("frameSignalWrite.close") {
            frameSignalWrite?.close()
        }

        frameSignalRead = null
        frameSignalWrite = null
        frameSignalFd = null
        hasNewFrameSignal = false
        renderScheduled = false
    }

    private fun updateViewportIfNeeded() {
        val wArr = IntArray(1)
        val hArr = IntArray(1)
        EGL14.eglQuerySurface(eglDisplay, eglSurface, EGL14.EGL_WIDTH, wArr, 0)
        EGL14.eglQuerySurface(eglDisplay, eglSurface, EGL14.EGL_HEIGHT, hArr, 0)

        val newW = wArr[0]
        val newH = hArr[0]
        if (newW > 0 && newH > 0 && (newW != surfaceW || newH != surfaceH)) {
            surfaceW = newW
            surfaceH = newH
            GLES20.glViewport(0, 0, surfaceW, surfaceH)
        }
    }

    private fun ensureFrameResourcesUpToDate() {
        val newW = NesiumNative.nativeFrameWidth()
        val newH = NesiumNative.nativeFrameHeight()
        if (newW <= 0 || newH <= 0) return
        if (newW == frameW && newH == frameH) return

        frameW = newW
        frameH = newH

        // Refresh DirectByteBuffers because the native backing store may have been reallocated.
        planeBuffers[0] = NesiumNative.nativePlaneBuffer(0)
        planeBuffers[1] = NesiumNative.nativePlaneBuffer(1)

        // Reallocate GL texture storage for the new size.
        GLES20.glBindTexture(GLES20.GL_TEXTURE_2D, textureId)
        GLES20.glTexImage2D(
            GLES20.GL_TEXTURE_2D,
            0,
            GLES20.GL_RGBA,
            frameW,
            frameH,
            0,
            GLES20.GL_RGBA,
            GLES20.GL_UNSIGNED_BYTE,
            null
        )
    }

    private fun handleSwapFailure() {
        val err = EGL14.eglGetError()
        // EGL_CONTEXT_LOST requires full context recreation.
        if (err == EGL14.EGL_CONTEXT_LOST || err == EGL14.EGL_BAD_CONTEXT) {
            if (!recreatingEgl) {
                recreatingEgl = true
                // Attempt a best-effort re-init. If this fails, the renderer will stop.
                handler.post {
                    try {
                        destroyEglOnly()
                        initGLAndLoop()
                    } catch (t: Throwable) {
                        running = false
                        throw t
                    } finally {
                        recreatingEgl = false
                    }
                }
            }
        }

        // Other EGL errors: keep running but back off slightly.
        handler.postDelayed({ scheduleRender() }, 16)
    }

    private fun destroyEglOnly() {
        if (textureId != 0) {
            val id = textureId
            textureId = 0
            bestEffort("glDeleteTextures($id)") {
                GLES20.glDeleteTextures(1, intArrayOf(id), 0)
            }
        }
        if (program != 0) {
            val id = program
            program = 0
            bestEffort("glDeleteProgram($id)") {
                GLES20.glDeleteProgram(id)
            }
        }

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

        if (releaseSurface) {
            bestEffort("surface.release") {
                surface.release()
            }
        }
        windowSurface = null

        // Reset cached state.
        lastSeq = -1L
        surfaceW = -1
        surfaceH = -1
    }

    private fun renderLoop() {
        // This function is scheduled via `scheduleRender()` (signal-driven).
        renderScheduled = false
        if (!running) return

        val hadSignal = hasNewFrameSignal
        val seq = NesiumNative.nativeFrameSeq()

        if (seq == lastSeq) {
            // If we were signaled but the producer hasn't published the new seq yet, retry soon.
            if (hadSignal) {
                handler.postDelayed({ scheduleRender() }, 1)
            } else {
                // Watchdog fallback: if signals are missed, we still make progress.
                handler.postDelayed({ scheduleRender() }, watchdogDelayMs)
            }
            return
        }

        // We observed a new frame seq.
        hasNewFrameSignal = false
        lastSeq = seq

        ensureFrameResourcesUpToDate()

        val idx = NesiumNative.nativeBeginFrontCopy()
        try {
            val buffer = planeBuffers[idx]
                ?: throw IllegalStateException("Plane buffer is not initialized")
            buffer.position(0)

            GLES20.glBindTexture(GLES20.GL_TEXTURE_2D, textureId)
            GLES20.glTexSubImage2D(
                GLES20.GL_TEXTURE_2D,
                0,
                0,
                0,
                frameW,
                frameH,
                GLES20.GL_RGBA,
                GLES20.GL_UNSIGNED_BYTE,
                buffer
            )

            updateViewportIfNeeded()

            GLES20.glUseProgram(program)
            GLES20.glUniform1i(uTexture, 0)

            vertexData.position(0)
            GLES20.glVertexAttribPointer(aPosition, 2, GLES20.GL_FLOAT, false, 16, vertexData)
            GLES20.glEnableVertexAttribArray(aPosition)

            vertexData.position(2)
            GLES20.glVertexAttribPointer(aTexCoord, 2, GLES20.GL_FLOAT, false, 16, vertexData)
            GLES20.glEnableVertexAttribArray(aTexCoord)

            GLES20.glDrawArrays(GLES20.GL_TRIANGLE_STRIP, 0, 4)

            if (!EGL14.eglSwapBuffers(eglDisplay, eglSurface)) {
                handleSwapFailure()
                return
            }

            // For SurfaceTexture-backed Flutter textures, swapping the EGL window surface enqueues
            // a new buffer to the underlying SurfaceTexture. The Flutter embedding registers an
            // internal OnFrameAvailableListener on that SurfaceTexture and will schedule a redraw.

            // If another frame arrived while we were uploading/swapping, render again.
            if (hasNewFrameSignal) {
                scheduleRender()
            }
        } finally {
            NesiumNative.nativeEndFrontCopy()
        }
    }

    fun dispose(waitForShutdown: Boolean) {
        if (!running) return
        running = false
        val latch = if (waitForShutdown) CountDownLatch(1) else null
        handler.post {
            try {
                destroyEglOnly()
            } finally {
                // Must run on the GL thread looper.
                teardownFrameSignalPipe()

                try {
                    onDispose?.invoke()
                } catch (t: Throwable) {
                    Log.w(TAG, "onDispose callback failed", t)
                }
                thread.quitSafely()
                latch?.countDown()
            }
        }

        if (waitForShutdown && latch != null) {
            try {
                latch.await()
                thread.join(500)
            } catch (_: InterruptedException) {
                Thread.currentThread().interrupt()
            }
        }
    }
}

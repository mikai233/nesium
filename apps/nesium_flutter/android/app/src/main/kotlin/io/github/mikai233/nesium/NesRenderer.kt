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
import android.os.Build
import android.os.Looper
import android.os.MessageQueue
import android.os.ParcelFileDescriptor
import android.system.ErrnoException
import android.system.Os
import android.system.OsConstants
import android.util.Log
import java.io.FileDescriptor
import java.util.concurrent.CountDownLatch


/**
 * A dedicated GL thread that uploads the latest NES frame into a GL texture.
 *
 * This implementation uses an EGL window surface created from a Java `Surface` that wraps the
 * Flutter-provided `SurfaceTexture`. Each swap produces a new buffer that Flutter can consume.
 */
class NesRenderer(
    private val flutterEngine: FlutterEngine,
    private val textureEntry: TextureRegistry.SurfaceTextureEntry,
    private val profilingEnabled: Boolean = false,
) {
    private companion object {
        private const val TAG = "NesRenderer"
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

    // Cached native constants (NES frame size is fixed).
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

    // --- Profiling (optional) ---
    private var profFrames = 0
    private var profLastLogNs = 0L
    private var profSumUploadNs = 0L
    private var profMaxUploadNs = 0L
    private var profSumSwapNs = 0L
    private var profMaxSwapNs = 0L
    private var profSumTotalNs = 0L
    private var profMaxTotalNs = 0L

    private fun recordFrameTiming(
        tStartNs: Long,
        tUploadStartNs: Long,
        tUploadEndNs: Long,
        tSwapStartNs: Long,
        tSwapEndNs: Long,
    ) {
        val pacingThresholdSwapNs = 25_000_000L // ~25ms, likely missed vsync on 60Hz

        val uploadNs = tUploadEndNs - tUploadStartNs
        val swapNs = tSwapEndNs - tSwapStartNs
        val totalNs = tSwapEndNs - tStartNs

        profFrames += 1
        profSumUploadNs += uploadNs
        profSumSwapNs += swapNs
        profSumTotalNs += totalNs
        if (uploadNs > profMaxUploadNs) profMaxUploadNs = uploadNs
        if (swapNs > profMaxSwapNs) profMaxSwapNs = swapNs
        if (totalNs > profMaxTotalNs) profMaxTotalNs = totalNs

        val now = tSwapEndNs
        val shouldLog = (profLastLogNs == 0L) ||
                (now - profLastLogNs >= 1_000_000_000L) ||
                (swapNs >= pacingThresholdSwapNs)

        if (!shouldLog) return

        val avgUploadMs = profSumUploadNs.toDouble() / profFrames / 1_000_000.0
        val avgSwapMs = profSumSwapNs.toDouble() / profFrames / 1_000_000.0
        val avgTotalMs = profSumTotalNs.toDouble() / profFrames / 1_000_000.0
        val maxUploadMs = profMaxUploadNs.toDouble() / 1_000_000.0
        val maxSwapMs = profMaxSwapNs.toDouble() / 1_000_000.0
        val maxTotalMs = profMaxTotalNs.toDouble() / 1_000_000.0

        val msg =
            "frame pacing: avg upload=${"%.2f".format(avgUploadMs)}ms (max ${"%.2f".format(maxUploadMs)}ms), " +
                "avg swap=${"%.2f".format(avgSwapMs)}ms (max ${"%.2f".format(maxSwapMs)}ms), " +
                "avg total=${"%.2f".format(avgTotalMs)}ms (max ${"%.2f".format(maxTotalMs)}ms), " +
                "frames=$profFrames"

        if (profMaxSwapNs >= pacingThresholdSwapNs) {
            Log.w(TAG, msg)
        } else {
            Log.d(TAG, msg)
        }

        profLastLogNs = now
        profFrames = 0
        profSumUploadNs = 0L
        profMaxUploadNs = 0L
        profSumSwapNs = 0L
        profMaxSwapNs = 0L
        profSumTotalNs = 0L
        profMaxTotalNs = 0L
    }

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
            try {
                Looper.myQueue().removeOnFileDescriptorEventListener(fd)
            } catch (_: Throwable) {
                // Best-effort cleanup.
            }
        }

        // Stop native writes before closing the pipe.
        NesiumNative.nativeSetFrameSignalFd(-1)

        try {
            frameSignalRead?.close()
        } catch (_: Throwable) {
        }
        try {
            frameSignalWrite?.close()
        } catch (_: Throwable) {
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
        try {
            if (textureId != 0) {
                GLES20.glDeleteTextures(1, intArrayOf(textureId), 0)
                textureId = 0
            }
            if (program != 0) {
                GLES20.glDeleteProgram(program)
                program = 0
            }
        } catch (_: Throwable) {
            // Best-effort cleanup; ignore GL errors during teardown.
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

        windowSurface?.release()
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

        val tStartNs = if (profilingEnabled) System.nanoTime() else 0L
        val idx = NesiumNative.nativeBeginFrontCopy()
        try {
            val buffer = planeBuffers[idx]
                ?: throw IllegalStateException("Plane buffer is not initialized")
            buffer.position(0)

            val tUploadStartNs = if (profilingEnabled) System.nanoTime() else 0L
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
            val tUploadEndNs = if (profilingEnabled) System.nanoTime() else 0L

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

            val tSwapStartNs = if (profilingEnabled) System.nanoTime() else 0L
            if (!EGL14.eglSwapBuffers(eglDisplay, eglSurface)) {
                handleSwapFailure()
                return
            }
            val tSwapEndNs = if (profilingEnabled) System.nanoTime() else 0L

            // For SurfaceTexture-backed Flutter textures, swapping the EGL window surface enqueues
            // a new buffer to the underlying SurfaceTexture. The Flutter embedding registers an
            // internal OnFrameAvailableListener on that SurfaceTexture and will schedule a redraw.

            // If another frame arrived while we were uploading/swapping, render again.
            if (hasNewFrameSignal) {
                scheduleRender()
            }

            if (profilingEnabled) {
                recordFrameTiming(
                    tStartNs = tStartNs,
                    tUploadStartNs = tUploadStartNs,
                    tUploadEndNs = tUploadEndNs,
                    tSwapStartNs = tSwapStartNs,
                    tSwapEndNs = tSwapEndNs,
                )
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

                textureEntry.release()
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

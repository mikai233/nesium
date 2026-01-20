// web/nes/nes_worker.js
// ES module worker. Create it with:
//   new Worker('nes/nes_worker.js', { type: 'module' })

// Use dynamic import so missing/invalid wasm-pack output can be reported back to the UI,
// instead of failing the entire worker module load with an opaque "Worker error".
let wasm = null;
let wasmMemory = null;
let wasmInitPromise = null;

let nes = null;

// Video
let canvas = null;
let ctx2d = null;
let width = 256;
let height = 240;
let imageData = null; // ImageData holding RGBA bytes

// Run loop
let running = false;
let emitAudio = true;
let timer = null;
let nextFrameAt = 0;

// Timing
// NTSC NES runs at ~60.0988 FPS. We use this as the "exact" default.
// The desktop runtime supports region-specific values; the web worker currently
// assumes NTSC for pacing.
const EXACT_NTSC_FPS = 60.0988;
let targetFps = EXACT_NTSC_FPS;
let integerFpsMode = false;
let rewinding = false;
let fastForwarding = false;
let fastForwardSpeedPercent = 100;
let rewindSpeedPercent = 100;
let baseFps = EXACT_NTSC_FPS;

// Input
let padBaseMasks = new Map(); // port -> bits
let padTurboMasks = new Map(); // port -> bits
let hasTurboInput = false;

// Turbo timing (frames)
let turboOnFrames = 2;
let turboOffFrames = 2;
let turboPhaseOn = true;
let turboPhaseRemaining = turboOnFrames;

function updateTargetFps() {
    let multiplier = 1;
    if (fastForwarding) {
        const speed = Math.max(100, Math.min(1000, fastForwardSpeedPercent | 0));
        multiplier = speed / 100;
    } else if (rewinding) {
        const speed = Math.max(100, Math.min(1000, rewindSpeedPercent | 0));
        multiplier = speed / 100;
    }
    targetFps = baseFps * multiplier;
}

function postError(message) {
    try {
        postMessage({ type: "error", message: String(message) });
    } catch (_) {
        // ignore
    }
}

function postLog(message) {
    try {
        postMessage({ type: "log", message: String(message) });
    } catch (_) {
        // ignore
    }
}

self.addEventListener("error", (e) => {
    // Some browser errors are opaque; still attempt to forward details.
    postError(e?.message ?? e);
});

self.addEventListener("unhandledrejection", (e) => {
    postError(e?.reason ?? e);
});

async function ensureWasmLoaded() {
    if (wasm) return wasm;
    postLog("loading wasm module: ./pkg/nesium_wasm.js");
    wasm = await import("./pkg/nesium_wasm.js");
    postLog("wasm module loaded");
    return wasm;
}

async function ensureWasmInitialized() {
    if (wasmInitPromise) return wasmInitPromise;

    wasmInitPromise = (async () => {
        const mod = await ensureWasmLoaded();
        postLog("initializing wasm...");
        const exports = await mod.default();
        wasmMemory = exports.memory;
        postLog("wasm initialized");
        mod.init_panic_hook();
        return mod;
    })();

    try {
        return await wasmInitPromise;
    } catch (e) {
        // Allow retry on subsequent calls.
        wasmInitPromise = null;
        throw e;
    }
}

// Utility: stop loop
function stopLoop() {
    if (timer) {
        clearTimeout(timer);
        timer = null;
    }
}

function computeTurboBits(port) {
    if (!turboPhaseOn) return 0;
    return (padTurboMasks.get(port) ?? 0) & 0xff;
}

function computeEffectivePadBits(port) {
    const base = (padBaseMasks.get(port) ?? 0) & 0xff;
    const turbo = computeTurboBits(port);
    return (base | turbo) & 0xff;
}

function applyPad(port) {
    const bits = computeEffectivePadBits(port);
    if (bits !== 0 && rewinding) {
        // Any button press stops rewind
        rewinding = false;
        if (typeof nes.set_rewinding === "function") {
            nes.set_rewinding(false);
        }
    }
    nes.set_pad(port, bits);
}

function applyAllPads() {
    // NES has two controller ports (0 and 1).
    for (let port = 0; port < 2; port += 1) {
        applyPad(port);
    }
}

function resetTurboPhase() {
    turboPhaseOn = true;
    turboPhaseRemaining = Math.max(1, turboOnFrames | 0);
}

function advanceTurboPhase() {
    const on = Math.max(1, turboOnFrames | 0);
    const off = Math.max(1, turboOffFrames | 0);
    if (turboPhaseRemaining <= 0) {
        turboPhaseOn = !turboPhaseOn;
        turboPhaseRemaining = turboPhaseOn ? on : off;
    }
    turboPhaseRemaining -= 1;
}

function recomputeHasTurboInput() {
    const t0 = padTurboMasks.get(0) ?? 0;
    const t1 = padTurboMasks.get(1) ?? 0;
    hasTurboInput = (((t0 | t1) & 0xff) !== 0);
}

// Render one or more frames (catch up if behind)
function tick() {
    if (!running && !rewinding) {
        stopLoop();
        return;
    }

    try {
        const now = performance.now();
        if (nextFrameAt === 0) nextFrameAt = now;

        const frameDuration = 1000 / targetFps;
        let framesRun = 0;
        // Limit catch-up to avoid "spiral of death" if the worker is too slow.
        // 10 frames is ~166ms at 60fps, enough to handle most bursts.
        const maxCatchUp = 10;

        while (now >= nextFrameAt && framesRun < maxCatchUp) {
            if (!rewinding) {
                // Always sync pads during forward simulation to avoid ghost inputs 
                // restored from snapshots.
                applyAllPads();
                if (hasTurboInput) {
                    advanceTurboPhase();
                }
            }

            // WASM nes.run_frame handles internal state management based on its own internal 'rewinding' flag.
            nes.run_frame(emitAudio && !rewinding);

            // ----- Audio: post interleaved stereo f32 (must do for every frame) -----
            if (emitAudio && !rewinding) {
                const aptr = nes.audio_ptr();
                const alen = nes.audio_len();

                if (alen > 0) {
                    const audioView = new Float32Array(wasmMemory.buffer, aptr, alen);
                    const copy = new Float32Array(alen);
                    copy.set(audioView);
                    postMessage({ type: "audio", buffer: copy.buffer }, [copy.buffer]);
                }
            }

            nextFrameAt += frameDuration;
            framesRun++;
        }

        if (framesRun > 0) {
            // ----- Video: copy RGBA bytes from WASM memory into ImageData (only once per tick) -----
            const fptr = nes.frame_ptr();
            const flen = nes.frame_len(); // should be width*height*4
            const rgbaView = new Uint8Array(wasmMemory.buffer, fptr, flen);

            // Copy into ImageData.data (Uint8ClampedArray). This avoids lifetime issues.
            imageData.data.set(rgbaView);
            ctx2d.putImageData(imageData, 0, 0);
        }

        // If we are STILL behind (extreme stall), reset the deadline.
        if (now - nextFrameAt > 200) {
            nextFrameAt = now;
        }

        const delay = Math.max(0, nextFrameAt - now);
        timer = setTimeout(tick, delay);
    } catch (e) {
        running = false;
        stopLoop();
        postError(e);
    }
}

async function handleInit(msg) {
    // msg: { canvas, width, height, sampleRate }
    width = msg.width ?? 256;
    height = msg.height ?? 240;

    // Initialize wasm and panic hook (can be pre-warmed via {type:'preload'})
    const mod = await ensureWasmInitialized();

    // Create NES instance with sampleRate (AudioContext.sampleRate from main thread)
    const sr = msg.sampleRate ?? 48000;
    nes = new mod.WasmNes(sr);

    // Default pacing: exact NES FPS with no integer-FPS audio stretch.
    baseFps = EXACT_NTSC_FPS;
    integerFpsMode = false;
    fastForwarding = false;
    fastForwardSpeedPercent = 100;
    rewindSpeedPercent = 100;
    updateTargetFps();
    if (typeof nes.reset_audio_integer_fps_scale === "function") {
        nes.reset_audio_integer_fps_scale();
    }

    resetTurboPhase();
    recomputeHasTurboInput();

    // Video context
    canvas = msg.canvas;
    ctx2d = canvas.getContext("2d", {
        alpha: false,
        desynchronized: true,
        willReadFrequently: false,
    });

    // Make sure we render at native resolution (CSS scaling can be done on main thread)
    canvas.width = width;
    canvas.height = height;

    // Create a reusable ImageData
    imageData = ctx2d.createImageData(width, height);

    // Optional: turn off smoothing if you ever scale via drawImage inside worker
    ctx2d.imageSmoothingEnabled = false;

    postMessage({ type: "ready", width, height, sampleRate: sr });
}

function ensureReady() {
    if (!nes || !ctx2d || !imageData) {
        throw new Error("Worker not initialized. Send {type:'init'} first.");
    }
}

onmessage = async (ev) => {
    const msg = ev.data;
    if (!msg || typeof msg.type !== "string") return;

    try {
        if (msg.type === "preload") {
            try {
                await ensureWasmInitialized();
            } catch (e) {
                // Preload is best-effort. Do not surface a persistent UI error
                // until the real {type:'init'} flow needs the wasm.
                postLog(`wasm preload failed: ${e?.message ?? e}`);
            }
            return;
        }

        if (msg.type === "init") {
            await handleInit(msg);
            return;
        }

        if (msg.type === "cmd") {
            ensureReady();

            switch (msg.cmd) {
                case "loadRom": {
                    // msg.rom is ArrayBuffer
                    const romBytes = new Uint8Array(msg.rom);
                    nes.load_rom(romBytes);

                    let hash = null;
                    if (typeof nes.get_rom_hash === "function") {
                        const hashBytes = nes.get_rom_hash(romBytes);
                        hash = Array.from(hashBytes);
                    }

                    postMessage({ type: "romLoaded", hash: hash });
                    break;
                }

                case "saveState": {
                    if (typeof nes.save_state !== "function") {
                        throw new Error("Missing wasm export: save_state. Rebuild `web/nes/pkg`.");
                    }
                    const data = nes.save_state();
                    postMessage({
                        type: "saveStateResult",
                        data: data.buffer,
                        requestId: msg.requestId
                    }, [data.buffer]);
                    break;
                }

                case "loadState": {
                    if (typeof nes.load_state !== "function") {
                        throw new Error("Missing wasm export: load_state. Rebuild `web/nes/pkg`.");
                    }
                    const bytes = new Uint8Array(msg.data);
                    nes.load_state(bytes);
                    postMessage({
                        type: "loadStateResult",
                        success: true,
                        requestId: msg.requestId
                    });
                    break;
                }

                case "loadTasMovie": {
                    if (typeof nes.load_tas_movie !== "function") {
                        throw new Error("Missing wasm export: load_tas_movie. Rebuild `web/nes/pkg`.");
                    }
                    nes.load_tas_movie(msg.data);
                    // Match native behavior: reset pacing and wake loop
                    nextFrameAt = 0;
                    if (!timer) tick();
                    break;
                }

                case "setRewindConfig": {
                    const enabled = !!msg.enabled;
                    const capacity = Number(msg.capacity);
                    if (typeof nes.set_rewind_config !== "function") {
                        throw new Error("Missing wasm export: set_rewind_config. Rebuild `web/nes/pkg`.");
                    }
                    nes.set_rewind_config(enabled, capacity);
                    break;
                }

                case "setRewinding": {
                    const nextRewinding = !!msg.rewinding;
                    if (typeof nes.set_rewinding !== "function") {
                        throw new Error("Missing wasm export: set_rewinding. Rebuild `web/nes/pkg`.");
                    }
                    nes.set_rewinding(nextRewinding);

                    if (nextRewinding !== rewinding) {
                        rewinding = nextRewinding;
                        updateTargetFps();
                        nextFrameAt = 0; // Reset pacing to avoid delay
                        if (nextRewinding) {
                            if (!timer) tick(); // Wake up the loop
                        } else {
                            // Exiting rewind: force immediate input sync to clear restored state
                            applyAllPads();
                        }
                    }
                    break;
                }

                case "setFastForwarding": {
                    const nextFastForwarding = !!msg.fastForwarding;
                    if (nextFastForwarding !== fastForwarding) {
                        fastForwarding = nextFastForwarding;
                        updateTargetFps();
                        nextFrameAt = 0;
                    }
                    break;
                }

                case "setFastForwardSpeed": {
                    const nextSpeed = msg.speedPercent ?? 100;
                    fastForwardSpeedPercent = Math.max(100, Math.min(1000, nextSpeed | 0));
                    updateTargetFps();
                    nextFrameAt = 0;
                    break;
                }

                case "setRewindSpeed": {
                    const nextSpeed = msg.speedPercent ?? 100;
                    rewindSpeedPercent = Math.max(100, Math.min(1000, nextSpeed | 0));
                    if (typeof nes.set_rewind_speed === "function") {
                        nes.set_rewind_speed(rewindSpeedPercent);
                    }
                    updateTargetFps();
                    nextFrameAt = 0;
                    break;
                }

                case "run": {
                    running = true;
                    emitAudio = msg.emitAudio ?? true;
                    nextFrameAt = 0;
                    stopLoop();
                    tick();
                    postMessage({ type: "running", value: true });
                    break;
                }

                case "pause": {
                    running = false;
                    stopLoop();
                    postMessage({ type: "running", value: false });
                    break;
                }

                case "step": {
                    // Single-frame step (no loop)
                    running = false;
                    stopLoop();
                    emitAudio = msg.emitAudio ?? true;

                    if (hasTurboInput) {
                        advanceTurboPhase();
                        applyAllPads();
                    }
                    nes.run_frame(emitAudio);

                    const fptr = nes.frame_ptr();
                    const flen = nes.frame_len();
                    const rgbaView = new Uint8Array(wasmMemory.buffer, fptr, flen);
                    imageData.data.set(rgbaView);
                    ctx2d.putImageData(imageData, 0, 0);

                    if (emitAudio) {
                        const aptr = nes.audio_ptr();
                        const alen = nes.audio_len();
                        if (alen > 0) {
                            const audioView = new Float32Array(wasmMemory.buffer, aptr, alen);
                            const copy = new Float32Array(alen);
                            copy.set(audioView);
                            postMessage({ type: "audio", buffer: copy.buffer }, [copy.buffer]);
                        }
                    }

                    postMessage({ type: "stepped" });
                    break;
                }

                case "powerOnReset": {
                    nes.power_on_reset();
                    postMessage({ type: "reset", kind: "powerOn" });
                    break;
                }

                case "softReset": {
                    nes.soft_reset();
                    postMessage({ type: "reset", kind: "soft" });
                    break;
                }

                case "setPad": {
                    const port = msg.port ?? 0;
                    const bits = msg.bits ?? 0;
                    padBaseMasks.set(port, bits & 0xff);
                    applyPad(port);
                    break;
                }

                case "setTurboMask": {
                    const port = msg.port ?? 0;
                    const bits = msg.bits ?? 0;
                    const next = bits & 0xff;
                    const prev = padTurboMasks.get(port) ?? 0;
                    padTurboMasks.set(port, next);
                    if (((prev | 0) & 0xff) === 0 && next !== 0) {
                        resetTurboPhase();
                    }
                    recomputeHasTurboInput();
                    applyPad(port);
                    break;
                }

                case "setTurboTiming": {
                    const onFrames = msg.onFrames ?? 2;
                    const offFrames = msg.offFrames ?? 2;
                    turboOnFrames = Math.max(1, onFrames | 0);
                    turboOffFrames = Math.max(1, offFrames | 0);
                    resetTurboPhase();
                    if (hasTurboInput) applyAllPads();
                    break;
                }

                case "setIntegerFpsMode": {
                    const enabled = !!msg.enabled;
                    integerFpsMode = enabled;
                    if (enabled) {
                        baseFps = 60;
                        if (typeof nes.set_audio_integer_fps_scale !== "function") {
                            throw new Error("Missing wasm export: set_audio_integer_fps_scale. Rebuild `web/nes/pkg`.");
                        }
                        nes.set_audio_integer_fps_scale(60 / EXACT_NTSC_FPS);
                    } else {
                        baseFps = EXACT_NTSC_FPS;
                        if (typeof nes.reset_audio_integer_fps_scale === "function") {
                            nes.reset_audio_integer_fps_scale();
                        }
                    }
                    updateTargetFps();
                    break;
                }

                case "setPalettePreset": {
                    const kind = String(msg.kind ?? "");
                    if (typeof nes.set_palette_preset !== "function") {
                        throw new Error("Missing wasm export: set_palette_preset. Rebuild `web/nes/pkg`.");
                    }
                    nes.set_palette_preset(kind);
                    break;
                }

                case "setPalettePalData": {
                    const raw = msg.data;
                    if (!raw) {
                        throw new Error("Missing palette data");
                    }
                    const bytes = raw instanceof ArrayBuffer ? new Uint8Array(raw) : new Uint8Array(raw);
                    if (typeof nes.set_palette_pal_data !== "function") {
                        throw new Error("Missing wasm export: set_palette_pal_data. Rebuild `web/nes/pkg`.");
                    }
                    nes.set_palette_pal_data(bytes);
                    break;
                }

                default:
                    postMessage({ type: "error", message: `Unknown cmd: ${msg.cmd}` });
            }
        }
    } catch (e) {
        postError(e);
    }
};

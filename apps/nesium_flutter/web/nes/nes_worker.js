// web/nes/nes_worker.js
// ES module worker. Create it with:
//   new Worker('nes/nes_worker.js', { type: 'module' })

// Use dynamic import so missing/invalid wasm-pack output can be reported back to the UI,
// instead of failing the entire worker module load with an opaque "Worker error".
let wasm = null;
let wasmMemory = null;

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

// Input
let padBaseMasks = new Map(); // port -> bits
let padTurboMasks = new Map(); // port -> bits
let hasTurboInput = false;

// Turbo timing (frames)
let turboOnFrames = 2;
let turboOffFrames = 2;
let turboPhaseOn = true;
let turboPhaseRemaining = turboOnFrames;

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

// Render one frame (and optionally send audio)
function tick() {
    if (!running && !rewinding) {
        stopLoop();
        return;
    }

    try {
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

        // ----- Video: copy RGBA bytes from WASM memory into ImageData -----
        const fptr = nes.frame_ptr();
        const flen = nes.frame_len(); // should be width*height*4
        const rgbaView = new Uint8Array(wasmMemory.buffer, fptr, flen);

        // Copy into ImageData.data (Uint8ClampedArray). This avoids lifetime issues.
        imageData.data.set(rgbaView);
        ctx2d.putImageData(imageData, 0, 0);

        // ----- Audio: post interleaved stereo f32 -----
        if (emitAudio && !rewinding) {
            const aptr = nes.audio_ptr();
            const alen = nes.audio_len();

            if (alen > 0) {
                const audioView = new Float32Array(wasmMemory.buffer, aptr, alen);

                // Copy to a transferable buffer so the main thread/worklet gets a stable chunk.
                const copy = new Float32Array(alen);
                copy.set(audioView);

                postMessage({ type: "audio", buffer: copy.buffer }, [copy.buffer]);
            }
        }
    } catch (e) {
        running = false;
        stopLoop();
        postError(e);
        return;
    }

    // Drift-corrected pacing.
    const now = performance.now();
    if (nextFrameAt === 0) nextFrameAt = now;
    nextFrameAt += 1000 / targetFps;

    // If we were stalled for too long (tab jank / scheduler delay), do not try to
    // "catch up" by running frames in a tight loop: that can build up a large
    // audio buffer and make audio lag behind video.
    if (now - nextFrameAt > 250) {
        nextFrameAt = now;
    }

    const delay = Math.max(0, nextFrameAt - now);
    timer = setTimeout(tick, delay);
}

async function handleInit(msg) {
    // msg: { canvas, width, height, sampleRate }
    width = msg.width ?? 256;
    height = msg.height ?? 240;

    // Initialize wasm and panic hook
    const mod = await ensureWasmLoaded();
    postLog("initializing wasm...");
    const exports = await mod.default();
    wasmMemory = exports.memory;
    postLog("wasm initialized");
    mod.init_panic_hook();

    // Create NES instance with sampleRate (AudioContext.sampleRate from main thread)
    const sr = msg.sampleRate ?? 48000;
    nes = new mod.WasmNes(sr);

    // Default pacing: exact NES FPS with no integer-FPS audio stretch.
    targetFps = EXACT_NTSC_FPS;
    integerFpsMode = false;
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
                        nextFrameAt = 0; // Reset pacing to avoid delay
                        if (nextRewinding) {
                            if (!timer) tick(); // Wake up the loop
                        } else {
                            // Exiting rewind: force immediate input sync to clear restored state
                            applyAllPads();
                        }
                    }
                    rewinding = nextRewinding;
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
                        targetFps = 60;
                        if (typeof nes.set_audio_integer_fps_scale !== "function") {
                            throw new Error("Missing wasm export: set_audio_integer_fps_scale. Rebuild `web/nes/pkg`.");
                        }
                        nes.set_audio_integer_fps_scale(60 / EXACT_NTSC_FPS);
                    } else {
                        targetFps = EXACT_NTSC_FPS;
                        if (typeof nes.reset_audio_integer_fps_scale === "function") {
                            nes.reset_audio_integer_fps_scale();
                        }
                    }
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

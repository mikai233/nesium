// web/nes/nes_worker.js
// ES module worker. Create it with:
//   new Worker('nes/nes_worker.js', { type: 'module' })

import init, { WasmNes, memory, init_panic_hook } from "./pkg/nesium_wasm.js";

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

// Utility: stop loop
function stopLoop() {
    if (timer) {
        clearTimeout(timer);
        timer = null;
    }
}

// Render one frame (and optionally send audio)
function tick() {
    if (!running) return;

    try {
        nes.run_frame(emitAudio);

        // ----- Video: copy RGBA bytes from WASM memory into ImageData -----
        const fptr = nes.frame_ptr();
        const flen = nes.frame_len(); // should be width*height*4
        const rgbaView = new Uint8Array(memory.buffer, fptr, flen);

        // Copy into ImageData.data (Uint8ClampedArray). This avoids lifetime issues.
        imageData.data.set(rgbaView);
        ctx2d.putImageData(imageData, 0, 0);

        // ----- Audio: post interleaved stereo f32 -----
        if (emitAudio) {
            const aptr = nes.audio_ptr();
            const alen = nes.audio_len();

            if (alen > 0) {
                const audioView = new Float32Array(memory.buffer, aptr, alen);

                // Copy to a transferable buffer so the main thread/worklet gets a stable chunk.
                const copy = new Float32Array(alen);
                copy.set(audioView);

                postMessage({ type: "audio", buffer: copy.buffer }, [copy.buffer]);
            }
        }
    } catch (e) {
        running = false;
        stopLoop();
        postMessage({ type: "error", message: String(e) });
        return;
    }

    // Simple 60fps pacing (drift-corrected)
    const now = performance.now();
    if (nextFrameAt === 0) nextFrameAt = now;
    nextFrameAt += 1000 / 60;

    const delay = Math.max(0, nextFrameAt - now);
    timer = setTimeout(tick, delay);
}

async function handleInit(msg) {
    // msg: { canvas, width, height, sampleRate }
    width = msg.width ?? 256;
    height = msg.height ?? 240;

    // Initialize wasm and panic hook
    await init();
    init_panic_hook();

    // Create NES instance with sampleRate (AudioContext.sampleRate from main thread)
    const sr = msg.sampleRate ?? 48000;
    nes = new WasmNes(sr);

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
                    postMessage({ type: "romLoaded" });
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

                    nes.run_frame(emitAudio);

                    const fptr = nes.frame_ptr();
                    const flen = nes.frame_len();
                    const rgbaView = new Uint8Array(memory.buffer, fptr, flen);
                    imageData.data.set(rgbaView);
                    ctx2d.putImageData(imageData, 0, 0);

                    if (emitAudio) {
                        const aptr = nes.audio_ptr();
                        const alen = nes.audio_len();
                        if (alen > 0) {
                            const audioView = new Float32Array(memory.buffer, aptr, alen);
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

                default:
                    postMessage({ type: "error", message: `Unknown cmd: ${msg.cmd}` });
            }
        }
    } catch (e) {
        postMessage({ type: "error", message: String(e) });
    }
};
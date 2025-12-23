// web/nes/audio_main.js
// Main-thread audio helper.
// Must be started from a user gesture (browser autoplay policy).

let audioCtx = null;
let audioNode = null;

/**
 * Start WebAudio + AudioWorklet.
 * Call this from a user interaction (click/tap).
 */
export async function startAudio() {
    if (audioCtx && audioNode) {
        // Already started
        return { sampleRate: audioCtx.sampleRate };
    }

    audioCtx = new AudioContext({ latencyHint: "interactive" });
    await audioCtx.audioWorklet.addModule("nes/audio_worklet.js");

    audioNode = new AudioWorkletNode(audioCtx, "nes-audio", {
        numberOfOutputs: 1,
        outputChannelCount: [2],
    });

    audioNode.connect(audioCtx.destination);

    // Must resume (autoplay policy)
    await audioCtx.resume();

    return { sampleRate: audioCtx.sampleRate };
}

/**
 * Push an interleaved stereo Float32Array chunk into the worklet.
 * You can pass either a Float32Array or an ArrayBuffer.
 */
export function pushAudioChunk(chunk) {
    if (!audioNode) return;
    const arr = chunk instanceof Float32Array ? chunk : new Float32Array(chunk);
    audioNode.port.postMessage(arr);
}

/**
 * Convenience: attach a Worker and auto-feed any {type:'audio', buffer:ArrayBuffer} messages.
 */
export function attachWorker(worker) {
    worker.addEventListener("message", (e) => {
        const msg = e.data;
        if (msg && msg.type === "audio" && msg.buffer) {
            pushAudioChunk(msg.buffer);
        }
    });
}

/**
 * Get current AudioContext sample rate (after startAudio()).
 */
export function getSampleRate() {
    return audioCtx ? audioCtx.sampleRate : null;
}
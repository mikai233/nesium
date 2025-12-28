// web/nes/audio_worklet.js
// Loaded via:
//   await audioContext.audioWorklet.addModule('nes/audio_worklet.js');

class NesAudioProcessor extends AudioWorkletProcessor {
    constructor() {
        super();

        // Ring buffer sized by the actual output sample rate.
        // Capacity ~1 second stereo.
        const sr = sampleRate || 48000;
        this.capacity = Math.max(2048, Math.floor(sr * 2));
        this.ring = new Float32Array(this.capacity);
        this.read = 0;
        this.write = 0;
        this.available = 0;

        // Keep audio latency bounded by trimming buffered samples when the worker
        // catches up after a jank spike.
        //
        // target ~= 120ms, clamp between 40ms and 250ms.
        const targetSeconds = 0.12;
        const minSeconds = 0.04;
        const maxSeconds = 0.25;
        this.targetAvailable = Math.floor(sr * 2 * targetSeconds);
        this.minAvailable = Math.floor(sr * 2 * minSeconds);
        this.maxAvailable = Math.floor(sr * 2 * maxSeconds);

        this.port.onmessage = (e) => {
            const chunk = e.data;
            if (!chunk) return;

            // Accept either Float32Array (or any TypedArray view) or raw ArrayBuffer.
            if (chunk instanceof ArrayBuffer) {
                this.push(new Float32Array(chunk));
                return;
            }

            if (ArrayBuffer.isView(chunk) && chunk.length != null) {
                this.push(chunk);
            }
        };
    }

    push(chunk) {
        // chunk is interleaved stereo: L R L R ...
        for (let i = 0; i < chunk.length; i++) {
            this.ring[this.write] = chunk[i];
            this.write = (this.write + 1) % this.capacity;

            if (this.available < this.capacity) {
                this.available++;
            } else {
                // Overrun: drop oldest
                this.read = (this.read + 1) % this.capacity;
            }
        }

        // If we have too much buffered audio, drop oldest samples to keep latency small.
        if (this.available > this.maxAvailable) {
            const drop = this.available - this.targetAvailable;
            if (drop > 0) {
                this.read = (this.read + drop) % this.capacity;
                this.available -= drop;
            }
        }
    }

    pullStereo(outL, outR) {
        const frames = outL.length;
        for (let i = 0; i < frames; i++) {
            if (this.available >= 2) {
                const l = this.ring[this.read];
                this.read = (this.read + 1) % this.capacity;
                const r = this.ring[this.read];
                this.read = (this.read + 1) % this.capacity;
                this.available -= 2;

                outL[i] = l;
                outR[i] = r;
            } else {
                // Underflow: silence
                outL[i] = 0;
                outR[i] = 0;
            }
        }
    }

    process(_inputs, outputs) {
        const out = outputs[0];
        const outL = out[0];
        const outR = out[1] ?? out[0]; // mono fallback

        this.pullStereo(outL, outR);
        return true;
    }
}

registerProcessor("nes-audio", NesAudioProcessor);

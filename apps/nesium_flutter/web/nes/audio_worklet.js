// web/nes/audio_worklet.js
// Loaded via:
//   await audioContext.audioWorklet.addModule('nes/audio_worklet.js');

class NesAudioProcessor extends AudioWorkletProcessor {
    constructor() {
        super();

        // ~1 second stereo ring buffer at 48kHz => 48000 * 2 samples
        // Make it larger if you want more jitter tolerance.
        this.capacity = 48000 * 2;
        this.ring = new Float32Array(this.capacity);
        this.read = 0;
        this.write = 0;
        this.available = 0;

        this.port.onmessage = (e) => {
            const chunk = e.data;
            // We expect Float32Array. If you send ArrayBuffer, wrap it before posting.
            if (chunk && chunk.length != null) {
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
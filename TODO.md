# TODO: NES Audio System – Mesen2 Parity

High-level plan: first make normal gameplay audio behaviour match Mesen2 as closely as possible (no obvious clipping, consistent loudness/balance), then gradually cover extreme scenarios, configuration options, and tooling.

## Phase 0 – APU/DMC Scheduling and Timing (match NesApu/DeltaModulationChannel)

Goal: bring the internal APU timing model (especially DMC) much closer to Mesen2’s `NesApu` + `DeltaModulationChannel`, so that `apu_debug.raw` aligns with `apu_debug_mesen.raw` not only statistically, but also around timing-sensitive DMC drums (e.g., Shadow of the Ninja intro cutscene).

### 0.1 – Introduce an APU “range runner” like NesApu::Run/Exec

- **Problem today**
  - `Apu::clock_with_reader` in `crates/nesium-core/src/apu.rs` runs the entire APU one CPU tick at a time, and `push_audio_levels` feeds deltas into `NesSoundMixer` based on “current per-channel level”.
  - Mesen2 uses `NesApu::_currentCycle` / `_previousCycle` and `NesApu::Run(cyclesToRun)` to advance the APU over a range of cycles, with each channel’s `Run(targetCycle)` using an `ApuTimer` to generate per-channel audio deltas at precise timestamps.

- **Target design**
  - Add an internal APU cycle counter and “range runner”:
    - Track `apu_cycle_current` and `apu_cycle_previous` inside `Apu`.
    - Expose `fn run_until(&mut self, target_cycle: u64, reader: &mut impl FnMut(u16) -> u8, mixer: Option<&mut NesSoundMixer>)`.
    - `run_until` should:
      - Use the frame counter to schedule quarter/half frame events between `apu_cycle_previous..target_cycle` (mirroring `FrameCounter::Run`).
      - Call each channel’s `run(target_cycle, mixer)` to emit audio deltas over the interval instead of one-cycle-at-a-time ticking.

- **Migration steps**
  1. Introduce a thin “range mode” behind a feature flag or internal helper while keeping the existing per-cycle `clock_with_reader` path for compatibility.
  2. Start by using `run_until` only from `Nes::run_frame_with_audio` / `step_dot_with_audio`:
     - For each dot where `(dot_counter + 2) % 3 == 0`, compute the corresponding CPU/APU cycle and call `apu.run_until(cycle, reader, Some(&mut mixer))`.
  3. Once stable, deprecate the per-cycle `clock_with_mixer` path in favour of the range-based one.

### 0.2 – Per-channel timers and Run(targetCycle) (Square/Triangle/Noise)

- **Problem today**
  - Pulse/triangle/noise channels in nesium use simple per-cycle timers (e.g., `clock_timer` decrementing a period counter), with audio deltas computed once per APU cycle from `output()` and mixed in `push_audio_levels`.
  - Mesen2’s `SquareChannel::Run`, `TriangleChannel::Run`, `NoiseChannel::Run` wrap their counters in an `ApuTimer` that:
    - Receives a `targetCycle` (CPU clock).
    - Emits one or more “ticks” with precise cycle-aligned output changes via `_timer.AddOutput(output)`.

- **Target design**
  - Introduce a small Rust `ApuTimer` helper in `nesium-core`:
    - Fields: `previous_cycle: u64`, `timer: u16`, `period: u16`, `last_output: i8` (or u8/f32 as needed).
    - Methods mirroring Mesen2:
      - `fn reset(&mut self, soft_reset: bool)`.
      - `fn set_period(&mut self, p: u16)` / `fn set_timer(&mut self, value: u16)`.
      - `fn run<F>(&mut self, target_cycle: u64, mut on_tick: F)` where `F: FnMut(u64)`:
        - While “cycles to run > timer”，调用 `on_tick(current_cycle)`，重装 timer 并更新 `previous_cycle`.
  - For each core channel (pulse1/2, triangle, noise):
    - Add a `timer: ApuTimer` field.
    - Refactor the existing per-cycle `clock_timer` into a `run(target_cycle, &mut NesSoundMixer)` that:
      - Calls `timer.run(target_cycle, |tick_cycle| { update internal phase; compute new output; mixer.add_delta(channel, tick_cycle as i64, delta); })`.

- **Migration steps**
  1. Implement `ApuTimer` in Rust, with unit tests that mirror `Core/NES/APU/ApuTimer` semantics (period/timer/reload behaviour).
  2. Port `SquareChannel::Run` semantics into `Pulse`:
     - Duty sequencing, sweep/length envelope tick points remain driven by the frame counter.
     - Timer-driven part becomes `Pulse::run(target_cycle, mixer)` using `ApuTimer`.
  3. Repeat for `Triangle` and `Noise`, validating that:
     - For a simple test pattern, `NesSoundMixer::mix_channels_stereo` still matches Mesen2’s `GetOutputVolume()` scaling tests.

### 0.3 – DMC state machine: _needToRun / transferStartDelay / disableDelay

- **Problem today**
  - Our `Dmc` in `crates/nesium-core/src/apu/dmc.rs` models:
    - Basic IRQ/loop flags, sample address/length, bit shifting, and a timer that now matches `DMC_RATE_TABLE[index]` in period.
    - Sample fetch as a simple “if buffer empty & bytes remaining > 0, read byte now” with no CPU DMA timing.
  - Missing compared to Mesen2’s `DeltaModulationChannel`:
    - `_needToRun` / `NeedToRun()` scheduling.
    - `_transferStartDelay` and `_disableDelay` based on CPU cycle odd/even.
    - DMA end behaviour when `sampleLength == 1` (sample duplication glitch / abort-on-next-bit).

- **Target design**
  - Extend `Dmc` with:
    - `need_to_run: bool`, `transfer_start_delay: u8`, `disable_delay: u8`.
  - Implement methods analogous to Mesen2:
    - `fn set_enabled(&mut self, enabled: bool, status: &mut StatusFlags, cpu_cycle: u64)`:
      - When disabling, set `disable_delay` to 2 or 3 based on `cpu_cycle & 1` and mark `need_to_run`.
      - When enabling with `bytes_remaining == 0`, call `restart_sample()` and set `transfer_start_delay` similarly, `need_to_run = true`.
    - `fn process_clock<F>(&mut self, mut reader: F, status: &mut StatusFlags, cpu_cycle: u64)`:
      - Decrement `disable_delay`; when it hits 0, cancel any pending DMA and clear `bytes_remaining`.
      - Decrement `transfer_start_delay`; when 0 and buffer empty & bytes_remaining > 0, trigger a “logical” DMA read via the provided `reader` (no CPU stall yet).
      - Update `need_to_run` based on `disable_delay`, `transfer_start_delay`, and `bytes_remaining`.
    - `fn need_to_run(&mut self, cpu_cycle: u64) -> bool`:
      - Mirror `DeltaModulationChannel::NeedToRun` by calling `process_clock` when `need_to_run` is true and returning the new flag state.
    - Implement the sample-length==1 glitch paths:
      - When DMA ends exactly when `_bitsRemaining == 8` and `_timer.GetTimer() == _timer.GetPeriod()`, optionally duplicate the last sample byte as Mesen2 does (behind a config flag in nesium).
      - When DMA ends on the cycle before bit counter resets (`_bitsRemaining == 1` and `_timer.GetTimer() < 2`), abort a single-byte DMA but still cause the halted CPU cycle (we can approximate this without a full stall model).

- **Integration steps**
  1. Thread `cpu_cycle` into DMC calls from `Apu` (we already track an internal cycle counter).
  2. Update `Apu::clock_with_reader` / future `run_until` to:
     - Call `dmc.need_to_run(cpu_cycle)` and, if true, run `dmc.process_clock(reader, &mut status, cpu_cycle)` on each relevant cycle.
  3. Keep CPU stall modelling as a future enhancement; for now, limit ourselves to “logical” DMA timing and glitch behaviour.

### 0.4 – Align APU<->Mixer wiring with Mesen2’s AddDelta/EndFrame

- **Problem today**
  - nesium’s `NesSoundMixer::add_delta` immediately mixes all channels via `mix_channels_stereo` and sends left/right deltas to blip_buf.
  - Mesen2 stores per-channel deltas in `_channelOutput[channel][time]`, then in `EndFrame`:
    - Sorts timestamps.
    - Incrementally updates `_currentOutput[channel]` over time.
    - Calls `GetOutputVolume()` at each timestamp and feeds that into `blip_add_delta`.

- **Target design**
  - Keep the existing `mix_channels_stereo` and scaling (they already match `GetOutputVolume()*4/32768`), but move toward an “APU ≥ NesSoundMixer” contract like Mesen2:
    - Each channel’s `run(target_cycle)` calls `mixer.add_delta(AudioChannel::X, tick_cycle, delta)` at the time its output changes.
    - `add_delta` appends per-channel deltas to a timestamped buffer, not immediately mixing stereo.
    - `end_frame(frame_end_clock, out)`:
      - Sorts timestamps.
      - Accumulates per-channel outputs.
      - Calls `mix_channels_stereo` at each timestamp to generate stereo deltas for blip_buf.
  - This is a larger refactor but brings the model almost 1:1 with Mesen2’s `NesSoundMixer`.

- **Incremental steps**
  1. Introduce an internal “per-channel delta buffer” type in `NesSoundMixer` behind a feature flag, while keeping the current fast path.
  2. During transition:
     - Use the buffered mode only in debug/profile builds when generating `apu_debug.raw`, so we can validate waveform parity without impacting runtime too much.
  3. Once parity is acceptable on key test ROMs and problematic games (e.g., 赤影战士鼓点段不再爆音), consider making the buffered mode the default.
     - Once parity is acceptable on key test ROMs and problematic games (e.g., Shadow of the Ninja drum-heavy cutscene segments no longer pop), consider making the buffered mode the default.

## Phase 1 – Core Mixer Parity (most important: clean, stable audio)

1. ~~**Match NesSoundMixer channel math and scaling**~~
   - Verify `GetChannelOutput` + `GetOutputVolume` vs `audio::mixer::NesSoundMixer::mix_channels_stereo`:
     - Square/TND/expansion coefficients identical (95.88, 159.79, 8128, 22638, FDS/MMC5/N163/S5B/VRC6/VRC7 weights).
   - Rework our `mixed / 8192.0 + BLIP_SCALE + soft_clip` path to match Mesen2’s effective range:
     - Derive a mapping from `GetOutputVolume()*4` (int16) to our float [-1,1] without relying heavily on tanh.
     - Keep enough headroom so strong drum hits (for example, intro cutscenes with heavy percussion) do not hit hard clipping.

2. ~~**Align blip_buf usage and frame timing**~~
   - Ensure `NesSoundMixer::add_delta` and `end_frame` use clock-relative timestamps like `NesSoundMixer::AddDelta/EndFrame`:
     - Non-decreasing per-frame clock, correct frame duration passed to `blip_end_frame`.
   - Confirm we never silently drop or truncate samples under normal NTSC/PAL rates.

3. ~~**Implement Mesen2-style UpdateRates and panning behaviour**~~
   - Add a dedicated `update_rates(clock_rate, sample_rate)` path mirroring `NesSoundMixer::UpdateRates`:
     - Update blip rates when region/clock/sample rate changes.
     - Track `_hasPanning` and clear left/right blip buffers when panning switches between “all center” and “per-channel”.
   - Mirror exact `ChannelPanning` mapping:
     - `(ChannelPanning[i] + 100) / 100.0` → `[0,2]` internal representation.

4. ~~**Expansion audio wiring audit**~~
   - ~~Cross-check `ExpansionSamples` → `AudioChannel::{Fds,Mmc5,Namco163,Sunsoft5B,Vrc6,Vrc7}` mapping against Mesen2’s channel indices.~~
   - ~~Verify per-chip gains (20/43/20/15/5/1) are applied identically in both left/right paths.~~

5. ~~**Stereo post-filter parity (Delay/Panning/Comb)**~~
   - ~~Compare Rust `StereoDelayState`, `StereoPanningState`, `StereoCombState` with their Mesen2 counterparts:~~
     - ~~Units (ms vs samples), strength ranges, angle mapping.~~
   - ~~Ensure default filter config (i.e., “StereoFilter: None” and its params) matches Mesen2 defaults.~~
   - Known behavioural differences (intended for now):
     - Comb strength uses a `[0.0, 1.0]` float ratio (`MixerSettings::stereo_comb_strength`) instead of Mesen2's integer `0..100` scale (`StereoCombFilterStrength`); frontends must divide the Mesen2-style value by 100 when mapping configs.
     - Panning filter in nesium leaves stereo intact when `angle_deg == 0` (no-op), while Mesen2 still runs the filter and effectively collapses to mono at 0°.
     - Delay/comb filters in nesium early-return when `delay_ms <= 0`, whereas Mesen2 still runs the filter with a zero delay; in practice UIs avoid "enabled filter with zero delay", so this is unlikely to matter.

6. ~~**Shared audio output wrapper parity (nesium-audio)**~~
   - ~~Confirm `nesium-audio::NesAudioPlayer` matches Mesen2’s device-facing expectations:~~
     - ~~Interleaved stereo input (L,R), downmix behaviour for mono devices, extra channels mirroring.~~
   - ~~Use the same implementation for `nesium-egui` and `nesium-flutter` to eliminate frontend differences.~~
   - Notes:
     - nesium currently uses `cpal`’s default output config and a small (~0.2s) float FIFO in `NesAudioPlayer`; Mesen2 routes `int16` samples through a configurable `IAudioDevice` with latency control in `AudioConfig` and a separate `SoundMixer` bus (tracked under Phase 2 resampling/master-volume tasks).
     - Both `nesium-egui` and `nesium-flutter` now construct `Nes` with the runtime audio device’s sample rate and feed the same interleaved stereo stream into `NesAudioPlayer`, so frontend‑specific audio differences are eliminated at this layer.

## Phase 2 – Global SoundMixer Bus (after core behaviour matches, unify bus/config)

7. **Introduce a SoundMixer-equivalent bus layer**
   - Add a Rust module/crate mirroring `Core/Shared/Audio/SoundMixer`:
     - Accepts PCM from one or more `NesSoundMixer` instances.
     - Owns global resampler, EQ, reverb, crossfeed, and master volume.

8. ~~**Match resampling path (96 kHz → user sample rate)**~~
   - ~~Mirror Mesen2’s fixed `_sampleRate = 96000` in `NesSoundMixer` and resample to user-configurable rate:~~
     - ~~Implement a `SoundResampler` equivalent (can start with a simple quality setting).~~
     - **Remaining difference:** nesium currently uses a per-frame linear resampler in `SoundMixerBus` without dynamic rate adjustment hooks for fast-forward / slow motion; Mesen2’s `SoundResampler` and `HermiteResampler` support higher-quality resampling and pitch-adjust for variable emulation speed.

9. ~~**Master volume and background/fast-forward attenuation**~~
   - ~~Introduce `AudioConfig`-style master volume handling:~~
     - ~~MasterVolume, MuteSoundInBackground, ReduceSoundInBackground, ReduceSoundInFastForward, VolumeReduction.~~
   - ~~Apply these scalings in the bus right before sending to the audio device.~~
   - Notes:
     - nesium’s `AudioBusConfig` uses `master_volume` and `volume_reduction` as `[0.0, 1.0]` floats, whereas Mesen2’s `AudioConfig` uses `0..100` integer percentages; frontends must map UI values accordingly (e.g. 75% reduction → `volume_reduction = 0.75`).
     - Background / fast-forward attenuation in nesium is driven by `AudioBusConfig::{in_background,is_fast_forward}` flags, which frontends can toggle; Mesen2 derives them from `EmulationFlags`.
     - Scaling is applied in float space after mixing/resampling on the global bus, while Mesen2 applies it on the `int16` buffer; behaviour is equivalent for normal ranges but not bit‑exact.

10. ~~**Basic EQ / reverb / crossfeed integration**~~
    - ~~Implement minimal versions of `Equalizer`, `ReverbFilter`, `CrossFeedFilter`:~~
      - ~~Initially wired with default/neutral settings, then tuned to Mesen2’s defaults.~~
    - ~~Add hooks so they can be configured from frontends later, even if UI is not exposed yet.~~
    - Notes / differences:
      - The bus-level `Equalizer` currently approximates the 20-band EQ with a single global gain derived from the average band gain (no per-band frequency shaping yet), whereas Mesen2 uses `orfanidis_eq` to realise true multi-band filters.
      - The `ReverbFilter` implementation in nesium uses a single stereo feedback delay line controlled by `reverb_strength` and `reverb_delay_ms`; Mesen2 uses a bank of 5 delays per channel with tuned taps/decays.
      - `CrossFeedFilter` in nesium expects a `[0.0, 1.0]` float ratio (applied directly) instead of Mesen2’s integer `0..100` percent; frontends must map UI values accordingly.

## Phase 3 – Advanced Features and Edge Cases

11. **VS DualSystem audio routing**
    - Mirror `NesSoundMixer::ProcessVsDualSystemAudio`:
      - Support Main/Sub/Both output modes.
      - Implement buffering and mixing semantics for dual-console setups.

12. **Save-state integration for audio state**
    - Include `NesSoundMixer` and global SoundMixer state in save/load:
      - blip_buf internal state, filter histories, resampler state, per-channel levels.
    - Aim to avoid pops/phase jumps when loading states mid-frame or mid-note.

13. **Audio configuration and frontend UI mapping**
    - Define a shared `AudioConfig` struct that mirrors Mesen2’s audio settings:
      - SampleRate, StereoFilter, per-channel volume/pan, EQ band gains, reverb/crossfeed options, VS DualSystem settings.
    - Expose a minimal subset in `nesium-egui` and Flutter UIs for manual tuning.

14. **Verification against Mesen2 recordings**
    - Build a small set of reference WAVs from Mesen2 (e.g., intro cutscenes with strong drums, typical BGMs, noisy/percussion-heavy passages) and record corresponding nesium output:
      - Compare peaks/RMS and visually inspect waveforms to ensure no extra clipping or obvious spectral differences under default settings.
    - Re-run under different sample rates and stereo filter configs to catch regressions.

---

# TODO: PPU Open Bus (Mesen2-aligned)

Follow-up tasks to finish parity with Mesen2's open-bus behaviour on the PPU side.

1. ~~**Add a PPU-specific open-bus latch with decay**~~
   - ~~Mirror `bus::open_bus::OpenBus` semantics (per-bit decay ~1 second of CPU time, last-driven value).~~
   - ~~Decide whether to reuse `OpenBus` generically or add a PPU-local copy to avoid accidental cross-clock coupling with the CPU bus.~~
   - ~~Provide `step()`, `sample()`, and `latch()` entry points so reads from floating sources get decayed values and writes refresh decay deadlines.~~

2. **Clock the PPU open bus on every relevant access**
   - ~~CPU-visible register mirror (`cpu_read`/`cpu_write`): step before accesses; latch on writes; latch returned values for reads.~~ (nesium now latches on writes and on the registers that actively drive the bus: `$2002`, `$2004`, `$2007`.)
   - ~~Internal VRAM/palette/OAM accesses that leave data on the PPU data bus (e.g., `read_vram_data`, palette reads, OAM reads outside rendering) should latch their results.~~ (internal VRAM/OAM fetches update the internal OAM bus and palette reads go through `apply_masked`, matching Mesen2’s CPU-visible behaviour.)
   - ~~For reads that are “open” on hardware (e.g., OAMDATA during rendering, write-only registers), return the decayed sample instead of a fixed constant (unless hardware dictates a specific value).~~ (default cases use the decayed PPU open bus; `$2004` during rendering now exposes the internal OAM bus instead of a fixed constant.)

3. **Hook decay tick into PPU timing**
   - ~~Increment the PPU bus tick once per CPU-visible PPU access and for internal PPU fetches that would keep the data bus charged.~~ (nesium now advances the PPU open-bus tick once per completed frame, with decay after ~3 frames, matching Mesen2’s frame-based decay model.)
   - ~~Keep tick units aligned with CPU bus timing expectations (Mesen2 uses CPU-cycle granularity for the decay window; match that to stay compatible with test ROMs).~~

4. **Reset/initialisation**
   - ~~Clear/latch to 0 on `Ppu::new()`/`reset()`.~~
   - Ensure state is saved/restored appropriately if snapshots are ever added for PPU.

5. **Compatibility checks**
   - Re-run `cpu_exec_space/test_cpu_exec_space_ppuio.nes` and `cpu_dummy_writes_ppumem.nes` to confirm open-bus expectations.
   - Watch for regressions in PPU reads that expect status bit overlays (e.g., `$2002` should still merge VBlank/flags with bus bits).

6. **Documentation**
   - Update `README.md` accuracy notes to mention PPU-side open bus and decay once implemented.
   - Brief code comment near the new latch describing the Mesen2 alignment and decay window choice.

7. **PPU model-specific status open-bus patterns**
   - Mesen2 adjusts the low bits of `$2002` depending on the PPU model (2C05A-E) via `ProcessStatusRegOpenBus`; nesium currently always assumes the standard 2C02 mask/behaviour.
   - TODO: If Vs. System / 2C05 support is added, introduce a PPU model enum and feed it into `read_status` so `$2002` open-bus patterns can match the specific chip variant.

# TODO: PPU vs Mesen2 parity

High-level behaviour differences between `nesium-core`'s NES PPU and Mesen2's `NesPpu` to revisit.

1. **$2007 read/write timing and VRAM increment**
   - Mesen2 delays the VRAM address increment by 1 PPU cycle after `$2007` reads/writes (`_needVideoRamIncrement` + `UpdateVideoRamAddr()`), and ignores a second `$2007` read when it happens on consecutive CPU cycles (`_ignoreVramRead`).
   - ~~Nesium currently increments `v` immediately in `write_vram_data`/`read_vram_data` and has no special handling for back-to-back `$2007` reads.~~ (nesium now uses a small ignore window for consecutive `$2007` reads and delays the VRAM increment by one PPU dot after each read/write.)
   - ~~TODO: Consider adding a small pending increment latch (similar to `_needVideoRamIncrement`) to better match Mesen2 / hardware tests (e.g. `full_palette`).~~

2. **PPUSCROLL/PPUADDR scroll glitches ($2000/$2005/$2006)**
   - Mesen2 implements the well-known scroll glitches when writing to `$2000/$2005/$2006` on specific cycles (e.g. 257 and at 8-dot boundaries) via `ProcessTmpAddrScrollGlitch` and the `EnablePpu2006ScrollGlitch` setting, corrupting `v`/`t` in the same way as hardware.
   - ~~Nesium currently performs clean copies between `t` and `v` (`copy_horizontal_scroll`/`copy_vertical_scroll`) with no glitch emulation; TODO comments in `ppu.rs` acknowledge this.~~ (nesium now emulates the basic dot-257 glitch for `$2000/$2005/$2006` and the `$2006` AND-style corruption when the delayed update lands on Y/X increments, matching Mesen2’s `UpdateState` logic.)
   - TODO: Keep an eye on region-specific behaviour (PAL/Dendy) and any remaining 8-dot-boundary edge cases that may show up in test ROMs (e.g. VisualNES-based scroll glitch tests) and refine the implementation if discrepancies are observed.

3. **OAMDATA ($2004) read/write behaviour**
   - ~~Writes: Mesen2 ignores writes to `$2004` during rendering and instead performs the “high 6 bits only” increment (`_spriteRamAddr = (_spriteRamAddr + 4) & 0xFF`), which models the hardware OAMADDR glitch.~~
   - ~~Reads: During rendering, Mesen2 returns the value currently on the internal OAM bus (`_oamCopybuffer` or secondary OAM contents) rather than primary OAM; nesium approximates this as a fixed `0xFF` placeholder.~~ (nesium now tracks an internal OAM bus copybuffer and maps `$2004` reads in the 257..=320 sprite-fetch window to the corresponding secondary OAM address.)
   - TODO: Keep iterating on corner cases (sprite overflow / PAL refresh behaviour) if specific test ROMs demonstrate remaining differences in `$2004` bus contents.

4. **Sprite evaluation and overflow bug**
   - Mesen2 has a very detailed implementation of the sprite overflow bug during the overflow scan phase, including the exact `n`/`m` increment pattern and the “realign” behaviour after overflow (`_overflowBugCounter`, `_oamCopyDone`, etc.).
   - ~~Nesium implements a simplified `SpriteEvalPhase::OverflowScan` that captures the general effect (overflow flag set when more than 8 in-range sprites) and uses an `overflow_bug_counter` but does not yet model all of Mesen2’s address realignment and `_oamCopyDone`/secondary-OAM interaction edge cases.~~ (nesium now mirrors Mesen2’s overflow scan `n`/`m` pattern, wrap-to-start behaviour, and realign via `overflow_bug_counter`/`oam_copy_done`, and derives per-scanline sprite count from the number of bytes copied into secondary OAM, like Mesen2’s `(_secondaryOamAddr + 3) >> 2`.)
   - TODO: If tests start depending on OAMADDR-based misalignment and the 2C02B X=255 phantom-sprite quirk, extend the sprite eval state to model the full `_spriteRamAddr`/`_spriteAddrL` interaction and add an optional 2C02B mode flag to match Mesen2’s `EnablePpuSpriteEvalBug`.

5. **PPUMASK grayscale and color emphasis bits**
   - In Mesen2, `$2001` grayscale and R/G/B emphasis bits are applied when reading palette RAM and when writing the final framebuffer (`UpdateGrayscaleAndIntensifyBits`, `_paletteRamMask`, `_intensifyColorBits`).
   - ~~Nesium defines the corresponding `Mask` bits but does not yet apply them to palette reads or to the framebuffer;~~ ~~palette writes also don’t clamp values to 0x3F like Mesen2.~~ (nesium now clamps palette writes to 0x3F and applies a grayscale mask to palette reads and framebuffer writes when `$2001` bit 0 is set.)
   - TODO: Decide whether to apply color emphasis bits (R/G/B) in the core PPU (palette path) or in the front-end video pipeline, and add masking/intensify logic to bring behaviour in line with Mesen2 where it matters for test ROMs.

6. **Region-specific timing (NTSC vs PAL/Dendy)**
   - Mesen2 supports NTSC/PAL/Dendy via `UpdateTimings` and changes `_nmiScanline`, `_vblankEnd`, master clock divider and PAL-specific OAM refresh behaviour; nesium currently targets NTSC only (`SCANLINES_PER_FRAME = 262`, fixed odd-frame skip logic).
   - TODO: If multi-region support is desired, introduce a small region enum + timing table for scanline counts, NMI/vblank start/end, and PAL-specific sprite eval behaviour, guided by Mesen2’s `UpdateTimings`.

7. **$2007 palette reads / open bus interaction**
   - Mesen2 merges palette RAM reads with open-bus high bits and applies grayscale masking; ~~nesium’s `read_vram_data` currently reads directly from `PaletteRam` without mixing in bus bits or mask.~~ (nesium now merges palette reads with the PPU-side open-bus latch, preserving high bits.)
   - TODO: Add an optional grayscale mask (like Mesen2’s `_paletteRamMask`) on palette reads when `$2001` grayscale is enabled so that palette open-bus behaviour also matches test ROM expectations.

# TODO: CPU Open Bus (Mesen2-aligned)

Follow-up tasks to finish parity with Mesen2's CPU-side open-bus behaviour.

1. ~~Share a single CPU open-bus latch across all CPU memory reads/writes~~
   - ~~Use `OpenBus::new()` (no decay) for the CPU data bus and step it once per bus access in `CpuBus::read`/`write`.~~

2. ~~Special-case APU status reads at `$4015`~~
   - ~~Reading `$4015` should not update the external open-bus latch (only the internal CPU data bus).~~
   - ~~Mix bit 5 from the current open bus into the value returned by `$4015` to approximate Mesen2's `GetInternalOpenBus() & 0x20` behaviour.~~

3. **Expose CPU open bus to mappers/controllers (future)**
   - Add a small helper on `NES` to return the current CPU open-bus value with an optional mask (equivalent to Mesen2's `GetOpenBus(mask)`).
   - Once available, audit mapper/controller code paths that currently assume constant defaults and replace them with masked CPU open-bus reads where appropriate.

4. **Model internal vs external CPU data bus (future refinement)**
   - Mesen2 keeps separate internal/external latches in `OpenBusHandler`; nesium currently approximates `GetInternalOpenBus()` by reusing the external latch.
   - If tests start depending on the distinction, add a secondary “internal bus” byte and `set_internal_only()`/`internal_sample()` helpers, then use those in the APU/CPU where Mesen2 does.

5. **Per-register CPU open-bus masks (future refinement)**
   - Some CPU-visible registers only drive a subset of bits and leave the remaining bits floating on the bus (handled via `ApplyOpenBus(mask, value)` in Mesen2).
- TODO: Audit CPU register reads that should partially drive the bus and switch them from raw `latch()`/`sample()` usage to `apply_masked(mask, value)` once concrete cases are identified.

## Mapper abstraction review (moved from `nes_mapper_review_and_todo.md`)

### 1. Goals

You want an accurate NES emulator in Rust, with a `Mapper` abstraction that:

- Comfortably covers most **mainstream mappers**: NROM, UxROM, CNROM, AxROM, MMC1, MMC3, common UNROM variants, etc.
- Can be extended to support more complex boards (VRC family, MMC5, various pirate / homebrew mappers).
- Stays reasonably simple to implement and debug, while still being precise enough to pass demanding NES test ROMs.

---

### 2. Current `Mapper` Trait

```rust
pub trait Mapper: Debug + Send + DynClone + Any + 'static {
    fn cpu_read(&self, addr: u16) -> Option<u8>;
    fn cpu_write(&mut self, addr: u16, data: u8, cpu_cycle: u64);

    fn ppu_read(&self, addr: u16) -> u8;
    fn ppu_write(&mut self, addr: u16, data: u8);

    fn irq_pending(&self) -> bool { false }
    fn clear_irq(&mut self) {}

    fn prg_rom(&self) -> Option<&[u8]> { None }
    fn prg_ram(&self) -> Option<&[u8]> { None }
    fn prg_ram_mut(&mut self) -> Option<&mut [u8]> { None }
    fn chr_rom(&self) -> Option<&[u8]> { None }
    fn chr_ram(&self) -> Option<&[u8]> { None }
    fn chr_ram_mut(&mut self) -> Option<&mut [u8]> { None }

    fn mirroring(&self) -> Mirroring;

    fn mapper_id(&self) -> u16;
    fn name(&self) -> Cow<'static, str> {
        Cow::Owned(format!("Mapper {}", self.mapper_id()))
    }
}
```

---

### 3. What the Current Design Already Handles Well

#### 3.1 CPU Bus Side

- `cpu_read(&self, addr) -> Option<u8>` cleanly expresses:
  - `Some(byte)` → mapper drives the data bus.
  - `None` → open bus (CPU should reuse the last bus value).
- `cpu_write(&mut self, addr, data, cpu_cycle)` gives mappers the current CPU cycle for timing quirks.
  - You already use this for MMC1's "ignore consecutive serial writes" behavior.

This is enough for:

- Basic ROM/RAM banking (NROM, UxROM, CNROM, AxROM, etc.).
- MMC1 behavior including write timing quirks.
- Timers / IRQs that care about when writes happen.

#### 3.2 PPU CHR Side

- `ppu_read/ppu_write` allow mapper-controlled CHR for `$0000-$1FFF`:
  - Suitable for CHR ROM/RAM banking (1K/2K/4K/8K windows).
  - Works for MMC1, MMC3, MMC2/4 latches, and many simple mappers.

#### 3.3 IRQ Support

- `irq_pending` + `clear_irq` is enough to:
  - Implement scanline / counter IRQs in MMC3, FME-7, VRC variants, etc.
  - Have the CPU core poll and assert the IRQ line correctly.

#### 3.4 Introspection and Save/Debug

- `prg_rom/prg_ram/chr_rom/chr_ram` hooks let you:
  - Implement UI-level memory viewers.
  - Implement basic battery save by dumping PRG/CHR RAM.

#### 3.5 Mirroring / Nametable Wiring (Basic Case)

- `mirroring()` exposes:
  - Horizontal / vertical / single-screen / etc.
  - Compatible with the usual 2 KB CIRAM setup in the PPU.

This is sufficient for the majority of licensed games using simple mirroring schemes.

**Conclusion**: The current trait is already good enough to implement **most "normal" NES cartridges**, especially those that only touch PRG/CHR banking + simple nametable mirroring.

---

### 4. Key Limitations / Missing Capabilities

These are the main areas where the current interface will start to struggle if you aim for very high accuracy or more exotic boards.

#### 4.1 Nametable / VRAM Mapping Is Too Limited

Right now:

- The mapper only controls CHR (`$0000-$1FFF`) via `ppu_read/ppu_write`.
- Nametable area (`$2000-$3EFF`) is assumed to be:
  - Handled by the PPU itself, plus
  - A global `Mirroring` enum.

This cannot express:

- **MMC5 ExRAM used as nametable** or extended attribute tables.
- Boards with **extra nametable RAM or ROM** on the cartridge.
- 4-screen VRAM and other non-standard nametable wiring beyond simple mirroring.

For those, the mapper must be able to:

- Decide where each nametable byte actually comes from (CIRAM vs mapper RAM vs ROM).
- Potentially alter values returned during attribute or pattern fetches.

#### 4.2 No General PPU/CPU Timing Hooks

You have timing info on CPU writes, but not:

- A hook that runs **every CPU cycle** for mappers with cycle-based IRQ timers.
- A general hook for **every VRAM access** for PPU-side timing (e.g., MMC3 scanline IRQ based on A12 edges).

This makes high-accuracy implementations of:

- MMC3 IRQ (PPU A12 rising edges + CPU M2 gating).
- Some VRC scanline timers.
- MMC5 advanced graphics modes (vertical split / extended attributes).

much harder or forces hacks in the CPU/PPU core instead of keeping logic localized in the mapper.

#### 4.3 No Way to Express PPU Open-Bus

CPU side:

- `Option<u8>` makes it clear when the bus is floating.

PPU side:

- `fn ppu_read(&self, addr: u16) -> u8` always returns *some* value, even when on real hardware:
  - The read would actually be open-bus.
  - The result might depend on previous VRAM / palette / nametable accesses.

This only matters for very strict accuracy or specific test ROMs, but it's a mismatch in API expressiveness.

#### 4.4 RAM Types Are Not Distinguished

Many real boards have more than one RAM category:

- Battery-backed save RAM.
- Non-battery work RAM.
- Mapper-private RAM (sometimes not CPU-visible).
- Possibly battery-backed CHR RAM.

The current:

```rust
fn prg_ram(&self) -> Option<&[u8]>;
fn prg_ram_mut(&mut self) -> Option<&mut [u8]>;
```

combines all CPU-side RAM into one bucket. This is fine for running games, but:

- Complicates battery save (what should be persisted?).
- Limits tooling (debugger cannot easily display "this is PRG RAM vs mapper RAM").

#### 4.5 No Built-In Path for Expansion Audio

Expansion audio (VRC6/VRC7/Namco 163/Sunsoft 5B/MMC5/FDS, etc.) requires:

- Mapper responding to CPU writes to audio registers.
- A per-cycle clock to update audio synth state.
- A way for the APU/mixer to get the expansion audio sample.

The current trait doesn’t mention audio at all, which is okay, but:

- You will need a separate trait or mechanism to plug expansion audio into the APU.

---

### 5. Suggested Trait Extensions (Design Ideas)

All of these can be added as **optional methods with default no-op implementations** so that existing mappers keep compiling.

#### 5.1 VRAM Access Hook for PPU (MMC3/MMC5, etc.)

Add something like:

```rust
pub trait Mapper {
    // existing methods...

    /// Called on every PPU VRAM access (0x0000-0x3FFF).
    /// cpu_cycle is the current CPU cycle for M2-based gating logic.
    fn ppu_vram_access(&mut self, _addr: u16, _is_read: bool, _cpu_cycle: u64) {}
}
```

PPU integration (pseudo-code):

```rust
fn vram_read(&mut self, addr: u16, is_render: bool) -> u8 {
    let cpu_cycle = self.cpu_cycle;
    self.mapper.ppu_vram_access(addr, true, cpu_cycle);

    if addr < 0x2000 {
        self.mapper.ppu_read(addr)
    } else {
        // nametable / palette handling
    }
}
```

Use cases:

- **MMC3**: detect A12 rising edges in `ppu_vram_access` and update the IRQ counter with correct CPU/PPU timing.
- **MMC5**: track VRAM fetch patterns to support vertical split, extended attributes, etc.

#### 5.2 Nametable Mapping API (CIRAM vs Mapper VRAM/ROM)

Introduce a `NametableTarget` enum:

```rust
pub enum NametableTarget {
    /// Use PPU CIRAM (internal 2 KB VRAM). `u16` is CIRAM offset.
    Ciram(u16),
    /// Use mapper-controlled VRAM/ROM. `u16` is mapper-local offset.
    MapperVram(u16),
    /// No device drives the bus (open bus).
    None,
}
```

Extend the trait:

```rust
pub trait Mapper {
    // existing methods...

    /// Map PPU $2000-$2FFF address to an underlying nametable source.
    /// Default implementation uses `mirroring()` and standard CIRAM mapping.
    fn map_nametable(&self, addr: u16) -> NametableTarget {
        let base = addr & 0x0FFF;
        let offset = match self.mirroring() {
            Mirroring::Horizontal => {
                let nt = (base >> 10) & 3;
                let within = base & 0x03FF;
                match nt {
                    0 | 1 => within,
                    _ => 0x0400 | within,
                }
            }
            Mirroring::Vertical => {
                let nt = (base >> 10) & 3;
                let within = base & 0x03FF;
                match nt {
                    0 | 2 => within,
                    _ => 0x0400 | within,
                }
            }
            Mirroring::SingleScreenLower => base & 0x03FF,
            Mirroring::SingleScreenUpper => 0x0400 | (base & 0x03FF),
            // Mirroring::FourScreen / MapperControlled could be added later.
        };
        NametableTarget::Ciram(offset)
    }

    /// Called when `map_nametable` returns `MapperVram`.
    fn mapper_nametable_read(&self, _offset: u16) -> u8 { 0 }

    fn mapper_nametable_write(&mut self, _offset: u16, _value: u8) {}
}
```

PPU nametable path:

```rust
fn nametable_read(&mut self, addr: u16) -> u8 {
    match self.mapper.map_nametable(addr) {
        NametableTarget::Ciram(off) => self.ciram[off as usize],
        NametableTarget::MapperVram(off) => self.mapper.mapper_nametable_read(off),
        NametableTarget::None => self.open_bus_value,
    }
}
```

This allows:

- Ordinary mappers to continue using `mirroring()` + CIRAM.
- MMC5 / 4-screen / ExRAM boards to override `map_nametable` and fully control nametable behavior.

#### 5.3 Optional Per-CPU-Cycle Hook

If you need true per-cycle timers inside the mapper, add:

```rust
pub trait Mapper {
    // existing methods...

    /// Called once per CPU cycle.
    fn cpu_clock(&mut self, _cpu_cycle: u64) {}
}
```

Then in your CPU loop:

```rust
for _ in 0..cycles_for_this_instruction {
    cpu_cycle += 1;
    mapper.cpu_clock(cpu_cycle);
    // tick APU, PPU, etc.
}
```

Not every mapper will need this, but it’s useful for:

- Certain VRC IRQ modes.
- Mappers with hardware timers.
- Potentially expansion audio (if you choose to couple it here).

#### 5.4 Optional PPU Open-Bus Expressiveness

If you decide to model PPU open-bus more accurately later, you can:

- Either change `ppu_read` to return `Option<u8>`.
- Or introduce a small enum (e.g., `PpuReadResult { Data(u8), OpenBus }`).

For now, this can be postponed if test ROMs don’t require that level of detail.

#### 5.5 More Granular RAM Introspection (Save vs Work vs Mapper)

Longer-term refinement:

```rust
pub trait Mapper {
    // PRG save RAM (battery-backed)
    fn prg_save_ram(&self) -> Option<&[u8]> { None }
    fn prg_save_ram_mut(&mut self) -> Option<&mut [u8]> { None }

    // PRG work RAM (not battery-backed)
    fn prg_work_ram(&self) -> Option<&[u8]> { None }
    fn prg_work_ram_mut(&mut self) -> Option<&mut [u8]> { None }

    // Mapper-private RAM (could be used for ExRAM, etc.)
    fn mapper_ram(&self) -> Option<&[u8]> { None }
    fn mapper_ram_mut(&mut self) -> Option<&mut [u8]> { None }

    // Optional: CHR battery-backed RAM
    fn chr_battery_ram(&self) -> Option<&[u8]> { None }
    fn chr_battery_ram_mut(&mut self) -> Option<&mut [u8]> { None }
}
```

This lets:

- The save system persist only the right subsets.
- Debug tools label different RAM regions more accurately.

You can keep your existing `prg_ram/chr_ram` methods as compatibility layers or slowly migrate to the more granular version.

#### 5.6 Separate Expansion Audio Trait

To keep `Mapper` focused on memory/IRQ, expansion audio can be handled via a separate trait:

```rust
pub trait ExpansionAudio {
    /// Advance the expansion audio state by one CPU cycle.
    fn clock_audio(&mut self);

    /// Produce the current expansion audio sample (e.g., linear 0.0..1.0).
    fn sample(&mut self) -> f32;
}
```

- Some mapper types implement both `Mapper + ExpansionAudio`.
- The APU/mixer holds a handle to `dyn ExpansionAudio` where applicable and:
  - Calls `clock_audio()` per CPU cycle.
  - Calls `sample()` when mixing audio.

---

### 6. Notes on the Current `Mapper1` Implementation

Your `Mapper1` implementation:

- Uses `cpu_cycle` in `cpu_write` to correctly emulate MMC1’s serial write timing quirks.
- Respects mirroring by exposing it via `mirroring()`.
- Nicely exposes PRG/CHR ROM/RAM through the introspection hooks.
- Handles PRG-RAM enable/disable and returns `None` for reads when disabled, which fits nicely with the bus-level open-bus model.

Potential future tweaks:

- Once you add VRAM hooks / nametable mapping:
  - MMC1 will mostly ignore them; only more advanced SxROM variants might use extended behaviors.
- If you extend RAM introspection (save/work/mapper RAM), you can simply map:
  - `prg_save_ram` → your existing `prg_ram` for plain MMC1 boards.
  - Leave `mapper_ram` and `chr_battery_ram` as `None`.

---

### 7. TODO Checklist

#### 7.1 Core Trait & Architecture

- [x] Add `fn ppu_vram_access(&mut self, addr: u16, is_read: bool, cpu_cycle: u64)` to the `Mapper` trait with a default no-op implementation.
- [x] Update the PPU so that **every VRAM access** (pattern fetch + CPU `$2007` accesses) calls `mapper.ppu_vram_access`.
- [x] Design and implement the `NametableTarget` enum and the trait methods `map_nametable`, `mapper_nametable_read`, and `mapper_nametable_write`.
- [x] Change the PPU nametable read/write path to use `map_nametable`, selecting between CIRAM, mapper VRAM, or open-bus.
- [x] (Optional) Extend the `Mirroring` enum if needed (e.g., `FourScreen`, `MapperControlled`) and adapt the default `map_nametable` implementation accordingly.
- [x] Decide whether to upgrade `ppu_read` to an `Option<u8>` or similar to explicitly represent PPU open-bus conditions.
- [x] Evaluate whether you need a per-CPU-cycle hook `fn cpu_clock(&mut self, cpu_cycle: u64)` and, if so, integrate it into the CPU core loop.
- [x] Design a separate `ExpansionAudio` trait and integrate it into the audio pipeline (APU/mixer).
- [x] (Optional) Refine RAM introspection by splitting `prg_ram`/`chr_ram` into save/work/mapper RAM types, and adjust battery save & debugger code.

#### 7.2 Specific Mapper Implementations (Future Work)

- [ ] Use `ppu_vram_access` + `cpu_cycle` to implement **MMC3 IRQ** with proper A12 edge detection and M2-based timing.
- [ ] Use `map_nametable` + `mapper_nametable_*` to implement **MMC5 ExRAM** as nametable / extended attributes / fill mode.
- [ ] For mappers with bus conflicts (CNROM, certain UNROM boards), implement `data &= prg_rom_byte` behavior in `cpu_write`.
- [ ] Extend MMC1/SxROM implementations with CHR/PRG high-bit banking and board-specific quirks using NES 2.0 submapper & board DB info.
- [ ] Implement `ExpansionAudio` for mappers that provide extra audio (VRC6/VRC7/Sunsoft 5B/MMC5/FDS) and connect them to the audio mixer.

---

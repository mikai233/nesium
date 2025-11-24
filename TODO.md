# TODO: PPU Open Bus (Mesen2-aligned)

Follow-up tasks to finish parity with Mesen2's open-bus behaviour on the PPU side.

1. ~~**Add a PPU-specific open-bus latch with decay**~~
   - ~~Mirror `bus::open_bus::OpenBus` semantics (per-bit decay ~1 second of CPU time, last-driven value).~~
   - ~~Decide whether to reuse `OpenBus` generically or add a PPU-local copy to avoid accidental cross-clock coupling with the CPU bus.~~
   - ~~Provide `step()`, `sample()`, and `latch()` entry points so reads from floating sources get decayed values and writes refresh decay deadlines.~~

2. **Clock the PPU open bus on every relevant access**
   - ~~CPU-visible register mirror (`cpu_read`/`cpu_write`): step before accesses; latch on writes; latch returned values for reads.~~
   - Internal VRAM/palette/OAM accesses that leave data on the PPU data bus (e.g., `read_vram_data`, palette reads, OAM reads outside rendering) should latch their results.
   - For reads that are “open” on hardware (e.g., OAMDATA during rendering, write-only registers), return the decayed sample instead of a fixed constant (unless hardware dictates a specific value).

3. **Hook decay tick into PPU timing**
   - Increment the PPU bus tick once per CPU-visible PPU access and for internal PPU fetches that would keep the data bus charged.
   - Keep tick units aligned with CPU bus timing expectations (Mesen2 uses CPU-cycle granularity for the decay window; match that to stay compatible with test ROMs).

4. **Reset/initialisation**
   - ~~Clear/latch to 0 on `Ppu::new()`/`reset()`.~~
   - Ensure state is saved/restored appropriately if snapshots are ever added for PPU.

5. **Compatibility checks**
   - Re-run `cpu_exec_space/test_cpu_exec_space_ppuio.nes` and `cpu_dummy_writes_ppumem.nes` to confirm open-bus expectations.
   - Watch for regressions in PPU reads that expect status bit overlays (e.g., `$2002` should still merge VBlank/flags with bus bits).

6. **Documentation**
   - Update `README.md` accuracy notes to mention PPU-side open bus and decay once implemented.
   - Brief code comment near the new latch describing the Mesen2 alignment and decay window choice.

# TODO: PPU vs Mesen2 parity

High-level behaviour differences between `nesium-core`'s NES PPU and Mesen2's `NesPpu` to revisit.

1. **$2007 read/write timing and VRAM increment**
   - Mesen2 delays the VRAM address increment by 1 PPU cycle after `$2007` reads/writes (`_needVideoRamIncrement` + `UpdateVideoRamAddr()`), and ignores a second `$2007` read when it happens on consecutive CPU cycles (`_ignoreVramRead`).
   - Nesium currently increments `v` immediately in `write_vram_data`/`read_vram_data` ~~and has no special handling for back-to-back `$2007` reads~~ (now uses a small ignore window, but still has no 1-dot increment delay).
   - TODO: Consider adding a small pending increment latch (similar to `_needVideoRamIncrement`) to better match Mesen2 / hardware tests (e.g. `full_palette`).

2. **PPUSCROLL/PPUADDR scroll glitches ($2000/$2005/$2006)**
   - Mesen2 implements the well-known scroll glitches when writing to `$2000/$2005/$2006` on specific cycles (e.g. 257 and at 8-dot boundaries) via `ProcessTmpAddrScrollGlitch` and the `EnablePpu2006ScrollGlitch` setting, corrupting `v`/`t` in the same way as hardware.
   - Nesium currently performs clean copies between `t` and `v` (`copy_horizontal_scroll`/`copy_vertical_scroll`) with no glitch emulation; TODO comments in `ppu.rs` acknowledge this.
   - TODO: If we aim for full Mesen2 parity, port the scroll glitch behaviour in a minimal form (gated behind a config flag) so `$2000/$2005/$2006` writes during rendering can perturb `VramAddr` like on 2C02.

3. **OAMDATA ($2004) read/write behaviour**
   - ~~Writes: Mesen2 ignores writes to `$2004` during rendering and instead performs the “high 6 bits only” increment (`_spriteRamAddr = (_spriteRamAddr + 4) & 0xFF`), which models the hardware OAMADDR glitch.~~
   - Reads: During rendering, Mesen2 returns the value currently on the internal OAM bus (`_oamCopybuffer` or secondary OAM contents) rather than primary OAM; nesium approximates this as a fixed `0xFF` placeholder.
   - TODO: ~~Align `$2004` writes with Mesen2 (no OAM modification + glitchy increment during rendering) and,~~ if needed, refine `$2004` reads to expose the internal OAM bus state instead of `0xFF`.

4. **Sprite evaluation and overflow bug**
   - Mesen2 has a very detailed implementation of the sprite overflow bug during the overflow scan phase, including the exact `n`/`m` increment pattern and the “realign” behaviour after overflow (`_overflowBugCounter`, `_oamCopyDone`, etc.).
   - Nesium has a simplified `SpriteEvalPhase::OverflowScan` that captures the general effect (overflow flag set when more than 8 in-range sprites) but not all the edge-case patterns (marked with `TODO(sprite-overflow)` in `sprite_state.rs`/`ppu.rs`).
   - TODO: Tighten the overflow scan logic to match Mesen2’s n/m stepping and `overflow_bug_counter` semantics so tests like `oam_stress` and sprite overflow edge cases behave identically.

5. **PPUMASK grayscale and color emphasis bits**
   - In Mesen2, `$2001` grayscale and R/G/B emphasis bits are applied when reading palette RAM and when writing the final framebuffer (`UpdateGrayscaleAndIntensifyBits`, `_paletteRamMask`, `_intensifyColorBits`).
   - Nesium defines the corresponding `Mask` bits but does not yet apply them to palette reads or to the framebuffer; ~~palette writes also don’t clamp values to 0x3F like Mesen2.~~ (palette writes are now clamped to 0x3F).
   - TODO: Decide whether to apply grayscale/ emphasis in the core PPU (palette path) or in the front-end video pipeline, and add masking to bring behaviour in line with Mesen2 where it matters for test ROMs.

6. **Region-specific timing (NTSC vs PAL/Dendy)**
   - Mesen2 supports NTSC/PAL/Dendy via `UpdateTimings` and changes `_nmiScanline`, `_vblankEnd`, master clock divider and PAL-specific OAM refresh behaviour; nesium currently targets NTSC only (`SCANLINES_PER_FRAME = 262`, fixed odd-frame skip logic).
   - TODO: If multi-region support is desired, introduce a small region enum + timing table for scanline counts, NMI/vblank start/end, and PAL-specific sprite eval behaviour, guided by Mesen2’s `UpdateTimings`.

7. **$2007 palette reads / open bus interaction**
   - Mesen2 merges palette RAM reads with open-bus high bits and applies grayscale masking; nesium’s `read_vram_data` currently reads directly from `PaletteRam` without mixing in bus bits or mask.
   - TODO: Consider merging palette reads with the PPU-side open-bus latch (once implemented) and optionally applying a grayscale mask like Mesen2’s `_paletteRamMask`, to better match test ROM expectations.

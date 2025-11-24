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
   - ~~Nesium currently performs clean copies between `t` and `v` (`copy_horizontal_scroll`/`copy_vertical_scroll`) with no glitch emulation; TODO comments in `ppu.rs` acknowledge this.~~ (nesium now emulates the basic scroll glitch when `$2000/$2005/$2006` writes land on dot 257 of a visible scanline while rendering is enabled.)
   - TODO: If we aim for full Mesen2 parity, extend the current dot-257 glitch to also cover the more subtle `$2006` AND-style corruption and any additional 8-dot-boundary effects described in `ProcessTmpAddrScrollGlitch`.

3. **OAMDATA ($2004) read/write behaviour**
   - ~~Writes: Mesen2 ignores writes to `$2004` during rendering and instead performs the “high 6 bits only” increment (`_spriteRamAddr = (_spriteRamAddr + 4) & 0xFF`), which models the hardware OAMADDR glitch.~~
   - ~~Reads: During rendering, Mesen2 returns the value currently on the internal OAM bus (`_oamCopybuffer` or secondary OAM contents) rather than primary OAM; nesium approximates this as a fixed `0xFF` placeholder.~~ (nesium now tracks an internal OAM bus copybuffer and maps `$2004` reads in the 257..=320 sprite-fetch window to the corresponding secondary OAM address.)
   - TODO: Keep iterating on corner cases (sprite overflow / PAL refresh behaviour) if specific test ROMs demonstrate remaining differences in `$2004` bus contents.

4. **Sprite evaluation and overflow bug**
   - Mesen2 has a very detailed implementation of the sprite overflow bug during the overflow scan phase, including the exact `n`/`m` increment pattern and the “realign” behaviour after overflow (`_overflowBugCounter`, `_oamCopyDone`, etc.).
   - Nesium has a simplified `SpriteEvalPhase::OverflowScan` that captures the general effect (overflow flag set when more than 8 in-range sprites) but not all the edge-case patterns (marked with `TODO(sprite-overflow)` in `sprite_state.rs`/`ppu.rs`).
   - TODO: Tighten the overflow scan logic to match Mesen2’s n/m stepping and `overflow_bug_counter` semantics so tests like `oam_stress` and sprite overflow edge cases behave identically.

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

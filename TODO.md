# TODO: PPU Open Bus (Mesen2-aligned)

Follow-up tasks to finish parity with Mesen2's open-bus behaviour on the PPU side.

1. **Add a PPU-specific open-bus latch with decay**
   - Mirror `bus::open_bus::OpenBus` semantics (per-bit decay ~1 second of CPU time, last-driven value).
   - Decide whether to reuse `OpenBus` generically or add a PPU-local copy to avoid accidental cross-clock coupling with the CPU bus.
   - Provide `step()`, `sample()`, and `latch()` entry points so reads from floating sources get decayed values and writes refresh decay deadlines.

2. **Clock the PPU open bus on every relevant access**
   - CPU-visible register mirror (`cpu_read`/`cpu_write`): step before accesses; latch on writes; latch returned values for reads.
   - Internal VRAM/palette/OAM accesses that leave data on the PPU data bus (e.g., `read_vram_data`, palette reads, OAM reads outside rendering) should latch their results.
   - For reads that are “open” on hardware (e.g., OAMDATA during rendering, write-only registers), return the decayed sample instead of a fixed constant (unless hardware dictates a specific value).

3. **Hook decay tick into PPU timing**
   - Increment the PPU bus tick once per CPU-visible PPU access and for internal PPU fetches that would keep the data bus charged.
   - Keep tick units aligned with CPU bus timing expectations (Mesen2 uses CPU-cycle granularity for the decay window; match that to stay compatible with test ROMs).

4. **Reset/initialisation**
   - Clear/latch to 0 on `Ppu::new()`/`reset()`.
   - Ensure state is saved/restored appropriately if snapshots are ever added for PPU.

5. **Compatibility checks**
   - Re-run `cpu_exec_space/test_cpu_exec_space_ppuio.nes` and `cpu_dummy_writes_ppumem.nes` to confirm open-bus expectations.
   - Watch for regressions in PPU reads that expect status bit overlays (e.g., `$2002` should still merge VBlank/flags with bus bits).

6. **Documentation**
   - Update `README.md` accuracy notes to mention PPU-side open bus and decay once implemented.
   - Brief code comment near the new latch describing the Mesen2 alignment and decay window choice.


# NES Emulator in Rust

A cycle-accurate NES emulator written in Rust, designed to faithfully replicate the behavior of the Nintendo Entertainment System hardware. This project strives to provide precise emulation of the CPU, PPU, APU, and other critical components, ensuring that every game runs as it would on the original hardware.

This emulator’s design and implementation draw heavily from the excellent Mesen2 project. Mesen2’s documentation, code structure, and many of its implementation ideas (especially around timing, open-bus behaviour, and audio mixing) have been an invaluable reference. Huge thanks to the Mesen2 authors and contributors for making such a high‑quality emulator available.

## Key Features:
- **Cycle-accurate emulation**: Every clock cycle is emulated precisely to ensure accurate game behavior.
- **CPU (6502) Emulation**: Full emulation of the 6502 processor with support for all instructions.
- **PPU Emulation**: Accurate rendering of graphics, including support for palettes, sprites, and background layers.
- **APU Emulation**: Recreates sound processing with support for the NES sound channels.
- **Compatibility**: Supports a variety of NES games, with ongoing improvements to compatibility and performance.

## Current Status:
- Active development with ongoing improvements to accuracy, performance, and compatibility.
- Still in the early stages, but several key components are already functional.

## Mapper support
- [x] 0 – NROM
- [x] 1 – MMC1 (SxROM)
- [x] 2 – UxROM
- [x] 3 – CNROM
- [x] 4 – MMC3 (full IRQ + CHR/PRG/mirroring)
- [x] 5 – MMC5 (core features; ExRAM/nametable TODO)
- [x] 6 – Front Fareast Magic Card
- [x] 7 – AxROM
- [x] 8 – FFE GUI mode
- [x] 9 – MMC2
- [x] 10 – MMC4
- [x] 11 – Color Dreams
- [x] 13 – CPROM
- [x] 19 – Namco 163 (basic audio)
- [x] 21 – VRC4a/VRC4c
- [x] 23 – VRC2b/VRC4e
- [x] 25 – VRC4b/VRC4d/VRC2c
- [x] 26 – VRC6b (expansion audio stubbed; CHR-ROM nametable modes TODO)
- [x] 34 – BNROM / NINA-001
- [x] 66 – GxROM / GNROM
- [x] 71 – Camerica / Codemasters
- [x] 78 – Irem 74HC161/32 (Holy Diver) – simple IRQ/mirroring
- [x] 85 – VRC7 (audio stubbed; enable OPLL later)
- [x] 90 – JY Company multicart (simplified; advanced NT/IRQ behaviour TODO)
- [x] 119 – TQROM (MMC3 with CHR ROM/RAM bit) – verify against edge cases
- [x] 228 – Action 52 / Cheetahmen II

### Mapper gaps / caveats
- **MMC5 (mapper 5)**: ExRAM-as-nametable modes and extended attribute/fill features are still TODO; expansion audio unimplemented.
- **Namco 163 (mapper 19)**: Only basic audio routing implemented; full 8-channel wavetable behaviour and per-channel timing/phase wrapping remain to be completed.
- **VRC6b (mapper 26)**: Expansion audio stubbed; CHR-ROM nametable modes not finished.
- **VRC7 (mapper 85)**: Audio core not wired; OPLL implementation pending.
- **J.Y. Company 90**: Multicart NT/IRQ tricks are simplified; advanced nametable/IRQ behaviour needs work.
- **TQROM (mapper 119)**: Edge cases around CHR ROM/RAM bit toggling still need verification.
- **Action 52 / Cheetahmen II (mapper 228)**: Mapper RAM window behaviour is minimal; verify against all carts.
- **Generic**: Bus conflict handling for certain discrete boards (e.g., some UNROM/CNROM variants) is not fully modelled yet.

## Accuracy notes
- **Open bus (Mesen2-style)**: CPU bus keeps a decaying latch (~1s of CPU cycles) and feeds it back for write-only/unmapped reads ($4000-$4013, $4014 read, $4018-$401F, or no cartridge PRG space). PPU register traffic refreshes the latch so Blargg/Mesen2 open-bus expectations hold.

## Test ROM status

Nesium integrates a large number of NES test ROM suites (via `rom_suites.rs`) to validate CPU, PPU, APU, and mapper behaviour. The tables below summarize which suites currently pass automatically, which are interactive/manual, and which are still marked as failing/ignored and need more work.

Legend:

- ✅: Enabled automated tests (no `#[ignore]`) that currently pass  
- ❌: Tests marked with `#[ignore = "this test fails and needs investigation"]`  
- 🔶: Interactive/manual ROMs (e.g., controller/visual tests)  
- 🛠: Debug-only helper tests, not counted toward pass/fail  

### Automatically passing ROM suites (✅)

| Suite name                  | ROM directory               | Notes                                        | TASVideos accuracy-required |
| --------------------------- | --------------------------- | -------------------------------------------- | --------------------------- |
| `_240pee_suite`             | `240pee/*.nes`              | TV colour diversity / timing test            | No                          |
| `mmc1_a12_suite`            | `MMC1_A12/*.nes`            | MMC1 A12 line behaviour                      | No                          |
| `apu_mixer_suite`           | `apu_mixer/*.nes`           | APU mixer / TASVideos test set               | Yes                         |
| `apu_reset_suite`           | `apu_reset/*.nes`           | APU reset behaviour                          | Yes                         |
| `branch_timing_tests_suite` | `branch_timing_tests/*.nes` | Branch instruction timing (zero-page result) | Yes                         |
| `cpu_dummy_reads_suite`     | `cpu_dummy_reads/*.nes`     | CPU dummy read behaviour                     | Yes                         |
| `cpu_dummy_writes_suite`    | `cpu_dummy_writes/*.nes`    | CPU dummy write behaviour                    | Yes                         |
| `cpu_timing_test6_suite`    | `cpu_timing_test6/*.nes`    | TASVideos CPU timing (TV SHA1)               | Yes                         |
| `instr_misc_suite`          | `instr_misc/*.nes`          | Misc instruction behaviour                   | Yes                         |
| `instr_test_v3_suite`       | `instr_test-v3/*.nes`       | Blargg instruction test v3                   | Yes                         |
| `instr_test_v5_suite`       | `instr_test-v5/*.nes`       | Blargg instruction test v5                   | Yes                         |
| `instr_timing_suite`        | `instr_timing/*.nes`        | Instruction timing                           | Yes                         |
| `nes_instr_test_suite`      | `nes_instr_test/*.nes`      | Additional instruction behaviour tests       | Yes                         |
| `ny2011_suite`              | `ny2011/*.nes`              | Visual diversity / timing                    | No                          |
| `oam_read_suite`            | `oam_read/*.nes`            | OAM read behaviour                           | Yes                         |
| `oam_stress_suite`          | `oam_stress/*.nes`          | OAM stress / overflow conditions             | Yes                         |
| `spritecans_2011_suite`     | `spritecans-2011/*.nes`     | Visual diversity / sprite stress             | No                          |
| `stomper_suite`             | `stomper/*.nes`             | Visual diversity / timing                    | No                          |
| `tutor_suite`               | `tutor/*.nes`               | Visual diversity / reference demo            | No                          |
| `window5_suite`             | `window5/*.nes`             | Colour windowing tests (NTSC/PAL)            | No                          |

### Interactive / manual ROMs (🔶)

These ROMs are designed for interactive/manual verification and do not expose a simple $6000 state byte or TV hash protocol. They are wired into the test harness but kept under `#[ignore]` and should be checked by hand.

| Suite name             | ROM directory          | Notes                                                                               | TASVideos accuracy-required |
| ---------------------- | ---------------------- | ----------------------------------------------------------------------------------- | --------------------------- |
| `paddletest3_manual`   | `PaddleTest3/*.nes`    | Paddle/analog controller test; follow ROM `Info.txt` for instructions               | No                          |
| `tvpassfail_manual`    | `tvpassfail/*.nes`     | TV characteristics (NTSC chroma/luma, artifacts); verify visually                   | No                          |
| `vaus_test_manual`     | `vaus-test/*.nes`      | Arkanoid Vaus controller test (interactive)                                         | No                          |
| `vbl_nmi_timing_suite` | `vbl_nmi_timing/*.nes` | VBL/NMI timing; currently treated as manual, even though a zeropage protocol exists | Yes                         |

### Failing / ignored ROM suites (❌)

The following suites are currently marked with `#[ignore = "this test fails and needs investigation"]`. They highlight areas where Nesium’s behaviour still diverges from reference emulators and hardware.

| Suite name                           | ROM directory                        | Notes                                        | TASVideos accuracy-required |
| ------------------------------------ | ------------------------------------ | -------------------------------------------- | --------------------------- |
| `apu_test_suite`                     | `apu_test/*.nes`                     | APU accuracy tests (including `rom_singles`) | Yes                         |
| `blargg_apu_2005_07_30_suite`        | `blargg_apu_2005.07.30/*.nes`        | Early Blargg APU tests                       | Yes                         |
| `blargg_litewall_suite`              | `blargg_litewall/*.nes`              | Litewall / timing-related tests              | No                          |
| `blargg_nes_cpu_test5_suite`         | `blargg_nes_cpu_test5/*.nes`         | CPU precision tests                          | Yes                         |
| `blargg_ppu_tests_2005_09_15b_suite` | `blargg_ppu_tests_2005.09.15b/*.nes` | PPU palette/VRAM/scrolling behaviour         | Yes                         |
| `cpu_exec_space_suite`               | `cpu_exec_space/*.nes`               | CPU exec space tests (APU/PPU I/O)           | Yes                         |
| `cpu_interrupts_v2_suite`            | `cpu_interrupts_v2/*.nes`            | NMI/IRQ/BRK/DMA interrupt timing             | Yes                         |
| `cpu_reset_suite`                    | `cpu_reset/*.nes`                    | Post-reset RAM/register state                | Yes                         |
| `dmc_dma_during_read4_suite`         | `dmc_dma_during_read4/*.nes`         | DMC DMA interaction with CPU read cycles     | Yes                         |
| `dmc_tests_suite`                    | `dmc_tests/*.nes`                    | DMC buffer/delay/IRQ behaviour               | Yes                         |
| `dpcmletterbox_suite`                | `dpcmletterbox/*.nes`                | DPCM-related visual/audio test               | Yes                         |
| `exram_suite`                        | `exram/*.nes`                        | MMC5 ExRAM behaviour (currently failing)     | No                          |
| `full_palette_suite`                 | `full_palette/*.nes`                 | Full palette rendering and emphasis tests    | No                          |
| `m22chrbankingtest_suite`            | `m22chrbankingtest/*.nes`            | Mapper 22 CHR banking behaviour              | No                          |
| `mmc3_irq_tests_suite`               | `mmc3_irq_tests/*.nes`               | MMC3 IRQ behaviour                           | Yes                         |
| `mmc3_test_suite`                    | `mmc3_test/*.nes`                    | MMC3/MMC6 functional tests                   | Yes                         |
| `mmc3_test_2_suite`                  | `mmc3_test_2/rom_singles/*.nes`      | Second MMC3 test set                         | Yes                         |
| `mmc5test_suite`                     | `mmc5test/*.nes`                     | MMC5 functional tests                        | Yes                         |
| `mmc5test_v2_suite`                  | `mmc5test_v2/*.nes`                  | MMC5 test set v2                             | Yes                         |
| `nes15_1_0_0_suite`                  | `nes15-1.0.0/*.nes`                  | `nes15` series tests (NTSC/PAL)              | Yes                         |
| `nmi_sync_suite`                     | `nmi_sync/*.nes`                     | NMI sync behaviour                           | Yes                         |
| `nrom368_suite`                      | `nrom368/*.nes`                      | NROM-368 mapping tests                       | No                          |
| `other_suite`                        | `other/*.nes`                        | Misc demos/tests bundled with nes-test-roms  | No                          |
| `pal_apu_tests_suite`                | `pal_apu_tests/*.nes`                | PAL APU behaviour                            | Yes                         |
| `ppu_open_bus_suite`                 | `ppu_open_bus/*.nes`                 | PPU open-bus behaviour                       | Yes                         |
| `ppu_read_buffer_suite`              | `ppu_read_buffer/*.nes`              | PPU read buffer behaviour                    | Yes                         |
| `ppu_vbl_nmi_suite`                  | `ppu_vbl_nmi/*.nes`                  | PPU VBL/NMI timing                           | Yes                         |
| `read_joy3_suite`                    | `read_joy3/*.nes`                    | Controller read timing                       | Yes                         |
| `scanline_suite`                     | `scanline/*.nes`                     | Scanline timing                              | Yes                         |
| `scanline_a1_suite`                  | `scanline-a1/*.nes`                  | Alternate scanline tests                     | Yes                         |
| `scrolltest_suite`                   | `scrolltest/*.nes`                   | Scrolling behaviour                          | Yes                         |
| `sprdma_and_dmc_dma_suite`           | `sprdma_and_dmc_dma/*.nes`           | Sprite DMA and DMC DMA interaction           | Yes                         |
| `sprite_hit_tests_2005_10_05_suite`  | `sprite_hit_tests_2005.10.05/*.nes`  | Sprite 0 hit timing and edge cases           | Yes                         |
| `sprite_overflow_tests_suite`        | `sprite_overflow_tests/*.nes`        | Sprite overflow behaviour                    | Yes                         |
| `volume_tests_suite`                 | `volume_tests/*.nes`                 | Volume/mixing behaviour                      | Yes                         |

### Debug helper tests (🛠)

These helpers are wired to specific ROMs and used for local debugging (e.g., logging APU state). They are not treated as pass/fail for emulator accuracy.

| Test name                      | Purpose                                       |
| ------------------------------ | --------------------------------------------- |
| `apu_reset_4017_timing_debug`  | Debug helper for `apu_reset/4017_timing.nes`  |
| `apu_reset_4017_written_debug` | Debug helper for `apu_reset/4017_written.nes` |

## Disclaimer

This project is a fan-made, non-commercial emulator intended for educational and preservation purposes. It is not affiliated with, endorsed, or sponsored by Nintendo or any other rights holder. You are solely responsible for complying with local laws and for ensuring that any ROMs or other copyrighted content you use with this emulator are obtained and used legally (for example, from cartridges you personally own).

## Contributions:
Feel free to fork the project, open issues, and submit pull requests. Contributions are welcome as we work to improve accuracy and expand the feature set.

## License

Nesium is distributed under the terms of the GNU General Public License, version 3 or (at your option) any later version (GPL‑3.0‑or‑later). See `LICENSE.md` for the full text.

This project also includes Shay Green’s `blip_buf` library (used via the `nesium-blip` crate), which is licensed under the GNU Lesser General Public License v2.1. The relevant license text is included alongside the imported sources in `crates/nesium-blip/csrc/license.md`.

## Libretro bindings
The workspace includes the `libretro-bridge` crate, which automatically generates Rust bindings for the upstream `libretro.h` header via `bindgen`. The build script fetches the most recent header at compile time (with a vendored fallback for offline builds) so Nesium—and any other Rust project—can integrate with the libretro ecosystem as soon as API changes land upstream.

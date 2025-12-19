# NES Emulator in Rust

A cycle-accurate NES emulator written in Rust, designed to faithfully replicate the behavior of the Nintendo Entertainment System hardware. This project strives to provide precise emulation of the CPU, PPU, APU, and other critical components, ensuring that every game runs as it would on the original hardware.

This emulator’s design and implementation draw heavily from the excellent Mesen2 project. Mesen2’s documentation, code structure, and many of its implementation ideas (especially around timing, open-bus behaviour, and audio mixing) have been an invaluable reference. Huge thanks to the Mesen2 authors and contributors for making such a high‑quality emulator available.

## Key Features

- **Cycle-accurate emulation**: Every clock cycle is emulated precisely to ensure accurate game behavior.
- **CPU (6502) Emulation**: Full emulation of the 6502 processor with support for all instructions.
- **PPU Emulation**: Accurate rendering of graphics, including support for palettes, sprites, and background layers.
- **APU Emulation**: Recreates sound processing with support for the NES sound channels.
- **Compatibility**: Supports a variety of NES games, with ongoing improvements to compatibility and performance.

## Current Status

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

## Test ROM status

Nesium integrates a large number of NES test ROM suites (via `rom_suites.rs`) to validate CPU, PPU, APU, and mapper behaviour. The tables below summarize which suites currently pass automatically, which are interactive/manual, and which are still marked as failing/ignored and need more work.

Legend:

- ✅: Enabled automated tests (no `#[ignore]`) that currently pass  
- ❌: Tests marked with `#[ignore = "this test fails and needs investigation"]`  
- 🔶: Interactive/manual ROMs (e.g., controller/visual tests)  

### Automatically passing ROM suites (✅)

| Suite name                  | Notes                                        | TASVideos accuracy-required |
| --------------------------- | -------------------------------------------- | --------------------------- |
| `_240pee_suite`             | TV colour diversity / timing test            | No                          |
| `mmc1_a12_suite`            | MMC1 A12 line behaviour                      | No                          |
| `apu_mixer_suite`           | APU mixer / TASVideos test set               | Yes                         |
| `apu_reset_suite`           | APU reset behaviour                          | Yes                         |
| `blargg_nes_cpu_test5_suite`| CPU precision tests                          | Yes                         |
| `blargg_ppu_tests_2005_09_15b_suite` | PPU palette/VRAM/scrolling behaviour         | Yes                         |
| `branch_timing_tests_suite` | Branch instruction timing (zero-page result) | Yes                         |
| `cpu_dummy_reads_suite`     | CPU dummy read behaviour                     | Yes                         |
| `cpu_dummy_writes_suite`    | CPU dummy write behaviour                    | Yes                         |
| `cpu_reset_suite`           | Post-reset RAM/register state                | Yes                         |
| `cpu_timing_test6_suite`    | TASVideos CPU timing (TV SHA1)               | Yes                         |
| `instr_misc_suite`          | Misc instruction behaviour                   | Yes                         |
| `instr_test_v3_suite`       | Blargg instruction test v3                   | Yes                         |
| `instr_test_v5_suite`       | Blargg instruction test v5                   | Yes                         |
| `instr_timing_suite`        | Instruction timing                           | Yes                         |
| `nes_instr_test_suite`      | Additional instruction behaviour tests       | Yes                         |
| `ny2011_suite`              | Visual diversity / timing                    | No                          |
| `oam_read_suite`            | OAM read behaviour                           | Yes                         |
| `oam_stress_suite`          | OAM stress / overflow conditions             | Yes                         |
| `ppu_open_bus_suite`        | PPU open-bus behaviour                       | Yes                         |
| `ppu_read_buffer_suite`     | PPU read buffer behaviour                    | Yes                         |
| `ppu_vbl_nmi_suite`         | PPU VBL/NMI timing                           | Yes                         |
| `sprite_hit_tests_2005_10_05_suite` | Sprite 0 hit timing and edge cases           | Yes                         |
| `spritecans_2011_suite`     | Visual diversity / sprite stress             | No                          |
| `stomper_suite`             | Visual diversity / timing                    | No                          |
| `tutor_suite`               | Visual diversity / reference demo            | No                          |
| `vbl_nmi_timing_suite`      | VBL/NMI timing (zeropage result)             | Yes                         |
| `window5_suite`             | Colour windowing tests (NTSC/PAL)            | No                          |

### Interactive / manual ROMs (🔶)

These ROMs are designed for interactive/manual verification and do not expose a simple $6000 state byte or TV hash protocol. They are wired into the test harness but kept under `#[ignore]` and should be checked by hand.

| Suite name             | Notes                                                                               | TASVideos accuracy-required |
| ---------------------- | ----------------------------------------------------------------------------------- | --------------------------- |
| `paddletest3_manual`   | Paddle/analog controller test; follow ROM `Info.txt` for instructions               | No                          |
| `tvpassfail_manual`    | TV characteristics (NTSC chroma/luma, artifacts); verify visually                   | No                          |
| `vaus_test_manual`     | Arkanoid Vaus controller test (interactive)                                         | No                          |

### Failing / ignored ROM suites (❌)

The following suites are currently marked with `#[ignore = "this test fails and needs investigation"]`. They highlight areas where Nesium’s behaviour still diverges from reference emulators and hardware.

| Suite name                           | Notes                                        | TASVideos accuracy-required |
| ------------------------------------ | -------------------------------------------- | --------------------------- |
| `apu_test_suite`                     | APU accuracy tests (including `rom_singles`) | Yes                         |
| `blargg_apu_2005_07_30_suite`        | Early Blargg APU tests                       | Yes                         |
| `blargg_litewall_suite`              | Litewall / timing-related tests              | No                          |
| `cpu_exec_space_suite`               | CPU exec space tests (APU/PPU I/O)           | Yes                         |
| `cpu_interrupts_v2_suite`            | NMI/IRQ/BRK/DMA interrupt timing             | Yes                         |
| `dmc_dma_during_read4_suite`         | DMC DMA interaction with CPU read cycles     | Yes                         |
| `dmc_tests_suite`                    | DMC buffer/delay/IRQ behaviour               | Yes                         |
| `dpcmletterbox_suite`                | DPCM-related visual/audio test               | Yes                         |
| `exram_suite`                        | MMC5 ExRAM behaviour (currently failing)     | No                          |
| `full_palette_suite`                 | Full palette rendering and emphasis tests    | No                          |
| `m22chrbankingtest_suite`            | Mapper 22 CHR banking behaviour              | No                          |
| `mmc3_irq_tests_suite`               | MMC3 IRQ behaviour                           | Yes                         |
| `mmc3_test_suite`                    | MMC3/MMC6 functional tests                   | Yes                         |
| `mmc3_test_2_suite`                  | Second MMC3 test set                         | Yes                         |
| `mmc5test_suite`                     | MMC5 functional tests                        | Yes                         |
| `mmc5test_v2_suite`                  | MMC5 test set v2                             | Yes                         |
| `nes15_1_0_0_suite`                  | `nes15` series tests (NTSC/PAL)              | Yes                         |
| `nmi_sync_suite`                     | NMI sync behaviour                           | Yes                         |
| `nrom368_suite`                      | NROM-368 mapping tests                       | No                          |
| `other_suite`                        | Misc demos/tests bundled with nes-test-roms  | No                          |
| `pal_apu_tests_suite`                | PAL APU behaviour                            | Yes                         |
| `read_joy3_suite`                    | Controller read timing                       | Yes                         |
| `scanline_suite`                     | Scanline timing                              | Yes                         |
| `scanline_a1_suite`                  | Alternate scanline tests                     | Yes                         |
| `scrolltest_suite`                   | Scrolling behaviour                          | Yes                         |
| `sprdma_and_dmc_dma_suite`           | Sprite DMA and DMC DMA interaction           | Yes                         |
| `sprite_overflow_tests_suite`        | Sprite overflow behaviour                    | Yes                         |
| `volume_tests_suite`                 | Volume/mixing behaviour                      | Yes                         |

## Disclaimer

This project is a fan-made, non-commercial emulator intended for educational and preservation purposes. It is not affiliated with, endorsed, or sponsored by Nintendo or any other rights holder. You are solely responsible for complying with local laws and for ensuring that any ROMs or other copyrighted content you use with this emulator are obtained and used legally (for example, from cartridges you personally own).

## Contributions

Feel free to fork the project, open issues, and submit pull requests. Contributions are welcome as we work to improve accuracy and expand the feature set.

## License

Nesium is distributed under the terms of the GNU General Public License, version 3 or (at your option) any later version (GPL‑3.0‑or‑later). See `LICENSE.md` for the full text.

This project also includes Shay Green’s `blip_buf` library (used via the `nesium-blip` crate), which is licensed under the GNU Lesser General Public License v2.1. The relevant license text is included alongside the imported sources in `crates/nesium-blip/csrc/license.md`.

## Libretro bindings

The workspace includes the `libretro-bridge` crate, which automatically generates Rust bindings for the upstream `libretro.h` header via `bindgen`. The build script fetches the most recent header at compile time (with a vendored fallback for offline builds) so Nesium—and any other Rust project—can integrate with the libretro ecosystem as soon as API changes land upstream.

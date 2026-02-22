# <img src="icon.svg" width="48" align="center" /> Nesium

<div align="center">

[![Rust](https://github.com/mikai233/nesium/actions/workflows/rust.yml/badge.svg)](https://github.com/mikai233/nesium/actions/workflows/rust.yml)
[![Flutter](https://github.com/mikai233/nesium/actions/workflows/flutter.yml/badge.svg)](https://github.com/mikai233/nesium/actions/workflows/flutter.yml)
[![Web Demo](https://img.shields.io/website?label=play%20online&url=https%3A%2F%2Fmikai233.github.io%2Fnesium%2F)](https://mikai233.github.io/nesium/)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE.md)

<p>
  <img src="https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/Flutter-%2302569B.svg?style=flat&logo=Flutter&logoColor=white" alt="Flutter" />
  <img src="https://img.shields.io/badge/WebAssembly-654FF0.svg?style=flat&logo=webassembly&logoColor=white" alt="Wasm" />
</p>

[**中文说明**](./README_zh.md)

</div>

A cycle-accurate NES emulator written in Rust, designed to faithfully replicate the behavior of the Nintendo Entertainment System hardware. This project strives to provide precise emulation of the CPU, PPU, APU, and other critical components, ensuring that every game runs as it would on the original hardware.

This emulator’s design and implementation draw heavily from the excellent [Mesen2](https://github.com/SourMesen/Mesen2) project. Mesen2’s documentation, code structure, and many of its implementation ideas (especially around timing, open-bus behaviour, and audio mixing) have been an invaluable reference. Huge thanks to the Mesen2 authors and contributors for making such a high‑quality emulator available.

## Key Features

- **Cycle-accurate emulation**: Every clock cycle is emulated precisely to ensure accurate game behavior.
- **CPU (6502) Emulation**: Full emulation of the 6502 processor with support for all instructions.
- **PPU Emulation**: Accurate rendering of graphics, including support for palettes, sprites, and background layers.
- **APU Emulation**: Recreates sound processing with support for the NES sound channels.
- **Compatibility**: Supports a variety of NES games, with ongoing improvements to compatibility and performance.

## UI frontends

This repository currently ships **two** UI frontends:

- **`nesium-egui`** (`apps/nesium-egui`) — A lightweight desktop frontend built with `egui`. It has a small footprint and provides the essentials for **quick debugging and development**.
  - ![](https://img.shields.io/badge/Windows-x86_64/arm64-blue?logo=windows) ![](https://img.shields.io/badge/macOS-Universal-black?logo=apple) ![](https://img.shields.io/badge/Linux-x86_64/arm64-orange?logo=linux)
- **`nesium-flutter`** (`apps/nesium_flutter`) — A modern frontend built with **Flutter**. It aims for a more polished UI and broader cross‑platform reach than the `egui` app.
  - ![](https://img.shields.io/badge/Windows-x86_64-blue?logo=windows) ![](https://img.shields.io/badge/macOS-Universal-black?logo=apple) ![](https://img.shields.io/badge/Linux-x86_64/arm64-orange?logo=linux) ![](https://img.shields.io/badge/Android-Multi--arch-green?logo=android) ![](https://img.shields.io/badge/iOS-Supported-lightgrey?logo=apple)
- **Web build (play online)** — https://mikai233.github.io/nesium/ (Runs in the browser via high-performance **Flutter WASM (dart2wasm)** + Web Worker + Rust WASM).
  - ![](https://img.shields.io/badge/Web-WasmGC-purple?logo=webassembly) (Chrome/Edge 119+, Firefox 120+)

## Current Status

- Active development with ongoing improvements to accuracy, performance, and compatibility.
- Still in the early stages, but several key components are already functional.

## Roadmap

The long-term vision for Nesium focuses on precision, tooling, and extensibility:

- [ ] **Accurate NES Emulation**:  
    Achieve cycle-perfect accuracy across CPU, PPU, and APU components. The goal is to pass all standard compliance suites (including tricky edge cases in `blargg`'s tests and `nes-test-roms`) and support "unlicensed" or hardware-quirk-dependent titles correctly.

- [ ] **Advanced Debugging Suite**:  
    Implement a comprehensive debugger within the frontend. Planned features include:
    - Real-time disassembly and stepping.
    - Memory inspection/editing (RAM, VRAM, OAM).
    - Nametable, Pattern Table, and Palette viewers.
    - Breakpoint management (execution, read/write, IRQ).

- [ ] **Lua Scripting Integration**:  
    Embed a Lua runtime to enable powerful automation and analysis. This will support:
    - Tool-Assisted Speedrun (TAS) workflows.
    - Custom HUDs and overlays for training or streaming.
    - Automated regression testing scripts.

- [ ] **Netplay**:
    Implement networked multiplayer support for two-player games over the internet.

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
- ℹ️: Tracking/diagnostic ROMs kept under `#[ignore]` by design  

### Automatically passing ROM suites (✅)

| Suite name                           | Notes                                        | TASVideos accuracy-required |
| ------------------------------------ | -------------------------------------------- | --------------------------- |
| `_240pee_suite`                      | TV colour diversity / timing test            | No                          |
| `mmc1_a12_suite`                     | MMC1 A12 line behaviour                      | No                          |
| `apu_mixer_suite`                    | APU mixer / TASVideos test set               | Yes                         |
| `apu_reset_suite`                    | APU reset behaviour                          | Yes                         |
| `apu_test_suite`                     | APU accuracy tests (including `rom_singles`) | Yes                         |
| `blargg_apu_2005_07_30_suite`        | Early Blargg APU tests                       | Yes                         |
| `blargg_nes_cpu_test5_suite`         | CPU precision tests                          | Yes                         |
| `blargg_ppu_tests_2005_09_15b_suite` | PPU palette/VRAM/scrolling behaviour         | Yes                         |
| `branch_timing_tests_suite`          | Branch instruction timing (zero-page result) | Yes                         |
| `cpu_dummy_reads_suite`              | CPU dummy read behaviour                     | Yes                         |
| `cpu_dummy_writes_suite`             | CPU dummy write behaviour                    | Yes                         |
| `cpu_exec_space_suite`               | CPU exec space tests (APU/PPU I/O)           | Yes                         |
| `cpu_interrupts_v2_suite`            | NMI/IRQ/BRK/DMA interrupt timing             | Yes                         |
| `cpu_reset_suite`                    | Post-reset RAM/register state                | Yes                         |
| `cpu_timing_test6_suite`             | TASVideos CPU timing (TV SHA1)               | Yes                         |
| `dmc_dma_during_read4_suite`         | DMC DMA interaction with CPU read cycles     | Yes                         |
| `dmc_tests_suite`                    | DMC buffer/delay/IRQ behaviour               | Yes                         |
| `full_palette_suite`                 | Full palette rendering and emphasis tests (Mesen2 RGB24 baseline) | No      |
| `scanline_suite`                     | Scanline timing (Mesen2 RGB24 multi-frame baseline) | Yes                    |
| `instr_misc_suite`                   | Misc instruction behaviour                   | Yes                         |
| `instr_test_v3_suite`                | Blargg instruction test v3                   | Yes                         |
| `instr_test_v5_suite`                | Blargg instruction test v5                   | Yes                         |
| `instr_timing_suite`                 | Instruction timing                           | Yes                         |
| `mmc3_irq_tests_suite`               | MMC3 IRQ test set (passes required + one revision variant) | Yes            |
| `mmc3_test_suite`                    | MMC3 functional test set (passes required + one MMC3/MMC6 variant) | Yes     |
| `mmc3_test_2_suite`                  | MMC3 test set v2 (passes required + one MMC3/MMC3_alt variant) | Yes      |
| `nes_instr_test_suite`               | Additional instruction behaviour tests       | Yes                         |
| `ny2011_suite`                       | Visual diversity / timing                    | No                          |
| `oam_read_suite`                     | OAM read behaviour                           | Yes                         |
| `oam_stress_suite`                   | OAM stress / overflow conditions             | Yes                         |
| `ppu_open_bus_suite`                 | PPU open-bus behaviour                       | Yes                         |
| `ppu_read_buffer_suite`              | PPU read buffer behaviour                    | Yes                         |
| `ppu_vbl_nmi_suite`                  | PPU VBL/NMI timing                           | Yes                         |
| `sprite_hit_tests_2005_10_05_suite`  | Sprite 0 hit timing and edge cases           | Yes                         |
| `sprite_overflow_tests_suite`        | Sprite overflow behaviour                    | Yes                         |
| `spritecans_2011_suite`              | Visual diversity / sprite stress             | No                          |
| `sprdma_and_dmc_dma_suite`           | Sprite DMA and DMC DMA interaction           | Yes                         |
| `stomper_suite`                      | Visual diversity / timing                    | No                          |
| `tutor_suite`                        | Visual diversity / reference demo            | No                          |
| `vbl_nmi_timing_suite`               | VBL/NMI timing (zeropage result)             | Yes                         |
| `window5_suite`                      | Colour windowing tests (NTSC/PAL)            | No                          |

### Interactive / manual ROMs (🔶)

These ROMs are designed for interactive/manual verification and do not expose a simple $6000 state byte or TV hash protocol. They are wired into the test harness but kept under `#[ignore]` and should be checked by hand.

| Suite name           | Notes                                                                 | TASVideos accuracy-required |
| -------------------- | --------------------------------------------------------------------- | --------------------------- |
| `dpcmletterbox_suite`| Visual DPCM demo ROM; verify manually per `dpcmletterbox/README.txt` | Yes                         |
| `nmi_sync_manual`    | Visual NMI-sync demo ROM; verify manually per `nmi_sync/readme.txt`  | Yes                         |
| `paddletest3_manual` | Paddle/analog controller test; follow ROM `Info.txt` for instructions | No                          |
| `tvpassfail_manual`  | TV characteristics (NTSC chroma/luma, artifacts); verify visually     | No                          |
| `vaus_test_manual`   | Arkanoid Vaus controller test (interactive)                           | No                          |

### Failing / ignored ROM suites (❌)

The following suites are currently marked with `#[ignore = "this test fails and needs investigation"]`. They highlight areas where Nesium’s behaviour still diverges from reference emulators and hardware.

| Suite name                    | Notes                                        | TASVideos accuracy-required |
| ----------------------------- | -------------------------------------------- | --------------------------- |
| `blargg_litewall_suite`       | Litewall / timing-related tests              | No                          |
| `exram_suite`                 | MMC5 ExRAM behaviour (currently failing)     | No                          |
| `m22chrbankingtest_suite`     | Mapper 22 CHR banking behaviour              | No                          |
| `mmc5test_suite`              | MMC5 functional tests                        | Yes                         |
| `mmc5test_v2_suite`           | MMC5 test set v2                             | Yes                         |
| `nes15_1_0_0_suite`           | `nes15` series tests (NTSC/PAL)              | Yes                         |
| `nrom368_suite`               | NROM-368 mapping tests                       | No                          |
| `other_suite`                 | Misc demos/tests bundled with nes-test-roms  | No                          |
| `pal_apu_tests_suite`         | PAL APU behaviour                            | Yes                         |
| `read_joy3_suite`             | Controller read timing                       | Yes                         |
| `scanline_a1_suite`           | Alternate scanline tests                     | Yes                         |
| `scrolltest_suite`            | Scrolling behaviour                          | Yes                         |
| `volume_tests_suite`          | Volume/mixing behaviour                      | Yes                         |

### Tracking / diagnostic ROM suites (ℹ️)

These suites are baseline-tracking diagnostics against reference output and do not indicate a known failing area by themselves.

| Suite name                        | Notes                                                    | TASVideos accuracy-required |
| --------------------------------- | -------------------------------------------------------- | --------------------------- |
| `nmi_sync_ntsc_mesen_baseline`    | NTSC frame-hash baseline tracking against Mesen2 output (enabled in default runs) | Yes                         |

## Disclaimer

This project is a fan-made, non-commercial emulator intended for educational and preservation purposes. It is not affiliated with, endorsed, or sponsored by Nintendo or any other rights holder. You are solely responsible for complying with local laws and for ensuring that any ROMs or other copyrighted content you use with this emulator are obtained and used legally (for example, from cartridges you personally own).

## Contributions

Feel free to fork the project, open issues, and submit pull requests. Contributions are welcome as we work to improve accuracy and expand the feature set.

## License

Nesium is distributed under the terms of the GNU General Public License, version 3 or (at your option) any later version (GPL‑3.0‑or‑later). See `LICENSE.md` for the full text.

This project also includes Shay Green’s `blip_buf` library (used via the `nesium-blip` crate), which is licensed under the GNU Lesser General Public License v2.1. The relevant license text is included alongside the imported sources in `crates/nesium-blip/csrc/license.md`.

## Libretro bindings

The workspace includes the `libretro-bridge` crate, which automatically generates Rust bindings for the upstream `libretro.h` header via `bindgen`. The build script fetches the most recent header at compile time (with a vendored fallback for offline builds) so Nesium—and any other Rust project—can integrate with the libretro ecosystem as soon as API changes land upstream.

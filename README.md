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

## Disclaimer

This project is a fan-made, non-commercial emulator intended for educational and preservation purposes. It is not affiliated with, endorsed, or sponsored by Nintendo or any other rights holder. You are solely responsible for complying with local laws and for ensuring that any ROMs or other copyrighted content you use with this emulator are obtained and used legally (for example, from cartridges you personally own).

## Contributions:
Feel free to fork the project, open issues, and submit pull requests. Contributions are welcome as we work to improve accuracy and expand the feature set.

## License

Nesium is distributed under the terms of the GNU General Public License, version 3 or (at your option) any later version (GPL‑3.0‑or‑later). See `LICENSE.md` for the full text.

This project also includes Shay Green’s `blip_buf` library (used via the `nesium-blip` crate), which is licensed under the GNU Lesser General Public License v2.1. The relevant license text is included alongside the imported sources in `crates/nesium-blip/csrc/license.md`.

## Libretro bindings
The workspace includes the `libretro-bridge` crate, which automatically generates Rust bindings for the upstream `libretro.h` header via `bindgen`. The build script fetches the most recent header at compile time (with a vendored fallback for offline builds) so Nesium—and any other Rust project—can integrate with the libretro ecosystem as soon as API changes land upstream.

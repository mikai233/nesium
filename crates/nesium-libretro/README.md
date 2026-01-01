# Nesium Libretro Core

This crate provides a fully functional [Libretro](https://www.libretro.com/) core for **Nesium**, allowing you to play NES games in frontends like RetroArch. It wraps the cycle-accurate [`nesium-core`](../nesium-core) engine using the [`libretro-bridge`](../libretro-bridge) API.

## Features

- **Accurate Emulation**: Leverages `nesium-core` for cycle-accurate CPU, PPU, and APU emulation.
- **Audio/Video**: Outputs correct NES aspect ratio and full APU sound mixing.
- **Input**: Supports standard NES controllers for Player 1 and Player 2.
- **Mapper Support**: Compatible with all mappers supported by the main emulator (see [Mapper Support](../../README.md#mapper-support)).

## Building

To build the core dynamic library:

```bash
cargo build -p nesium-libretro --profile release
```

The resulting artifact will be located in `target/release/` with a reliable system-specific extension (e.g., `.dll` on Windows, `.so` on Linux, `.dylib` on macOS).

## Installation

1. Copy the generated library file to your RetroArch `cores/` directory.
2. Launch RetroArch.
3. Select **Load Core** and choose **Nesium Core**.
4. Load your favorite NES ROM.

## Limitations

This core is production-ready for gameplay but currently lacks some advanced Libretro integration features:
- **Save States**: Serialization support is planned but not yet implemented.
- **Cheats**: Libretro cheat code API is not yet wired up.
- **Controller Expansion**: Only standard controllers are currently supported (no Zapper/Four Score yet).

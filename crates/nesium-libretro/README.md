# Nesium Libretro Demo

This crate builds a small showcase libretro core powered by the reusable [`libretro-bridge`](../libretro-bridge) crate. It renders a colourful gradient and plays a simple sine wave so RetroArch users can verify that the bindings work end-to-end.

## Building

```
cargo build -p nesium-libretro --profile release
```

The resulting dynamic library is located under `target/release/` (with the OS-specific extension). Drop it into RetroArch’s `cores/` directory and load it as an “Nesium Demo Core”. The core does not expect real game content; launching it with “No Core Information Available” is fine.

## Caveats

* The demo is intentionally minimal and only exercises video/audio output along with the required lifecycle callbacks.
* Save states, input handling, and serialization APIs are not implemented.

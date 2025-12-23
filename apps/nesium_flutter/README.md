# nesium_flutter

A modern, cross‑platform Flutter frontend for **Nesium**, powered by the Rust core library **`nesium-core`**.

This app focuses on delivering a polished UI/UX while reusing the same emulator logic implemented in Rust. It is intended to be the primary UI client for Nesium across desktop and mobile platforms.

## Highlights

- **Rust-powered core**: Built on top of `nesium-core` for accurate, shared emulator behavior.
- **Modern Flutter UI**: Fast iteration and a responsive, platform-native experience.
- **Cross-platform**: Designed to run on multiple targets (desktop/mobile/web depending on enabled builds).
- **Advanced features**: Support for debugging workflows (e.g., inspection tools, developer utilities) as the project evolves.

## Project structure

- `apps/nesium_flutter/` — Flutter UI application.
- `crates/nesium-flutter/` — **Glue layer** that boots the NES runtime and bridges Flutter ↔ Rust. This crate is built as a native dynamic library that the Flutter app loads.
- `crates/nesium-core/` — Rust emulator core used by this app.

## Architecture overview

At a high level, the Flutter app drives UI and input, and loads a native dynamic library built from `crates/nesium-flutter`. The NES runtime runs in Rust on a dedicated thread:

- **NES core** runs on an **independent thread** (Rust) to keep emulation timing stable and avoid blocking the Flutter UI.
- **Video output** is presented to Flutter via a **Flutter Texture** and composited into the scene.
- **Control / debug messages** flow through **flutter_rust_bridge** (method calls / streams) between Dart and Rust via the glue layer (`crates/nesium-flutter`).
- **Build requirements**: desktop builds require a working **Rust toolchain** in addition to Flutter (because the Rust core is built as a native library).

## Getting started

```bash
cd apps/nesium_flutter
flutter pub get
flutter run
```

> Note: this project embeds the Rust core; make sure you have a Rust toolchain installed (`rustc`/`cargo`) before building.

> Desktop builds may require platform toolchains (Xcode for macOS, MSVC Build Tools for Windows, etc.).

## Notes

- The UI communicates with the Rust core through the `crates/nesium-flutter` dynamic library and its platform bindings.
- ROM loading and file access can be platform-specific; follow platform prompts and permissions.

## License

See the repository root for license information.

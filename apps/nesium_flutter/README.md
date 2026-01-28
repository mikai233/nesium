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

### Video output resizing (filters/upscalers)

The NES core canonical index buffer is fixed at `256x240`. The runtime derives a packed output buffer whose resolution can be changed at runtime. Each frame-ready callback reports the current output `width/height/pitch`, and the platform texture code reallocates as needed and only performs a copy (no per-platform scaling).

- Rust (FRB): `setVideoFilter(...)` (single-select filter; some filters rescale the output)
- Dart MethodChannel: `setPresentBufferSize` (presentation buffer size; required on Android to avoid compositor scaling)

## Getting started

```bash
cd apps/nesium_flutter
flutter pub get
flutter run
```

## Flutter ↔ Rust bindings (FRB codegen)

This project uses **flutter_rust_bridge** to generate Dart ↔ Rust bindings.

- Do **not** edit generated files by hand:
  - Dart: `apps/nesium_flutter/lib/bridge/` (e.g. `frb_generated*.dart`, `api/events.dart`)
  - Rust: `crates/nesium-flutter/src/frb_generated.rs`
- Edit the Rust API sources instead (annotated with `#[frb]`), under `crates/nesium-flutter/src/api/`.
- Then regenerate bindings using the config file `apps/nesium_flutter/flutter_rust_bridge.yaml`:

```bash
cd apps/nesium_flutter
flutter_rust_bridge_codegen generate --config-file flutter_rust_bridge.yaml
```

For iterative development:

```bash
cd apps/nesium_flutter
flutter_rust_bridge_codegen generate --config-file flutter_rust_bridge.yaml --watch
```

## Web (local)

This project uses a Web Worker + `wasm-pack` build of `crates/nesium-wasm`.
To auto-build the WASM output into `web/nes/pkg` before launching Chrome:

```bash
cd apps/nesium_flutter
dart run tool/run_web.dart
```

Options:
- `--force-wasm`: rebuild even if up-to-date
- `--skip-wasm`: skip wasm-pack step

> Note: this project embeds the Rust core; make sure you have a Rust toolchain installed (`rustc`/`cargo`) before building.

> Desktop builds may require platform toolchains (Xcode for macOS, MSVC Build Tools for Windows, etc.).

## Web (CI + GitHub Pages)

- Online demo: https://mikai233.github.io/nesium/
- CI builds Flutter Web and runs `wasm-pack` to generate `web/nes/pkg` in `.github/workflows/flutter.yml`.
- Releases deploy the Web build to GitHub Pages from `.github/workflows/release.yml`.
- `mode=test` in the Release Controller workflow also deploys to GitHub Pages (no GitHub Release is created).
- GitHub Pages deployments run the same icon generation pipeline (Rust `nesium-icon` + `flutter_launcher_icons`) so the web favicon/manifest icons stay in sync with desktop/mobile.

Setup (once, in GitHub repo settings):
- `Settings → Pages → Build and deployment → Source: GitHub Actions`
- Optional: set repository variable `PAGES_BASE_HREF` (e.g. `/nesium/` for project pages, `/` for user pages).

## Notes

- The UI communicates with the Rust core through the `crates/nesium-flutter` dynamic library and its platform bindings.
- ROM loading and file access can be platform-specific; follow platform prompts and permissions.

## License

See the repository root for license information.

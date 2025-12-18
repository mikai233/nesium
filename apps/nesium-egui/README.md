# Nesium egui Icons (build-time)

Icons are generated at **build time** via `build.rs`:

- `nesium-icon` (Skia) is only a **build-dependency**; runtime has no `skia-safe`.
- Outputs:
  - `$OUT_DIR/icon_rgba.bin` and `$OUT_DIR/egui_icon.rs` (embedded by `include!`).
  - `$OUT_DIR/app.ico` (Windows, multi-size, embedded with `winres` on Windows only).
- `target/generated/icons/com.mikai233.nesium.png` (Linux/Wayland desktop icon, workspace target).
- `target/generated/com.mikai233.nesium.desktop` (Linux desktop entry sample, workspace target).
  - `target/generated/nesium.icns` (macOS, best-effort pure-Rust writer).

The app_id/desktop/icon name is unified as `com.mikai233.nesium`.

## Linux (Wayland/X11)
- Generated PNG: `target/generated/icons/com.mikai233.nesium.png` (workspace root).
- Desktop entry: `target/generated/com.mikai233.nesium.desktop` (Icon/com.mikai233.nesium matches `with_app_id("com.mikai233.nesium")`).
- Install locally (XDG):
  - `mkdir -p ~/.local/share/icons/hicolor/256x256/apps`
  - `cp target/generated/icons/com.mikai233.nesium.png ~/.local/share/icons/hicolor/256x256/apps/`
  - `mkdir -p ~/.local/share/applications`
  - `cp target/generated/com.mikai233.nesium.desktop ~/.local/share/applications/`
  - Refresh cache if needed: `gtk-update-icon-cache ~/.local/share/icons/hicolor`
- Wayland needs the app_id (`com.mikai233.nesium`) to match the desktop file name and `Icon=` field.

## Windows
- Build script renders multi-size ICO and embeds it via `winres` (Windows-only).
- No static icon files are kept in git; rebuild regenerates everything.

## macOS
- Best-effort `target/generated/nesium.icns` built from the rendered PNG (pure Rust).
- To bundle:
  - Place `nesium.icns` into `Nesium.app/Contents/Resources/`.
- In `Info.plist`, set `CFBundleIconFile` to `nesium` (no extension) and `CFBundleIdentifier` to `com.mikai233.nesium`.
- If you prefer external tooling (e.g., `iconutil`), run it manually against the generated PNG; the build will not fail if icns generation is skipped.

## Notes
- Generated artifacts live under `target/generated` or `$OUT_DIR` and are git-ignored.
- Change detection: `build.rs` reruns when `crates/nesium-icon/src/**` or itself changes.

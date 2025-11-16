# libretro-bridge

`libretro-bridge` provides low-level Rust bindings for the [libretro](https://www.libretro.com/) core API plus a thin runtime that makes implementing `LibretroCore` in Rust ergonomic. Bindings are generated with [`bindgen`](https://github.com/rust-lang/rust-bindgen) at compile time so they always match the upstream `libretro.h`, allowing any project to keep pace with RetroArch and other libretro front-ends without hand-written C shims.

## How it works

1. During the build the crate downloads the latest `libretro.h` from `https://github.com/libretro/libretro-common`.
2. The header is fed into `bindgen`, which emits Rust `repr(C)` types, constants, and function pointer definitions.
3. The generated code is exposed through the crate as a safe-to-include module named `raw`, while the crate root re-exports every symbol for convenience.

Because the bindings are produced on each build they automatically reflect upstream API additions or changes without hand-written glue code.

## Download strategy and configuration

The build script tries to fetch the canonical header every time the crate is compiled. Two environment variables let you control that behaviour. Offline mode is the default for reproducible builds; explicitly opt-in if you want the header to be refreshed automatically.

| Variable | Meaning |
| --- | --- |
| `LIBRETRO_BRIDGE_OFFLINE=1` | Force offline mode (default). Set to `0` to allow downloads. |
| `LIBRETRO_BRIDGE_FETCH=1` | Opt into downloading the header even if `LIBRETRO_BRIDGE_OFFLINE` is unset. |
| `LIBRETRO_BRIDGE_HEADER_URL=<url>` | Override the download location. This is handy when testing against a fork of the libretro API. |

Whenever the download fails, the build script logs a warning and falls back to the vendored header so builds remain reproducible even without Internet access.

## Example

```rust
use std::ffi::c_void;
use libretro_bridge::{
    raw::RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE,
    retro_environment_t,
};

unsafe extern "C" fn init_environment(callback: retro_environment_t) {
    if let Some(callback) = callback {
        let mut changed = false;
        callback(
            RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE,
            (&mut changed as *mut bool).cast::<c_void>(),
        );
    }
}
```

## Requirements

Generating bindings requires `libclang` to be available on your system (a standard requirement for `bindgen`). On Linux and macOS this typically means having LLVM installed, while on Windows the Visual Studio Build Tools bundle includes a compatible clang.

## Updating the bundled header

Even though the build script downloads `libretro.h`, the repository also keeps a fallback copy under `vendor/` so offline builds work out of the box. Whenever you intentionally update the header you should:

1. Delete or overwrite `vendor/libretro.h` with the new upstream version.
2. Commit the change so other contributors can build without a network connection.

This workflow ensures contributors immediately notice upstream libretro changes, while preserving deterministic builds.

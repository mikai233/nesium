// NOTE: We rely on `std::env::var("CARGO_CFG_TARGET_ARCH")` in `build.rs` to skip C++ compilation
// when targeting WASM. However, in a workspace environment (like with `nesium-flutter`), Cargo features
// can be unified, causing `*-cpp` features to be enabled even for the WASM target associated with
// `nesium-wasm`.
//
// To prevent linker errors (Rust trying to call C++ functions that weren't compiled), we must
// explicitly guard all C++ module inclusions and extern usages with `not(target_arch = "wasm32")`.

#[cfg(all(feature = "hqx-cpp", not(target_arch = "wasm32")))]
pub mod hqx;

#[cfg(all(feature = "ntsc-cpp", not(target_arch = "wasm32")))]
pub mod ntsc;

pub mod sai;

pub mod lcd_grid;

pub mod scanline;

#[cfg(all(feature = "ntsc-bisqwit-cpp", not(target_arch = "wasm32")))]
pub mod ntsc_bisqwit;

pub mod filters;

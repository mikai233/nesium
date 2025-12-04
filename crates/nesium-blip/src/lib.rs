#![deny(unsafe_op_in_unsafe_fn)]

//! Bindings and wrappers for Shay Green's blip_buf (1.1.0).
//!
//! Pick the implementation by module path:
//! - `nesium_blip::c_impl::BlipBuf` — calls the vendored C++ code via bindgen (feature: `c-impl`).
//! - `nesium_blip::rust_impl::BlipBuf` — pure Rust port with matching behavior (feature: `rust-impl`).
//! - `nesium_blip::BlipBuf` — convenience alias: uses C impl if enabled, otherwise Rust impl.

#[cfg(feature = "c-impl")]
pub mod c_impl;
#[cfg(feature = "rust-impl")]
pub mod rust_impl;

// Default alias: prefer C impl when both are enabled.
#[cfg(feature = "c-impl")]
pub use c_impl::BlipBuf;
#[cfg(all(not(feature = "c-impl"), feature = "rust-impl"))]
pub use rust_impl::BlipBuf;

#[cfg(all(test, feature = "c-impl", feature = "rust-impl"))]
mod tests;

#![deny(unsafe_op_in_unsafe_fn)]

//! Bindings and wrappers for Shay Green's blip_buf (1.1.0).
//!
//! Pick the implementation by module path:
//! - `nesium_blip::c_impl::BlipBuf` — calls the vendored C++ code via bindgen.
//! - `nesium_blip::rust_impl::BlipBuf` — pure Rust port with matching behavior.

pub mod c_impl;
pub mod rust_impl;

#[cfg(test)]
mod tests;

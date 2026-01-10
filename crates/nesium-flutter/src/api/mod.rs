pub mod emulation;
pub mod events;
pub mod input;
pub mod load_rom;
pub mod netplay;
pub mod palette;
pub mod pause;
pub mod server;
pub mod simple;

#[cfg(all(
    not(target_os = "android"),
    not(target_os = "ios"),
    not(target_arch = "wasm32")
))]
pub mod gamepad;

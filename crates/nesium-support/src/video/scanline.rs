#[cfg(all(feature = "scanline", not(target_arch = "wasm32")))]
unsafe extern "C" {
    fn nesium_scanline_apply_argb8888(
        buffer: *mut u32,
        width: u32,
        height: u32,
        brightness: u8,
        scale: u8,
    );
}

#[cfg(all(feature = "scanline", not(target_arch = "wasm32")))]
pub fn scanline_apply_argb8888(
    width: usize,
    height: usize,
    buffer: &mut [u32],
    brightness: u8,
    scale: u8,
) {
    if width == 0 || height == 0 {
        return;
    }
    debug_assert!(buffer.len() >= width * height);
    unsafe {
        nesium_scanline_apply_argb8888(
            buffer.as_mut_ptr(),
            width as u32,
            height as u32,
            brightness,
            scale,
        );
    }
}

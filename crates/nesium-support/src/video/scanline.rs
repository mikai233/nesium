#[inline(always)]
fn apply_scanline_effect(argb: u32, brightness: u8) -> u32 {
    let r = (((argb & 0x00FF0000) >> 16) * brightness as u32 / 255) as u8;
    let g = (((argb & 0x0000FF00) >> 8) * brightness as u32 / 255) as u8;
    let b = ((argb & 0x000000FF) * brightness as u32 / 255) as u8;
    0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

pub fn scanline_apply_argb8888(
    width: usize,
    height: usize,
    buffer: &mut [u32],
    brightness: u8,
    scale: u8,
) {
    if width == 0 || height == 0 || brightness == 255 {
        return;
    }
    debug_assert!(buffer.len() >= width * height);

    let scale = scale.max(2) as usize;
    for rows in buffer.chunks_exact_mut(width * scale) {
        let scanline_row = &mut rows[width * (scale - 1)..width * scale];
        for pixel in scanline_row.iter_mut() {
            *pixel = apply_scanline_effect(*pixel, brightness);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[cfg(all(feature = "scanline-cpp", not(target_arch = "wasm32")))]
    unsafe extern "C" {
        fn nesium_scanline_apply_argb8888(
            buffer: *mut u32,
            width: u32,
            height: u32,
            brightness: u8,
            scale: u8,
        );
    }

    proptest! {
        #[test]
        fn test_scanline_matches_cpp(
            width in 1usize..256usize,
            height in 2usize..256usize, // C++ loop needs at least scale lines
            brightness in 0u8..254u8,
            scale in 2u8..8u8,
        ) {
            let mut buffer_rust = vec![0xFF112233u32; width * height];
            #[cfg(all(feature = "scanline-cpp", not(target_arch = "wasm32")))]
            let mut buffer_cpp = vec![0xFF112233u32; width * height];

            // Run Rust version
            scanline_apply_argb8888(width, height, &mut buffer_rust, brightness, scale);

            // Run C++ version
            #[cfg(all(feature = "scanline-cpp", not(target_arch = "wasm32")))]
            unsafe {
                nesium_scanline_apply_argb8888(
                    buffer_cpp.as_mut_ptr(),
                    width as u32,
                    height as u32,
                    brightness,
                    scale,
                );
            }

            #[cfg(all(feature = "scanline-cpp", not(target_arch = "wasm32")))]
            assert_eq!(buffer_rust, buffer_cpp);
        }
    }
}

#[inline(always)]
fn apply_brightness(argb: u32, brightness: u8) -> u32 {
    let r = (((argb & 0x00FF0000) >> 16) * brightness as u32 / 255) as u8;
    let g = (((argb & 0x0000FF00) >> 8) * brightness as u32 / 255) as u8;
    let b = ((argb & 0x000000FF) * brightness as u32 / 255) as u8;
    0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

pub fn lcd_grid_2x_argb8888(
    width: usize,
    height: usize,
    src: &[u32],
    src_stride: usize,
    dst: &mut [u32],
    dst_stride: usize,
    top_left: u8,
    top_right: u8,
    bottom_left: u8,
    bottom_right: u8,
) {
    if width == 0 || height == 0 {
        return;
    }
    debug_assert!(src_stride >= width);
    debug_assert!(dst_stride >= width * 2);
    debug_assert!(src.len() >= src_stride * height);
    debug_assert!(dst.len() >= dst_stride * (height * 2));

    for (y, rows) in dst
        .chunks_exact_mut(dst_stride * 2)
        .take(height)
        .enumerate()
    {
        let (dst_row1, dst_row2) = rows.split_at_mut(dst_stride);
        let src_row = &src[y * src_stride..y * src_stride + width];

        for (x, &c) in src_row.iter().enumerate() {
            dst_row1[x * 2] = apply_brightness(c, top_left);
            dst_row1[x * 2 + 1] = apply_brightness(c, top_right);
            dst_row2[x * 2] = apply_brightness(c, bottom_left);
            dst_row2[x * 2 + 1] = apply_brightness(c, bottom_right);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[cfg(all(feature = "lcd-grid-cpp", not(target_arch = "wasm32")))]
    unsafe extern "C" {
        fn nesium_lcd_grid_2x_argb8888(
            src: *const u32,
            width: u32,
            height: u32,
            src_stride: u32,
            dst: *mut u32,
            dst_stride: u32,
            top_left: u8,
            top_right: u8,
            bottom_left: u8,
            bottom_right: u8,
        );
    }

    proptest! {
        #[test]
        fn test_lcd_grid_matches_cpp(
            width in 1usize..32usize,
            height in 1usize..32usize,
            top_left in 0u8..255u8,
            top_right in 0u8..255u8,
            bottom_left in 0u8..255u8,
            bottom_right in 0u8..255u8,
        ) {
            let src_stride = width;
            let dst_stride = width * 2;
            let src = vec![0xFF112233u32; src_stride * height]; // Simple filled buffer
            let mut dst_rust = vec![0u32; dst_stride * height * 2];
            #[cfg(all(feature = "lcd-grid-cpp", not(target_arch = "wasm32")))]
            let mut dst_cpp = vec![0u32; dst_stride * height * 2];

            // Run Rust version
            lcd_grid_2x_argb8888(
                width, height, &src, src_stride, &mut dst_rust, dst_stride,
                top_left, top_right, bottom_left, bottom_right
            );

            // Run C++ version
            #[cfg(all(feature = "lcd-grid-cpp", not(target_arch = "wasm32")))]
            unsafe {
                nesium_lcd_grid_2x_argb8888(
                    src.as_ptr(),
                    width as u32,
                    height as u32,
                    src_stride as u32,
                    dst_cpp.as_mut_ptr(),
                    dst_stride as u32,
                    top_left,
                    top_right,
                    bottom_left,
                    bottom_right,
                );
            }

            #[cfg(all(feature = "lcd-grid-cpp", not(target_arch = "wasm32")))]
            assert_eq!(dst_rust, dst_cpp);
        }
    }
}

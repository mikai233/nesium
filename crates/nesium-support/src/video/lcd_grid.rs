#[cfg(all(feature = "lcd-grid", not(target_arch = "wasm32")))]
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

#[cfg(all(feature = "lcd-grid", not(target_arch = "wasm32")))]
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
    unsafe {
        nesium_lcd_grid_2x_argb8888(
            src.as_ptr(),
            width as u32,
            height as u32,
            src_stride as u32,
            dst.as_mut_ptr(),
            dst_stride as u32,
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        );
    }
}

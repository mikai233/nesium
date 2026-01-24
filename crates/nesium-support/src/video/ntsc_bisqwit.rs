#[cfg(all(feature = "ntsc-bisqwit", not(target_arch = "wasm32")))]
unsafe extern "C" {
    fn nesium_ntsc_bisqwit_apply_argb8888(
        ppu: *const u16,
        ppu_width: i32,
        ppu_height: i32,
        dst: *mut u32,
        scale: i32,
        brightness: f64,
        contrast: f64,
        hue: f64,
        saturation: f64,
        y_filter_length: f64,
        i_filter_length: f64,
        q_filter_length: f64,
        phase_offset: i32,
    );
}

#[cfg(all(feature = "ntsc-bisqwit", not(target_arch = "wasm32")))]
pub fn ntsc_bisqwit_apply_argb8888(
    ppu: &[u16],
    ppu_width: usize,
    ppu_height: usize,
    dst: &mut [u32],
    scale: usize,
    brightness: f64,
    contrast: f64,
    hue: f64,
    saturation: f64,
    y_filter_length: f64,
    i_filter_length: f64,
    q_filter_length: f64,
    phase_offset: i32,
) {
    if ppu_width == 0 || ppu_height == 0 {
        return;
    }
    if !(scale == 2 || scale == 4 || scale == 8) {
        return;
    }

    let expected_ppu = match ppu_width.checked_mul(ppu_height) {
        Some(v) => v,
        None => return,
    };
    if ppu.len() != expected_ppu {
        return;
    }

    let out_w = ppu_width.saturating_mul(scale);
    let out_h = ppu_height.saturating_mul(scale);
    let expected_dst = match out_w.checked_mul(out_h) {
        Some(v) => v,
        None => return,
    };
    if dst.len() != expected_dst {
        return;
    }

    let w = match i32::try_from(ppu_width) {
        Ok(v) => v,
        Err(_) => return,
    };
    let h = match i32::try_from(ppu_height) {
        Ok(v) => v,
        Err(_) => return,
    };
    let s = match i32::try_from(scale) {
        Ok(v) => v,
        Err(_) => return,
    };

    unsafe {
        nesium_ntsc_bisqwit_apply_argb8888(
            ppu.as_ptr(),
            w,
            h,
            dst.as_mut_ptr(),
            s,
            brightness,
            contrast,
            hue,
            saturation,
            y_filter_length,
            i_filter_length,
            q_filter_length,
            phase_offset,
        );
    }
}

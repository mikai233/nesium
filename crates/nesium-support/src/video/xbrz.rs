use std::os::raw::c_int;

#[cfg(all(feature = "xbrz", not(target_arch = "wasm32")))]
unsafe extern "C" {
    fn nesium_xbrz_scale_argb8888(
        scale: usize,
        src: *const u32,
        src_width: c_int,
        src_height: c_int,
        dst: *mut u32,
    );
}

#[cfg(all(feature = "xbrz", not(target_arch = "wasm32")))]
pub fn xbrz_scale_argb8888(
    scale: usize,
    src: &[u32],
    src_width: usize,
    src_height: usize,
    dst: &mut [u32],
) {
    if src_width == 0 || src_height == 0 || scale < 2 || scale > 6 {
        return;
    }

    let expected_src = match src_width.checked_mul(src_height) {
        Some(v) => v,
        None => return,
    };
    if src.len() != expected_src {
        return;
    }

    let dst_width = src_width.saturating_mul(scale);
    let dst_height = src_height.saturating_mul(scale);
    let expected_dst = match dst_width.checked_mul(dst_height) {
        Some(v) => v,
        None => return,
    };
    if dst.len() != expected_dst {
        return;
    }

    let w = match c_int::try_from(src_width) {
        Ok(v) => v,
        Err(_) => return,
    };
    let h = match c_int::try_from(src_height) {
        Ok(v) => v,
        Err(_) => return,
    };

    unsafe {
        nesium_xbrz_scale_argb8888(scale, src.as_ptr(), w, h, dst.as_mut_ptr());
    }
}

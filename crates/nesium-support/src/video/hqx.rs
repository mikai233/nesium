use std::sync::OnceLock;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum HqxError {
    #[error("width/height overflow")]
    DimensionOverflow,

    #[error("invalid input size (expected {expected} u32 pixels, got {actual})")]
    InvalidInputSize { expected: usize, actual: usize },

    #[error("invalid output size (expected {expected} u32 pixels, got {actual})")]
    InvalidOutputSize { expected: usize, actual: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum HqxScale {
    X2 = 2,
    X3 = 3,
    X4 = 4,
}

static HQX_INIT: OnceLock<()> = OnceLock::new();

fn ensure_init() {
    HQX_INIT.get_or_init(|| unsafe {
        hqxInit();
    });
}

/// Upscales an ARGB8888 framebuffer using HQX (hq2x/hq3x/hq4x).
///
/// Notes
/// - Input/output buffers are `u32` pixels in **ARGB8888** order (`0xAARRGGBB`).
/// - Output dimensions are `width * scale` × `height * scale`.
pub fn hqx_scale_argb8888(
    scale: HqxScale,
    src: &[u32],
    width: usize,
    height: usize,
    dst: &mut [u32],
) -> Result<(), HqxError> {
    let expected_in = width
        .checked_mul(height)
        .ok_or(HqxError::DimensionOverflow)?;
    if src.len() != expected_in {
        return Err(HqxError::InvalidInputSize {
            expected: expected_in,
            actual: src.len(),
        });
    }

    let s = scale as usize;
    let expected_out = expected_in
        .checked_mul(s)
        .and_then(|v| v.checked_mul(s))
        .ok_or(HqxError::DimensionOverflow)?;
    if dst.len() != expected_out {
        return Err(HqxError::InvalidOutputSize {
            expected: expected_out,
            actual: dst.len(),
        });
    }

    ensure_init();

    let w = width as i32;
    let h = height as i32;

    unsafe {
        match scale {
            HqxScale::X2 => hq2x_32(src.as_ptr(), dst.as_mut_ptr(), w, h),
            HqxScale::X3 => hq3x_32(src.as_ptr(), dst.as_mut_ptr(), w, h),
            HqxScale::X4 => hq4x_32(src.as_ptr(), dst.as_mut_ptr(), w, h),
        }
    }

    Ok(())
}

unsafe extern "C" {
    fn hqxInit();
    fn hq2x_32(src: *const u32, dest: *mut u32, width: i32, height: i32);
    fn hq3x_32(src: *const u32, dest: *mut u32, width: i32, height: i32);
    fn hq4x_32(src: *const u32, dest: *mut u32, width: i32, height: i32);
}

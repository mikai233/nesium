pub mod hq2x;
pub mod hq3x;
pub mod hq4x;
pub mod hqx_common;

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

    hqx_common::ensure_init();

    match scale {
        HqxScale::X2 => hq2x::hq2x_32_rb(src, width, dst, width * 2, width, height),
        HqxScale::X3 => hq3x::hq3x_32_rb(src, width, dst, width * 3, width, height),
        HqxScale::X4 => hq4x::hq4x_32_rb(src, width, dst, width * 4, width, height),
    }

    Ok(())
}

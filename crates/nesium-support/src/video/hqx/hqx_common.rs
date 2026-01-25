use std::sync::OnceLock;

const Y_MASK: u32 = 0x00FF_0000;
const U_MASK: u32 = 0x0000_FF00;
const V_MASK: u32 = 0x0000_00FF;
const TR_Y: u32 = 0x0030_0000;
const TR_U: u32 = 0x0000_0700;
const TR_V: u32 = 0x0000_0006;

static RGB_TO_YUV: OnceLock<Box<[u32; 16777216]>> = OnceLock::new();

pub fn ensure_init() {
    RGB_TO_YUV.get_or_init(|| {
        let mut table: Box<[u32; 16777216]> =
            vec![0u32; 16777216].into_boxed_slice().try_into().unwrap();
        for (c, item) in table.iter_mut().enumerate() {
            let r = ((c & 0xFF0000) >> 16) as f64;
            let g = ((c & 0x00FF00) >> 8) as f64;
            let b = (c & 0x0000FF) as f64;
            let y = (0.299 * r + 0.587 * g + 0.114 * b) as u32;
            let u = ((-0.169 * r - 0.331 * g + 0.5 * b) + 128.0) as u32;
            let v = ((0.5 * r - 0.419 * g - 0.081 * b) + 128.0) as u32;
            *item = (y << 16) | (u << 8) | v;
        }
        table
    });
}

#[inline(always)]
pub fn rgb_to_yuv(argb: u32) -> u32 {
    let table = RGB_TO_YUV.get().expect("hqx not initialized");
    table[(argb & 0x00FF_FFFF) as usize]
}

#[inline(always)]
pub fn yuv_diff(yuv1: u32, yuv2: u32) -> bool {
    ((yuv1 & Y_MASK).abs_diff(yuv2 & Y_MASK) > TR_Y)
        || ((yuv1 & U_MASK).abs_diff(yuv2 & U_MASK) > TR_U)
        || ((yuv1 & V_MASK).abs_diff(yuv2 & V_MASK) > TR_V)
}

#[inline(always)]
pub fn diff(c1: u32, c2: u32) -> bool {
    yuv_diff(rgb_to_yuv(c1), rgb_to_yuv(c2))
}

const MASK_2: u32 = 0x0000_FF00;
const MASK_13: u32 = 0x00FF_00FF;
const MASK_ALPHA: u32 = 0xFF00_0000;

#[inline(always)]
fn interpolate_2(c1: u32, w1: u32, c2: u32, w2: u32, s: u32) -> u32 {
    if c1 == c2 {
        return c1;
    }
    let a = ((((c1 & MASK_ALPHA) >> 24) * w1 + ((c2 & MASK_ALPHA) >> 24) * w2) << (24 - s))
        & MASK_ALPHA;
    let rb = ((((c1 & MASK_13) * w1 + (c2 & MASK_13) * w2) >> s) & MASK_13);
    let g = ((((c1 & MASK_2) * w1 + (c2 & MASK_2) * w2) >> s) & MASK_2);
    a | rb | g
}

#[inline(always)]
fn interpolate_3(c1: u32, w1: u32, c2: u32, w2: u32, c3: u32, w3: u32, s: u32) -> u32 {
    let a = ((((c1 & MASK_ALPHA) >> 24) * w1
        + ((c2 & MASK_ALPHA) >> 24) * w2
        + ((c3 & MASK_ALPHA) >> 24) * w3)
        << (24 - s))
        & MASK_ALPHA;
    let rb = ((((c1 & MASK_13) * w1 + (c2 & MASK_13) * w2 + (c3 & MASK_13) * w3) >> s) & MASK_13);
    let g = ((((c1 & MASK_2) * w1 + (c2 & MASK_2) * w2 + (c3 & MASK_2) * w3) >> s) & MASK_2);
    a | rb | g
}

pub fn interp1(c1: u32, c2: u32) -> u32 {
    interpolate_2(c1, 3, c2, 1, 2)
}
pub fn interp2(c1: u32, c2: u32, c3: u32) -> u32 {
    interpolate_3(c1, 2, c2, 1, c3, 1, 2)
}
pub fn interp3(c1: u32, c2: u32) -> u32 {
    interpolate_2(c1, 7, c2, 1, 3)
}
pub fn interp4(c1: u32, c2: u32, c3: u32) -> u32 {
    interpolate_3(c1, 2, c2, 7, c3, 7, 4)
}
pub fn interp5(c1: u32, c2: u32) -> u32 {
    interpolate_2(c1, 1, c2, 1, 1)
}
pub fn interp6(c1: u32, c2: u32, c3: u32) -> u32 {
    interpolate_3(c1, 5, c2, 2, c3, 1, 3)
}
pub fn interp7(c1: u32, c2: u32, c3: u32) -> u32 {
    interpolate_3(c1, 6, c2, 1, c3, 1, 3)
}
pub fn interp8(c1: u32, c2: u32) -> u32 {
    interpolate_2(c1, 5, c2, 3, 3)
}
pub fn interp9(c1: u32, c2: u32, c3: u32) -> u32 {
    interpolate_3(c1, 2, c2, 3, c3, 3, 3)
}
pub fn interp10(c1: u32, c2: u32, c3: u32) -> u32 {
    interpolate_3(c1, 14, c2, 1, c3, 1, 4)
}

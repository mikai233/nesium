#![cfg(feature = "sai-cpp")]

use nesium_support::video::sai::{
    scale_2xsai_xrgb8888, scale_2xsai_xrgb8888_cpp, scale_super_2xsai_xrgb8888,
    scale_super_2xsai_xrgb8888_cpp, scale_supereagle_xrgb8888, scale_supereagle_xrgb8888_cpp,
};
use proptest::prelude::*;

#[inline]
fn opaque_xrgb8888(px: u32) -> u32 {
    0xFF00_0000 | (px & 0x00FF_FFFF)
}

#[derive(Clone, Debug)]
struct Case {
    width: usize,
    height: usize,
    src_stride: usize,
    dst_stride: usize,
    src: Vec<u32>,
}

fn case_strategy() -> impl Strategy<Value = Case> {
    // Moderate sizes to keep runtime reasonable while still stressing boundaries.
    (1usize..=64, 1usize..=48, 0usize..=16, 0usize..=24).prop_flat_map(
        |(width, height, src_pad, dst_pad)| {
            let src_stride = width + src_pad;
            let dst_stride = (width * 2) + dst_pad;
            let len = src_stride * height;

            #[derive(Clone, Debug)]
            enum Src {
                Random(Vec<u32>),
                Checkerboard {
                    a: u32,
                    b: u32,
                },
                HGradient {
                    seed: u32,
                },
                VGradient {
                    seed: u32,
                },
                Stripes {
                    a: u32,
                    b: u32,
                    period: u8,
                },
                HotPixel {
                    bg: u32,
                    fg: u32,
                    x: usize,
                    y: usize,
                },
            }

            let random_src = prop::collection::vec(any::<u32>(), len..=len).prop_map(Src::Random);
            let checkerboard =
                (any::<u32>(), any::<u32>()).prop_map(|(a, b)| Src::Checkerboard { a, b });
            let h_gradient = any::<u32>().prop_map(|seed| Src::HGradient { seed });
            let v_gradient = any::<u32>().prop_map(|seed| Src::VGradient { seed });
            let stripes = (any::<u32>(), any::<u32>(), 1u8..=8u8)
                .prop_map(|(a, b, period)| Src::Stripes { a, b, period });
            let hot_pixel = (any::<u32>(), any::<u32>(), 0usize..width, 0usize..height)
                .prop_map(|(bg, fg, x, y)| Src::HotPixel { bg, fg, x, y });

            prop_oneof![
                6 => random_src,
                1 => checkerboard,
                1 => h_gradient,
                1 => v_gradient,
                1 => stripes,
                1 => hot_pixel,
            ]
            .prop_map(move |src_kind| {
                let mut src = vec![0u32; len];

                match src_kind {
                    Src::Random(mut v) => {
                        for px in &mut v {
                            *px = opaque_xrgb8888(*px);
                        }
                        src = v;
                    }
                    Src::Checkerboard { a, b } => {
                        let a = opaque_xrgb8888(a);
                        let b = opaque_xrgb8888(b);
                        for y in 0..height {
                            for x in 0..width {
                                src[y * src_stride + x] = if ((x ^ y) & 1) == 0 { a } else { b };
                            }
                        }
                    }
                    Src::HGradient { seed } => {
                        let base = opaque_xrgb8888(seed);
                        let width_denom = (width.saturating_sub(1).max(1)) as u32;
                        for y in 0..height {
                            for x in 0..width {
                                let t = (x as u32 * 255) / width_denom;
                                let r = (((base >> 16) & 0xFF) ^ t) & 0xFF;
                                let g = (((base >> 8) & 0xFF).wrapping_add(t)) & 0xFF;
                                let b = ((base & 0xFF) ^ (t.rotate_left(3))) & 0xFF;
                                src[y * src_stride + x] = 0xFF00_0000 | (r << 16) | (g << 8) | b;
                            }
                        }
                    }
                    Src::VGradient { seed } => {
                        let base = opaque_xrgb8888(seed);
                        let height_denom = (height.saturating_sub(1).max(1)) as u32;
                        for y in 0..height {
                            let t = (y as u32 * 255) / height_denom;
                            for x in 0..width {
                                let r = (((base >> 16) & 0xFF).wrapping_add(t)) & 0xFF;
                                let g = (((base >> 8) & 0xFF) ^ (t.rotate_left(2))) & 0xFF;
                                let b = ((base & 0xFF).wrapping_add(t.wrapping_mul(3))) & 0xFF;
                                src[y * src_stride + x] = 0xFF00_0000 | (r << 16) | (g << 8) | b;
                            }
                        }
                    }
                    Src::Stripes { a, b, period } => {
                        let a = opaque_xrgb8888(a);
                        let b = opaque_xrgb8888(b);
                        let period = period.max(1) as usize;
                        for y in 0..height {
                            for x in 0..width {
                                src[y * src_stride + x] =
                                    if ((x / period) & 1) == 0 { a } else { b };
                            }
                        }
                    }
                    Src::HotPixel { bg, fg, x, y } => {
                        let bg = opaque_xrgb8888(bg);
                        let fg = opaque_xrgb8888(fg);
                        for y0 in 0..height {
                            for x0 in 0..width {
                                src[y0 * src_stride + x0] = bg;
                            }
                        }
                        src[y * src_stride + x] = fg;
                    }
                }

                // Ensure any padding is initialized (makes debugging easier; should be ignored).
                for y in 0..height {
                    for x in width..src_stride {
                        src[y * src_stride + x] = 0xFF00_0000;
                    }
                }

                Case {
                    width,
                    height,
                    src_stride,
                    dst_stride,
                    src,
                }
            })
        },
    )
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 128, .. ProptestConfig::default() })]

    #[test]
    fn sai_cpp_matches_rust(case in case_strategy()) {
        let w = case.width;
        let h = case.height;
        let src_stride = case.src_stride;
        let dst_stride = case.dst_stride;

        let mut rust_out = vec![0u32; dst_stride * (h * 2)];
        let mut cpp_out = vec![0u32; dst_stride * (h * 2)];

        // 2xSaI
        scale_2xsai_xrgb8888(w, h, &case.src, src_stride, &mut rust_out, dst_stride);
        scale_2xsai_xrgb8888_cpp(w, h, &case.src, src_stride, &mut cpp_out, dst_stride);
        prop_assert_eq!(rust_out.as_slice(), cpp_out.as_slice());

        // Super 2xSaI
        rust_out.fill(0);
        cpp_out.fill(0);
        scale_super_2xsai_xrgb8888(w, h, &case.src, src_stride, &mut rust_out, dst_stride);
        scale_super_2xsai_xrgb8888_cpp(w, h, &case.src, src_stride, &mut cpp_out, dst_stride);
        prop_assert_eq!(rust_out.as_slice(), cpp_out.as_slice());

        // SuperEagle
        rust_out.fill(0);
        cpp_out.fill(0);
        scale_supereagle_xrgb8888(w, h, &case.src, src_stride, &mut rust_out, dst_stride);
        scale_supereagle_xrgb8888_cpp(w, h, &case.src, src_stride, &mut cpp_out, dst_stride);
        prop_assert_eq!(rust_out.as_slice(), cpp_out.as_slice());
    }
}

#[test]
fn sai_cpp_matches_rust_corner_cases() {
    // Deterministic corner cases (useful when debugging failures).
    let cases = [
        (1usize, 1usize, 1usize, 2usize),
        (2, 2, 3, 6),
        (7, 5, 7, 14),
        (13, 9, 20, 28),
    ];
    for (w, h, src_stride, dst_stride) in cases {
        let mut src = vec![0u32; src_stride * h];
        for (i, px) in src.iter_mut().enumerate() {
            let rgb = (i as u32).wrapping_mul(0x9E37_79B1) & 0x00FF_FFFF;
            *px = 0xFF00_0000 | rgb;
        }

        let mut rust_out = vec![0u32; dst_stride * (h * 2)];
        let mut cpp_out = vec![0u32; dst_stride * (h * 2)];

        scale_2xsai_xrgb8888(w, h, &src, src_stride, &mut rust_out, dst_stride);
        scale_2xsai_xrgb8888_cpp(w, h, &src, src_stride, &mut cpp_out, dst_stride);
        assert_eq!(rust_out.as_slice(), cpp_out.as_slice());

        rust_out.fill(0);
        cpp_out.fill(0);
        scale_super_2xsai_xrgb8888(w, h, &src, src_stride, &mut rust_out, dst_stride);
        scale_super_2xsai_xrgb8888_cpp(w, h, &src, src_stride, &mut cpp_out, dst_stride);
        assert_eq!(rust_out.as_slice(), cpp_out.as_slice());

        rust_out.fill(0);
        cpp_out.fill(0);
        scale_supereagle_xrgb8888(w, h, &src, src_stride, &mut rust_out, dst_stride);
        scale_supereagle_xrgb8888_cpp(w, h, &src, src_stride, &mut cpp_out, dst_stride);
        assert_eq!(rust_out.as_slice(), cpp_out.as_slice());
    }
}

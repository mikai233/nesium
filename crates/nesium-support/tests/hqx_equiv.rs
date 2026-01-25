#![cfg(all(feature = "hqx", feature = "hqx-cpp"))]

use nesium_support::video::hqx::{HqxScale, hqx_scale_argb8888};
use proptest::prelude::*;

unsafe extern "C" {
    fn hqxInit();
    fn hq2x_32(src: *const u32, dest: *mut u32, width: i32, height: i32);
    fn hq3x_32(src: *const u32, dest: *mut u32, width: i32, height: i32);
    fn hq4x_32(src: *const u32, dest: *mut u32, width: i32, height: i32);
}

fn cpp_hqx_scale_argb8888(
    scale: HqxScale,
    src: &[u32],
    width: usize,
    height: usize,
    dst: &mut [u32],
) {
    unsafe {
        hqxInit();
        match scale {
            HqxScale::X2 => hq2x_32(src.as_ptr(), dst.as_mut_ptr(), width as i32, height as i32),
            HqxScale::X3 => hq3x_32(src.as_ptr(), dst.as_mut_ptr(), width as i32, height as i32),
            HqxScale::X4 => hq4x_32(src.as_ptr(), dst.as_mut_ptr(), width as i32, height as i32),
        }
    }
}

proptest! {
    #[test]
    fn hq2x_is_bit_identical(
        src in prop::collection::vec(any::<u32>(), 10 * 10),
    ) {
        let w = 10;
        let h = 10;
        let mut rust_dst = vec![0u32; (w * 2) * (h * 2)];
        let mut cpp_dst = vec![0u32; (w * 2) * (h * 2)];

        hqx_scale_argb8888(HqxScale::X2, &src, w, h, &mut rust_dst).unwrap();
        cpp_hqx_scale_argb8888(HqxScale::X2, &src, w, h, &mut cpp_dst);

        assert_eq!(rust_dst, cpp_dst);
    }

    #[test]
    fn hq3x_is_bit_identical(
        src in prop::collection::vec(any::<u32>(), 8 * 8),
    ) {
        let w = 8;
        let h = 8;
        let mut rust_dst = vec![0u32; (w * 3) * (h * 3)];
        let mut cpp_dst = vec![0u32; (w * 3) * (h * 3)];

        hqx_scale_argb8888(HqxScale::X3, &src, w, h, &mut rust_dst).unwrap();
        cpp_hqx_scale_argb8888(HqxScale::X3, &src, w, h, &mut cpp_dst);

        assert_eq!(rust_dst, cpp_dst);
    }

    #[test]
    fn hq4x_is_bit_identical(
        src in prop::collection::vec(any::<u32>(), 6 * 6),
    ) {
        let w = 6;
        let h = 6;
        let mut rust_dst = vec![0u32; (w * 4) * (h * 4)];
        let mut cpp_dst = vec![0u32; (w * 4) * (h * 4)];

        hqx_scale_argb8888(HqxScale::X4, &src, w, h, &mut rust_dst).unwrap();
        cpp_hqx_scale_argb8888(HqxScale::X4, &src, w, h, &mut cpp_dst);

        assert_eq!(rust_dst, cpp_dst);
    }
}

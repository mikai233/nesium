#[inline]
fn interp(a: u32, b: u32) -> u32 {
    ((a & 0xFEFE_FEFE) >> 1) + ((b & 0xFEFE_FEFE) >> 1) + (a & b & 0x0101_0101)
}

#[inline]
fn interp2(a: u32, b: u32, c: u32, d: u32) -> u32 {
    ((a & 0xFCFC_FCFC) >> 2)
        + ((b & 0xFCFC_FCFC) >> 2)
        + ((c & 0xFCFC_FCFC) >> 2)
        + ((d & 0xFCFC_FCFC) >> 2)
        + ((((a & 0x0303_0303) + (b & 0x0303_0303) + (c & 0x0303_0303) + (d & 0x0303_0303)) >> 2)
            & 0x0303_0303)
}

#[inline]
fn result(a: u32, b: u32, c: u32, d: u32) -> i32 {
    ((a != c || a != d) as i32) - ((b != c || b != d) as i32)
}

#[inline]
fn idx(x: usize, y: usize, stride: usize) -> usize {
    y * stride + x
}

fn neighbor_coords(x: usize, y: usize, width: usize, height: usize) -> NeighborCoords {
    let x0 = x;
    let x_prev = x.saturating_sub(1);
    let x_next = if x + 1 < width { x + 1 } else { x };
    let x_next2 = if x + 2 < width { x + 2 } else { x_next };

    let y_prev = y.saturating_sub(1);
    let y_next = if y + 1 < height { y + 1 } else { y };
    let y_next2 = if y + 2 < height { y + 2 } else { y_next };

    NeighborCoords {
        x_prev,
        x0,
        x_next,
        x_next2,
        y_prev,
        y0: y,
        y_next,
        y_next2,
    }
}

struct NeighborCoords {
    x_prev: usize,
    x0: usize,
    x_next: usize,
    x_next2: usize,
    y_prev: usize,
    y0: usize,
    y_next: usize,
    y_next2: usize,
}

#[cfg(feature = "sai-cpp")]
unsafe extern "C" {
    fn nesium_sai_2xsai_xrgb8888(
        width: u32,
        height: u32,
        src: *const u32,
        src_stride: u32,
        dst: *mut u32,
        dst_stride: u32,
    );
    fn nesium_sai_super2xsai_xrgb8888(
        width: u32,
        height: u32,
        src: *const u32,
        src_stride: u32,
        dst: *mut u32,
        dst_stride: u32,
    );
    fn nesium_sai_supereagle_xrgb8888(
        width: u32,
        height: u32,
        src: *const u32,
        src_stride: u32,
        dst: *mut u32,
        dst_stride: u32,
    );
}

#[cfg(feature = "sai-cpp")]
pub fn scale_2xsai_xrgb8888_cpp(
    width: usize,
    height: usize,
    src: &[u32],
    src_stride: usize,
    dst: &mut [u32],
    dst_stride: usize,
) {
    if width == 0 || height == 0 {
        return;
    }
    debug_assert!(src_stride >= width);
    debug_assert!(dst_stride >= width * 2);
    debug_assert!(src.len() >= src_stride * height);
    debug_assert!(dst.len() >= dst_stride * (height * 2));
    unsafe {
        nesium_sai_2xsai_xrgb8888(
            width as u32,
            height as u32,
            src.as_ptr(),
            src_stride as u32,
            dst.as_mut_ptr(),
            dst_stride as u32,
        );
    }
}

#[cfg(feature = "sai-cpp")]
pub fn scale_super_2xsai_xrgb8888_cpp(
    width: usize,
    height: usize,
    src: &[u32],
    src_stride: usize,
    dst: &mut [u32],
    dst_stride: usize,
) {
    if width == 0 || height == 0 {
        return;
    }
    debug_assert!(src_stride >= width);
    debug_assert!(dst_stride >= width * 2);
    debug_assert!(src.len() >= src_stride * height);
    debug_assert!(dst.len() >= dst_stride * (height * 2));
    unsafe {
        nesium_sai_super2xsai_xrgb8888(
            width as u32,
            height as u32,
            src.as_ptr(),
            src_stride as u32,
            dst.as_mut_ptr(),
            dst_stride as u32,
        );
    }
}

#[cfg(feature = "sai-cpp")]
pub fn scale_supereagle_xrgb8888_cpp(
    width: usize,
    height: usize,
    src: &[u32],
    src_stride: usize,
    dst: &mut [u32],
    dst_stride: usize,
) {
    if width == 0 || height == 0 {
        return;
    }
    debug_assert!(src_stride >= width);
    debug_assert!(dst_stride >= width * 2);
    debug_assert!(src.len() >= src_stride * height);
    debug_assert!(dst.len() >= dst_stride * (height * 2));
    unsafe {
        nesium_sai_supereagle_xrgb8888(
            width as u32,
            height as u32,
            src.as_ptr(),
            src_stride as u32,
            dst.as_mut_ptr(),
            dst_stride as u32,
        );
    }
}

/// 2xSaI scaler (xRGB8888).
///
/// `src_stride`/`dst_stride` are in pixels (not bytes).
pub fn scale_2xsai_xrgb8888(
    width: usize,
    height: usize,
    src: &[u32],
    src_stride: usize,
    dst: &mut [u32],
    dst_stride: usize,
) {
    if width == 0 || height == 0 {
        return;
    }

    for y in 0..height {
        for x in 0..width {
            let n = neighbor_coords(x, y, width, height);

            let color_i = src[idx(n.x_prev, n.y_prev, src_stride)];
            let color_e = src[idx(n.x0, n.y_prev, src_stride)];
            let color_f = src[idx(n.x_next, n.y_prev, src_stride)];
            let color_j = src[idx(n.x_next2, n.y_prev, src_stride)];

            let color_g = src[idx(n.x_prev, n.y0, src_stride)];
            let color_a = src[idx(n.x0, n.y0, src_stride)];
            let color_b = src[idx(n.x_next, n.y0, src_stride)];
            let color_k = src[idx(n.x_next2, n.y0, src_stride)];

            let color_h = src[idx(n.x_prev, n.y_next, src_stride)];
            let color_c = src[idx(n.x0, n.y_next, src_stride)];
            let color_d = src[idx(n.x_next, n.y_next, src_stride)];
            let color_l = src[idx(n.x_next2, n.y_next, src_stride)];

            let color_m = src[idx(n.x_prev, n.y_next2, src_stride)];
            let color_n = src[idx(n.x0, n.y_next2, src_stride)];
            let color_o = src[idx(n.x_next, n.y_next2, src_stride)];

            let (product, product1, product2) = if color_a == color_d && color_b != color_c {
                let product = if (color_a == color_e && color_b == color_l)
                    || (color_a == color_c
                        && color_a == color_f
                        && color_b != color_e
                        && color_b == color_j)
                {
                    color_a
                } else {
                    interp(color_a, color_b)
                };

                let product1 = if (color_a == color_g && color_c == color_o)
                    || (color_a == color_b
                        && color_a == color_h
                        && color_g != color_c
                        && color_c == color_m)
                {
                    color_a
                } else {
                    interp(color_a, color_c)
                };

                (product, product1, color_a)
            } else if color_b == color_c && color_a != color_d {
                let product = if (color_b == color_f && color_a == color_h)
                    || (color_b == color_e
                        && color_b == color_d
                        && color_a != color_f
                        && color_a == color_i)
                {
                    color_b
                } else {
                    interp(color_a, color_b)
                };

                let product1 = if (color_c == color_h && color_a == color_f)
                    || (color_c == color_g
                        && color_c == color_d
                        && color_a != color_h
                        && color_a == color_i)
                {
                    color_c
                } else {
                    interp(color_a, color_c)
                };

                (product, product1, color_b)
            } else if color_a == color_d && color_b == color_c {
                if color_a == color_b {
                    (color_a, color_a, color_a)
                } else {
                    let product1 = interp(color_a, color_c);
                    let product = interp(color_a, color_b);
                    let mut r = 0;
                    r += result(color_a, color_b, color_g, color_e);
                    r += result(color_b, color_a, color_k, color_f);
                    r += result(color_b, color_a, color_h, color_n);
                    r += result(color_a, color_b, color_l, color_o);

                    let product2 = if r > 0 {
                        color_a
                    } else if r < 0 {
                        color_b
                    } else {
                        interp2(color_a, color_b, color_c, color_d)
                    };

                    (product, product1, product2)
                }
            } else {
                let product2 = interp2(color_a, color_b, color_c, color_d);

                let product = if color_a == color_c
                    && color_a == color_f
                    && color_b != color_e
                    && color_b == color_j
                {
                    color_a
                } else if color_b == color_e
                    && color_b == color_d
                    && color_a != color_f
                    && color_a == color_i
                {
                    color_b
                } else {
                    interp(color_a, color_b)
                };

                let product1 = if color_a == color_b
                    && color_a == color_h
                    && color_g != color_c
                    && color_c == color_m
                {
                    color_a
                } else if color_c == color_g
                    && color_c == color_d
                    && color_a != color_h
                    && color_a == color_i
                {
                    color_c
                } else {
                    interp(color_a, color_c)
                };

                (product, product1, product2)
            };

            let out_y = y * 2;
            let out_x = x * 2;
            let o0 = idx(out_x, out_y, dst_stride);
            let o1 = o0 + 1;
            let o2 = idx(out_x, out_y + 1, dst_stride);
            let o3 = o2 + 1;

            dst[o0] = color_a;
            dst[o1] = product;
            dst[o2] = product1;
            dst[o3] = product2;
        }
    }
}

/// Super 2xSaI scaler (xRGB8888).
///
/// `src_stride`/`dst_stride` are in pixels (not bytes).
pub fn scale_super_2xsai_xrgb8888(
    width: usize,
    height: usize,
    src: &[u32],
    src_stride: usize,
    dst: &mut [u32],
    dst_stride: usize,
) {
    if width == 0 || height == 0 {
        return;
    }

    for y in 0..height {
        for x in 0..width {
            let n = neighbor_coords(x, y, width, height);

            let color_b0 = src[idx(n.x_prev, n.y_prev, src_stride)];
            let color_b1 = src[idx(n.x0, n.y_prev, src_stride)];
            let color_b2 = src[idx(n.x_next, n.y_prev, src_stride)];
            let color_b3 = src[idx(n.x_next2, n.y_prev, src_stride)];
            let color4 = src[idx(n.x_prev, n.y0, src_stride)];
            let color5 = src[idx(n.x0, n.y0, src_stride)];
            let color6 = src[idx(n.x_next, n.y0, src_stride)];
            let color_s2 = src[idx(n.x_next2, n.y0, src_stride)];
            let color1 = src[idx(n.x_prev, n.y_next, src_stride)];
            let color2 = src[idx(n.x0, n.y_next, src_stride)];
            let color3 = src[idx(n.x_next, n.y_next, src_stride)];
            let color_s1 = src[idx(n.x_next2, n.y_next, src_stride)];
            let color_a0 = src[idx(n.x_prev, n.y_next2, src_stride)];
            let color_a1 = src[idx(n.x0, n.y_next2, src_stride)];
            let color_a2 = src[idx(n.x_next, n.y_next2, src_stride)];
            let color_a3 = src[idx(n.x_next2, n.y_next2, src_stride)];

            let (product1b, product2b) = if color2 == color6 && color5 != color3 {
                (color2, color2)
            } else if color5 == color3 && color2 != color6 {
                (color5, color5)
            } else if color5 == color3 && color2 == color6 {
                let mut r = 0;
                r += result(color6, color5, color1, color_a1);
                r += result(color6, color5, color4, color_b1);
                r += result(color6, color5, color_a2, color_s1);
                r += result(color6, color5, color_b2, color_s2);
                if r > 0 {
                    (color6, color6)
                } else if r < 0 {
                    (color5, color5)
                } else {
                    let p = interp(color5, color6);
                    (p, p)
                }
            } else {
                let product2b = if color6 == color3
                    && color3 == color_a1
                    && color2 != color_a2
                    && color3 != color_a0
                {
                    interp2(color3, color3, color3, color2)
                } else if (color5 == color2 && color2 == color_a2)
                    && (color_a1 != color3 && color2 != color_a3)
                {
                    interp2(color2, color2, color2, color3)
                } else {
                    interp(color2, color3)
                };

                let product1b = if color6 == color3
                    && color6 == color_b1
                    && color5 != color_b2
                    && color6 != color_b0
                {
                    interp2(color6, color6, color6, color5)
                } else if color5 == color2
                    && color5 == color_b2
                    && color_b1 != color6
                    && color5 != color_b3
                {
                    interp2(color6, color5, color5, color5)
                } else {
                    interp(color5, color6)
                };

                (product1b, product2b)
            };

            let product2a = if color5 == color3
                && color2 != color6
                && color4 == color5
                && color5 != color_a2
            {
                interp(color2, color5)
            } else if color5 == color1 && color6 == color5 && color4 != color2 && color5 != color_a0
            {
                interp(color2, color5)
            } else {
                color2
            };

            let product1a = if color2 == color6
                && color5 != color3
                && color1 == color2
                && color2 != color_b2
            {
                interp(color2, color5)
            } else if color4 == color2 && color3 == color2 && color1 != color5 && color2 != color_b0
            {
                interp(color2, color5)
            } else {
                color5
            };

            let out_y = y * 2;
            let out_x = x * 2;
            let o0 = idx(out_x, out_y, dst_stride);
            let o1 = o0 + 1;
            let o2 = idx(out_x, out_y + 1, dst_stride);
            let o3 = o2 + 1;

            dst[o0] = product1a;
            dst[o1] = product1b;
            dst[o2] = product2a;
            dst[o3] = product2b;
        }
    }
}

/// SuperEagle scaler (xRGB8888).
///
/// `src_stride`/`dst_stride` are in pixels (not bytes).
pub fn scale_supereagle_xrgb8888(
    width: usize,
    height: usize,
    src: &[u32],
    src_stride: usize,
    dst: &mut [u32],
    dst_stride: usize,
) {
    if width == 0 || height == 0 {
        return;
    }

    for y in 0..height {
        for x in 0..width {
            let n = neighbor_coords(x, y, width, height);

            let color_b1 = src[idx(n.x0, n.y_prev, src_stride)];
            let color_b2 = src[idx(n.x_next, n.y_prev, src_stride)];
            let color4 = src[idx(n.x_prev, n.y0, src_stride)];
            let color5 = src[idx(n.x0, n.y0, src_stride)];
            let color6 = src[idx(n.x_next, n.y0, src_stride)];
            let color_s2 = src[idx(n.x_next2, n.y0, src_stride)];
            let color1 = src[idx(n.x_prev, n.y_next, src_stride)];
            let color2 = src[idx(n.x0, n.y_next, src_stride)];
            let color3 = src[idx(n.x_next, n.y_next, src_stride)];
            let color_s1 = src[idx(n.x_next2, n.y_next, src_stride)];
            let color_a1 = src[idx(n.x0, n.y_next2, src_stride)];
            let color_a2 = src[idx(n.x_next, n.y_next2, src_stride)];

            let (product1a, product1b, product2a, product2b) =
                if color2 == color6 && color5 != color3 {
                    let product1b = color2;
                    let product2a = color2;

                    let product1a = if color1 == color2 || color6 == color_b2 {
                        let p = interp(color2, color5);
                        interp(color2, p)
                    } else {
                        interp(color5, color6)
                    };

                    let product2b = if color6 == color_s2 || color2 == color_a1 {
                        let p = interp(color2, color3);
                        interp(color2, p)
                    } else {
                        interp(color2, color3)
                    };

                    (product1a, product1b, product2a, product2b)
                } else if color5 == color3 && color2 != color6 {
                    let product2b = color5;
                    let product1a = color5;

                    let product1b = if color_b1 == color5 || color3 == color_s1 {
                        let p = interp(color5, color6);
                        interp(color5, p)
                    } else {
                        interp(color5, color6)
                    };

                    let product2a = if color3 == color_a2 || color4 == color5 {
                        let p = interp(color5, color2);
                        interp(color5, p)
                    } else {
                        interp(color2, color3)
                    };

                    (product1a, product1b, product2a, product2b)
                } else if color5 == color3 && color2 == color6 {
                    let mut r = 0;
                    r += result(color6, color5, color1, color_a1);
                    r += result(color6, color5, color4, color_b1);
                    r += result(color6, color5, color_a2, color_s1);
                    r += result(color6, color5, color_b2, color_s2);
                    if r > 0 {
                        let p = interp(color5, color6);
                        (p, color2, color2, p)
                    } else if r < 0 {
                        let p = interp(color5, color6);
                        (color5, p, p, color5)
                    } else {
                        (color5, color2, color2, color5)
                    }
                } else {
                    let p = interp(color2, color6);
                    let product2b = interp2(color3, color3, color3, p);
                    let product1a = interp2(color5, color5, color5, p);
                    let q = interp(color5, color3);
                    let product2a = interp2(color2, color2, color2, q);
                    let product1b = interp2(color6, color6, color6, q);
                    (product1a, product1b, product2a, product2b)
                };

            let out_y = y * 2;
            let out_x = x * 2;
            let o0 = idx(out_x, out_y, dst_stride);
            let o1 = o0 + 1;
            let o2 = idx(out_x, out_y + 1, dst_stride);
            let o3 = o2 + 1;

            dst[o0] = product1a;
            dst[o1] = product1b;
            dst[o2] = product2a;
            dst[o3] = product2b;
        }
    }
}

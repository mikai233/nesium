use super::hqx_common::*;

// PIXEL00 opcodes
#[inline(always)]
fn render_p00(dst: &mut u32, w: &[u32; 10], op: u8) {
    match op {
        0 => *dst = w[5],
        11 => *dst = interp1(w[5], w[4]),
        12 => *dst = interp1(w[5], w[2]),
        20 => *dst = interp2(w[5], w[2], w[4]),
        50 => *dst = interp5(w[2], w[4]),
        80 => *dst = interp8(w[5], w[1]),
        81 => *dst = interp8(w[5], w[4]),
        82 => *dst = interp8(w[5], w[2]),
        _ => unreachable!(),
    }
}

// PIXEL01 opcodes
#[inline(always)]
fn render_p01(dst: &mut u32, w: &[u32; 10], op: u8) {
    match op {
        0 => *dst = w[5],
        10 => *dst = interp1(w[5], w[1]),
        12 => *dst = interp1(w[5], w[2]),
        14 => *dst = interp1(w[2], w[5]),
        21 => *dst = interp2(w[2], w[5], w[4]),
        31 => *dst = interp3(w[5], w[4]),
        50 => *dst = interp5(w[2], w[5]),
        60 => *dst = interp6(w[5], w[2], w[4]),
        61 => *dst = interp6(w[5], w[2], w[1]),
        82 => *dst = interp8(w[5], w[2]),
        83 => *dst = interp8(w[2], w[4]),
        _ => unreachable!(),
    }
}

// PIXEL02 opcodes
#[inline(always)]
fn render_p02(dst: &mut u32, w: &[u32; 10], op: u8) {
    match op {
        0 => *dst = w[5],
        10 => *dst = interp1(w[5], w[3]),
        11 => *dst = interp1(w[5], w[2]),
        13 => *dst = interp1(w[2], w[5]),
        21 => *dst = interp2(w[2], w[5], w[6]),
        32 => *dst = interp3(w[5], w[6]),
        50 => *dst = interp5(w[2], w[5]),
        60 => *dst = interp6(w[5], w[2], w[6]),
        61 => *dst = interp6(w[5], w[2], w[3]),
        81 => *dst = interp8(w[5], w[2]),
        83 => *dst = interp8(w[2], w[6]),
        _ => unreachable!(),
    }
}

// PIXEL03 opcodes
#[inline(always)]
fn render_p03(dst: &mut u32, w: &[u32; 10], op: u8) {
    match op {
        0 => *dst = w[5],
        11 => *dst = interp1(w[5], w[2]),
        12 => *dst = interp1(w[5], w[6]),
        20 => *dst = interp2(w[5], w[2], w[6]),
        50 => *dst = interp5(w[2], w[6]),
        80 => *dst = interp8(w[5], w[3]),
        81 => *dst = interp8(w[5], w[2]),
        82 => *dst = interp8(w[5], w[6]),
        _ => unreachable!(),
    }
}

// PIXEL10 opcodes
#[inline(always)]
fn render_p10(dst: &mut u32, w: &[u32; 10], op: u8) {
    match op {
        0 => *dst = w[5],
        10 => *dst = interp1(w[5], w[1]),
        11 => *dst = interp1(w[5], w[4]),
        13 => *dst = interp1(w[4], w[5]),
        21 => *dst = interp2(w[4], w[5], w[2]),
        32 => *dst = interp3(w[5], w[2]),
        50 => *dst = interp5(w[4], w[5]),
        60 => *dst = interp6(w[5], w[4], w[2]),
        61 => *dst = interp6(w[5], w[4], w[1]),
        81 => *dst = interp8(w[5], w[4]),
        83 => *dst = interp8(w[4], w[2]),
        _ => unreachable!(),
    }
}

// PIXEL11 opcodes
#[inline(always)]
fn render_p11(dst: &mut u32, w: &[u32; 10], op: u8) {
    match op {
        0 => *dst = w[5],
        30 => *dst = interp3(w[5], w[1]),
        31 => *dst = interp3(w[5], w[4]),
        32 => *dst = interp3(w[5], w[2]),
        70 => *dst = interp7(w[5], w[4], w[2]),
        _ => unreachable!(),
    }
}

// PIXEL12 opcodes
#[inline(always)]
fn render_p12(dst: &mut u32, w: &[u32; 10], op: u8) {
    match op {
        0 => *dst = w[5],
        30 => *dst = interp3(w[5], w[3]),
        31 => *dst = interp3(w[5], w[2]),
        32 => *dst = interp3(w[5], w[6]),
        70 => *dst = interp7(w[5], w[6], w[2]),
        _ => unreachable!(),
    }
}

// PIXEL13 opcodes
#[inline(always)]
fn render_p13(dst: &mut u32, w: &[u32; 10], op: u8) {
    match op {
        0 => *dst = w[5],
        10 => *dst = interp1(w[5], w[3]),
        12 => *dst = interp1(w[5], w[6]),
        14 => *dst = interp1(w[6], w[5]),
        21 => *dst = interp2(w[6], w[5], w[2]),
        31 => *dst = interp3(w[5], w[2]),
        50 => *dst = interp5(w[6], w[5]),
        60 => *dst = interp6(w[5], w[6], w[2]),
        61 => *dst = interp6(w[5], w[6], w[3]),
        82 => *dst = interp8(w[5], w[6]),
        83 => *dst = interp8(w[6], w[2]),
        _ => unreachable!(),
    }
}

// PIXEL20 opcodes
#[inline(always)]
fn render_p20(dst: &mut u32, w: &[u32; 10], op: u8) {
    match op {
        0 => *dst = w[5],
        10 => *dst = interp1(w[5], w[7]),
        12 => *dst = interp1(w[5], w[4]),
        14 => *dst = interp1(w[4], w[5]),
        21 => *dst = interp2(w[4], w[5], w[8]),
        31 => *dst = interp3(w[5], w[8]),
        50 => *dst = interp5(w[4], w[5]),
        60 => *dst = interp6(w[5], w[4], w[8]),
        61 => *dst = interp6(w[5], w[4], w[7]),
        82 => *dst = interp8(w[5], w[4]),
        83 => *dst = interp8(w[4], w[8]),
        _ => unreachable!(),
    }
}

// PIXEL21 opcodes
#[inline(always)]
fn render_p21(dst: &mut u32, w: &[u32; 10], op: u8) {
    match op {
        0 => *dst = w[5],
        30 => *dst = interp3(w[5], w[7]),
        31 => *dst = interp3(w[5], w[8]),
        32 => *dst = interp3(w[5], w[4]),
        70 => *dst = interp7(w[5], w[4], w[8]),
        _ => unreachable!(),
    }
}

// PIXEL22 opcodes
#[inline(always)]
fn render_p22(dst: &mut u32, w: &[u32; 10], op: u8) {
    match op {
        0 => *dst = w[5],
        30 => *dst = interp3(w[5], w[9]),
        31 => *dst = interp3(w[5], w[6]),
        32 => *dst = interp3(w[5], w[8]),
        70 => *dst = interp7(w[5], w[6], w[8]),
        _ => unreachable!(),
    }
}

// PIXEL23 opcodes
#[inline(always)]
fn render_p23(dst: &mut u32, w: &[u32; 10], op: u8) {
    match op {
        0 => *dst = w[5],
        10 => *dst = interp1(w[5], w[9]),
        11 => *dst = interp1(w[5], w[6]),
        13 => *dst = interp1(w[6], w[5]),
        21 => *dst = interp2(w[6], w[5], w[8]),
        32 => *dst = interp3(w[5], w[8]),
        50 => *dst = interp5(w[6], w[5]),
        60 => *dst = interp6(w[5], w[6], w[8]),
        61 => *dst = interp6(w[5], w[6], w[9]),
        81 => *dst = interp8(w[5], w[6]),
        83 => *dst = interp8(w[6], w[8]),
        _ => unreachable!(),
    }
}

// PIXEL30 opcodes
#[inline(always)]
fn render_p30(dst: &mut u32, w: &[u32; 10], op: u8) {
    match op {
        0 => *dst = w[5],
        11 => *dst = interp1(w[5], w[8]),
        12 => *dst = interp1(w[5], w[4]),
        20 => *dst = interp2(w[5], w[8], w[4]),
        50 => *dst = interp5(w[8], w[4]),
        80 => *dst = interp8(w[5], w[7]),
        81 => *dst = interp8(w[5], w[8]),
        82 => *dst = interp8(w[5], w[4]),
        _ => unreachable!(),
    }
}

// PIXEL31 opcodes
#[inline(always)]
fn render_p31(dst: &mut u32, w: &[u32; 10], op: u8) {
    match op {
        0 => *dst = w[5],
        10 => *dst = interp1(w[5], w[7]),
        11 => *dst = interp1(w[5], w[8]),
        13 => *dst = interp1(w[8], w[5]),
        21 => *dst = interp2(w[8], w[5], w[4]),
        32 => *dst = interp3(w[5], w[4]),
        50 => *dst = interp5(w[8], w[5]),
        60 => *dst = interp6(w[5], w[8], w[4]),
        61 => *dst = interp6(w[5], w[8], w[7]),
        81 => *dst = interp8(w[5], w[8]),
        83 => *dst = interp8(w[8], w[4]),
        _ => unreachable!(),
    }
}

// PIXEL32 opcodes
#[inline(always)]
fn render_p32(dst: &mut u32, w: &[u32; 10], op: u8) {
    match op {
        0 => *dst = w[5],
        10 => *dst = interp1(w[5], w[9]),
        12 => *dst = interp1(w[5], w[8]),
        14 => *dst = interp1(w[8], w[5]),
        21 => *dst = interp2(w[8], w[5], w[6]),
        31 => *dst = interp3(w[5], w[6]),
        50 => *dst = interp5(w[8], w[5]),
        60 => *dst = interp6(w[5], w[8], w[6]),
        61 => *dst = interp6(w[5], w[8], w[9]),
        82 => *dst = interp8(w[5], w[8]),
        83 => *dst = interp8(w[8], w[6]),
        _ => unreachable!(),
    }
}

// PIXEL33 opcodes
#[inline(always)]
fn render_p33(dst: &mut u32, w: &[u32; 10], op: u8) {
    match op {
        0 => *dst = w[5],
        11 => *dst = interp1(w[5], w[6]),
        12 => *dst = interp1(w[5], w[8]),
        20 => *dst = interp2(w[5], w[8], w[6]),
        50 => *dst = interp5(w[8], w[6]),
        80 => *dst = interp8(w[5], w[9]),
        81 => *dst = interp8(w[5], w[6]),
        82 => *dst = interp8(w[5], w[8]),
        _ => unreachable!(),
    }
}

pub fn hq4x_32_rb(
    sp: &[u32],
    sp_stride: usize,
    dp: &mut [u32],
    dp_stride: usize,
    width: usize,
    height: usize,
) {
    let mut w = [0u32; 10];

    for j in 0..height {
        let prev_line = if j > 0 { -(sp_stride as isize) } else { 0 };
        let next_line = if j < height - 1 {
            sp_stride as isize
        } else {
            0
        };

        for i in 0..width {
            let curr_ptr = j * sp_stride + i;

            w[2] = sp[(curr_ptr as isize + prev_line) as usize];
            w[5] = sp[curr_ptr];
            w[8] = sp[(curr_ptr as isize + next_line) as usize];

            if i > 0 {
                w[1] = sp[(curr_ptr as isize + prev_line - 1) as usize];
                w[4] = sp[curr_ptr - 1];
                w[7] = sp[(curr_ptr as isize + next_line - 1) as usize];
            } else {
                w[1] = w[2];
                w[4] = w[5];
                w[7] = w[8];
            }

            if i < width - 1 {
                w[3] = sp[(curr_ptr as isize + prev_line + 1) as usize];
                w[6] = sp[curr_ptr + 1];
                w[9] = sp[(curr_ptr as isize + next_line + 1) as usize];
            } else {
                w[3] = w[2];
                w[6] = w[5];
                w[9] = w[8];
            }

            let mut pattern = 0;
            let mut flag = 1;
            let center_yuv = rgb_to_yuv(w[5]);

            for k in [1, 2, 3, 4, 6, 7, 8, 9] {
                if w[k] != w[5] {
                    if yuv_diff(center_yuv, rgb_to_yuv(w[k])) {
                        pattern |= flag;
                    }
                }
                flag <<= 1;
            }

            let mut v00 = 0;
            let mut v01 = 0;
            let mut v02 = 0;
            let mut v03 = 0;
            let mut v10 = 0;
            let mut v11 = 0;
            let mut v12 = 0;
            let mut v13 = 0;
            let mut v20 = 0;
            let mut v21 = 0;
            let mut v22 = 0;
            let mut v23 = 0;
            let mut v30 = 0;
            let mut v31 = 0;
            let mut v32 = 0;
            let mut v33 = 0;

            match pattern {
                0 | 1 | 4 | 32 | 128 | 5 | 132 | 160 | 33 | 129 | 36 | 133 | 164 | 161 | 37
                | 165 => {
                    v00 = 20;
                    v01 = 60;
                    v02 = 60;
                    v03 = 20;
                    v10 = 60;
                    v11 = 70;
                    v12 = 70;
                    v13 = 60;
                    v20 = 60;
                    v21 = 70;
                    v22 = 70;
                    v23 = 60;
                    v30 = 20;
                    v31 = 60;
                    v32 = 60;
                    v33 = 20;
                }
                2 | 34 | 130 | 162 => {
                    v00 = 80;
                    v01 = 10;
                    v02 = 10;
                    v03 = 80;
                    v10 = 61;
                    v11 = 30;
                    v12 = 30;
                    v13 = 61;
                    v20 = 60;
                    v21 = 70;
                    v22 = 70;
                    v23 = 60;
                    v30 = 20;
                    v31 = 60;
                    v32 = 60;
                    v33 = 20;
                }
                16 | 17 | 48 | 49 => {
                    v00 = 20;
                    v01 = 60;
                    v02 = 61;
                    v03 = 80;
                    v10 = 60;
                    v11 = 70;
                    v12 = 30;
                    v13 = 10;
                    v20 = 60;
                    v21 = 70;
                    v22 = 30;
                    v23 = 10;
                    v30 = 20;
                    v31 = 60;
                    v32 = 61;
                    v33 = 80;
                }
                64 | 65 | 68 | 69 => {
                    v00 = 20;
                    v01 = 60;
                    v02 = 60;
                    v03 = 20;
                    v10 = 60;
                    v11 = 70;
                    v12 = 70;
                    v13 = 60;
                    v20 = 61;
                    v21 = 30;
                    v22 = 30;
                    v23 = 61;
                    v30 = 80;
                    v31 = 10;
                    v32 = 10;
                    v33 = 80;
                }
                8 | 12 | 136 | 140 => {
                    v00 = 80;
                    v01 = 61;
                    v02 = 60;
                    v03 = 20;
                    v10 = 10;
                    v11 = 30;
                    v12 = 70;
                    v13 = 60;
                    v20 = 10;
                    v21 = 30;
                    v22 = 70;
                    v23 = 60;
                    v30 = 80;
                    v31 = 61;
                    v32 = 60;
                    v33 = 20;
                }
                3 | 35 | 131 | 163 => {
                    v00 = 81;
                    v01 = 31;
                    v02 = 10;
                    v03 = 80;
                    v10 = 81;
                    v11 = 31;
                    v12 = 30;
                    v13 = 61;
                    v20 = 60;
                    v21 = 70;
                    v22 = 70;
                    v23 = 60;
                    v30 = 20;
                    v31 = 60;
                    v32 = 60;
                    v33 = 20;
                }
                6 | 38 | 134 | 166 => {
                    v00 = 80;
                    v01 = 10;
                    v02 = 32;
                    v03 = 82;
                    v10 = 61;
                    v11 = 30;
                    v12 = 32;
                    v13 = 82;
                    v20 = 60;
                    v21 = 70;
                    v22 = 70;
                    v23 = 60;
                    v30 = 20;
                    v31 = 60;
                    v32 = 60;
                    v33 = 20;
                }
                20 | 21 | 52 | 53 => {
                    v00 = 20;
                    v01 = 60;
                    v02 = 81;
                    v03 = 81;
                    v10 = 60;
                    v11 = 70;
                    v12 = 31;
                    v13 = 31;
                    v20 = 60;
                    v21 = 70;
                    v22 = 30;
                    v23 = 10;
                    v30 = 20;
                    v31 = 60;
                    v32 = 61;
                    v33 = 80;
                }
                144 | 145 | 176 | 177 => {
                    v00 = 20;
                    v01 = 60;
                    v02 = 61;
                    v03 = 80;
                    v10 = 60;
                    v11 = 70;
                    v12 = 30;
                    v13 = 10;
                    v20 = 60;
                    v21 = 70;
                    v22 = 32;
                    v23 = 32;
                    v30 = 20;
                    v31 = 60;
                    v32 = 82;
                    v33 = 82;
                }
                192 | 193 | 196 | 197 => {
                    v00 = 20;
                    v01 = 60;
                    v02 = 60;
                    v03 = 20;
                    v10 = 60;
                    v11 = 70;
                    v12 = 70;
                    v13 = 60;
                    v20 = 61;
                    v21 = 30;
                    v22 = 31;
                    v23 = 81;
                    v30 = 80;
                    v31 = 10;
                    v32 = 31;
                    v33 = 81;
                }
                96 | 97 | 100 | 101 => {
                    v00 = 20;
                    v01 = 60;
                    v02 = 60;
                    v03 = 20;
                    v10 = 60;
                    v11 = 70;
                    v12 = 70;
                    v13 = 60;
                    v20 = 82;
                    v21 = 32;
                    v22 = 30;
                    v23 = 61;
                    v30 = 82;
                    v31 = 32;
                    v32 = 10;
                    v33 = 80;
                }
                40 | 44 | 168 | 172 => {
                    v00 = 80;
                    v01 = 61;
                    v02 = 60;
                    v03 = 20;
                    v10 = 10;
                    v11 = 30;
                    v12 = 70;
                    v13 = 60;
                    v20 = 31;
                    v21 = 31;
                    v22 = 70;
                    v23 = 60;
                    v30 = 81;
                    v31 = 81;
                    v32 = 60;
                    v33 = 20;
                }
                9 | 13 | 137 | 141 => {
                    v00 = 82;
                    v01 = 82;
                    v02 = 60;
                    v03 = 20;
                    v10 = 32;
                    v11 = 32;
                    v12 = 70;
                    v13 = 60;
                    v20 = 10;
                    v21 = 30;
                    v22 = 70;
                    v23 = 60;
                    v30 = 80;
                    v31 = 61;
                    v32 = 60;
                    v33 = 20;
                }
                18 | 50 => {
                    v00 = 80;
                    v10 = 10;
                    v20 = 20;
                    v21 = 60;
                    v22 = 61;
                    v23 = 80;
                    v30 = 20;
                    v31 = 60;
                    v32 = 61;
                    v33 = 80;
                    if diff(w[2], w[6]) {
                        v01 = 10;
                        v02 = 10;
                        v03 = 80;
                        v11 = 30;
                        v12 = 30;
                        v13 = 10;
                    } else {
                        v01 = 50;
                        v02 = 50;
                        v03 = 50;
                        v11 = 30;
                        v12 = 0;
                        v13 = 50;
                    }
                }
                80 | 81 => {
                    v00 = 20;
                    v01 = 60;
                    v02 = 61;
                    v03 = 80;
                    v10 = 60;
                    v11 = 70;
                    v12 = 30;
                    v13 = 10;
                    v20 = 61;
                    v21 = 30;
                    v30 = 80;
                    v31 = 10;
                    if diff(w[6], w[8]) {
                        v22 = 30;
                        v23 = 10;
                        v32 = 10;
                        v33 = 80;
                    } else {
                        v22 = 0;
                        v23 = 50;
                        v32 = 50;
                        v33 = 50;
                    }
                }
                72 | 76 => {
                    v00 = 80;
                    v01 = 61;
                    v02 = 60;
                    v03 = 20;
                    v10 = 10;
                    v11 = 30;
                    v12 = 70;
                    v13 = 60;
                    v22 = 30;
                    v23 = 61;
                    v32 = 10;
                    v33 = 80;
                    if diff(w[8], w[4]) {
                        v20 = 10;
                        v21 = 30;
                        v30 = 80;
                        v31 = 10;
                    } else {
                        v20 = 50;
                        v21 = 0;
                        v30 = 50;
                        v31 = 50;
                    }
                }
                10 | 138 => {
                    v02 = 10;
                    v03 = 80;
                    v12 = 30;
                    v13 = 61;
                    v20 = 10;
                    v21 = 30;
                    v22 = 70;
                    v23 = 60;
                    v30 = 80;
                    v31 = 61;
                    v32 = 60;
                    v33 = 20;
                    if diff(w[4], w[2]) {
                        v00 = 80;
                        v01 = 10;
                        v10 = 10;
                        v11 = 30;
                    } else {
                        v00 = 50;
                        v01 = 50;
                        v10 = 50;
                        v11 = 0;
                    }
                }
                66 => {
                    v00 = 80;
                    v01 = 10;
                    v02 = 10;
                    v03 = 80;
                    v10 = 61;
                    v11 = 30;
                    v12 = 30;
                    v13 = 61;
                    v20 = 61;
                    v21 = 30;
                    v22 = 30;
                    v23 = 61;
                    v30 = 80;
                    v31 = 10;
                    v32 = 10;
                    v33 = 80;
                }
                24 => {
                    v00 = 80;
                    v01 = 61;
                    v02 = 61;
                    v03 = 80;
                    v10 = 10;
                    v11 = 30;
                    v12 = 30;
                    v13 = 10;
                    v20 = 10;
                    v21 = 30;
                    v22 = 30;
                    v23 = 10;
                    v30 = 80;
                    v31 = 61;
                    v32 = 61;
                    v33 = 80;
                }
                7 | 39 | 135 => {
                    v00 = 81;
                    v01 = 31;
                    v02 = 32;
                    v03 = 82;
                    v10 = 81;
                    v11 = 31;
                    v12 = 32;
                    v13 = 82;
                    v20 = 60;
                    v21 = 70;
                    v22 = 70;
                    v23 = 60;
                    v30 = 20;
                    v31 = 60;
                    v32 = 60;
                    v33 = 20;
                }
                148 | 149 | 180 => {
                    v00 = 20;
                    v01 = 60;
                    v02 = 81;
                    v03 = 81;
                    v10 = 60;
                    v11 = 70;
                    v12 = 31;
                    v13 = 31;
                    v20 = 60;
                    v21 = 70;
                    v22 = 32;
                    v23 = 32;
                    v30 = 20;
                    v31 = 60;
                    v32 = 82;
                    v33 = 82;
                }
                224 | 228 | 225 => {
                    v00 = 20;
                    v01 = 60;
                    v02 = 60;
                    v03 = 20;
                    v10 = 60;
                    v11 = 70;
                    v12 = 70;
                    v13 = 60;
                    v20 = 82;
                    v21 = 32;
                    v22 = 31;
                    v23 = 81;
                    v30 = 82;
                    v31 = 32;
                    v32 = 31;
                    v33 = 81;
                }
                41 | 169 | 45 => {
                    v00 = 82;
                    v01 = 82;
                    v02 = 60;
                    v03 = 20;
                    v10 = 32;
                    v11 = 32;
                    v12 = 70;
                    v13 = 60;
                    v20 = 31;
                    v21 = 31;
                    v22 = 70;
                    v23 = 60;
                    v30 = 81;
                    v31 = 81;
                    v32 = 60;
                    v33 = 20;
                }
                22 | 54 => {
                    v00 = 80;
                    v10 = 10;
                    v20 = 20;
                    v21 = 60;
                    v22 = 61;
                    v23 = 80;
                    v30 = 20;
                    v31 = 60;
                    v32 = 61;
                    v33 = 80;
                    if diff(w[2], w[6]) {
                        v01 = 1;
                        v02 = 0;
                        v03 = 0;
                        v11 = 3;
                        v12 = 0;
                        v13 = 0;
                    } else {
                        v01 = 6;
                        v02 = 10;
                        v03 = 10;
                        v11 = 6;
                        v12 = 0;
                        v13 = 6;
                    }
                }
                208 | 209 => {
                    v00 = 3;
                    v01 = 7;
                    v02 = 8;
                    v03 = 5;
                    v10 = 7;
                    v11 = 4;
                    v12 = 3;
                    v13 = 2;
                    v20 = 8;
                    v21 = 3;
                    v30 = 5;
                    v31 = 2;
                    if diff(w[6], w[8]) {
                        v22 = 4;
                        v23 = 0;
                        v32 = 2;
                        v33 = 0;
                    } else {
                        v22 = 0;
                        v23 = 6;
                        v32 = 6;
                        v33 = 6;
                    }
                }
                104 | 108 => {
                    v00 = 5;
                    v01 = 8;
                    v02 = 7;
                    v03 = 3;
                    v10 = 1;
                    v11 = 3;
                    v12 = 4;
                    v13 = 7;
                    v22 = 4;
                    v23 = 8;
                    v32 = 2;
                    v33 = 5;
                    if diff(w[8], w[4]) {
                        v20 = 0;
                        v21 = 0;
                        v30 = 0;
                        v31 = 0;
                    } else {
                        v20 = 6;
                        v21 = 0;
                        v30 = 6;
                        v31 = 6;
                    }
                }
                11 | 139 => {
                    v02 = 1;
                    v03 = 5;
                    v12 = 3;
                    v13 = 8;
                    v20 = 1;
                    v21 = 3;
                    v22 = 4;
                    v23 = 7;
                    v30 = 5;
                    v31 = 8;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 0;
                        v10 = 0;
                        v11 = 0;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                }
                19 | 51 => {
                    v10 = 7;
                    v11 = 4;
                    v20 = 7;
                    v21 = 4;
                    v22 = 3;
                    v23 = 2;
                    v30 = 3;
                    v31 = 7;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[2], w[6]) {
                        v00 = 6;
                        v01 = 5;
                        v02 = 1;
                        v03 = 1;
                        v12 = 3;
                        v13 = 3;
                    } else {
                        v00 = 3;
                        v01 = 7;
                        v02 = 10;
                        v03 = 10;
                        v12 = 0;
                        v13 = 7;
                    }
                }
                146 | 178 => {
                    v00 = 5;
                    v01 = 1;
                    v10 = 8;
                    v11 = 3;
                    v20 = 8;
                    v21 = 3;
                    v22 = 9;
                    v23 = 7;
                    v30 = 5;
                    v31 = 2;
                    if diff(w[2], w[6]) {
                        v02 = 1;
                        v03 = 1;
                        v12 = 3;
                        v13 = 3;
                        v32 = 3;
                        v33 = 6;
                    } else {
                        v02 = 6;
                        v03 = 3;
                        v12 = 5;
                        v13 = 5;
                        v32 = 10;
                        v33 = 10;
                    }
                }
                84 | 85 => {
                    v00 = 3;
                    v01 = 7;
                    v02 = 9;
                    v03 = 7;
                    v10 = 7;
                    v11 = 4;
                    v20 = 8;
                    v21 = 3;
                    v22 = 5;
                    v23 = 5;
                    v30 = 5;
                    v31 = 2;
                    if diff(w[6], w[8]) {
                        v12 = 3;
                        v13 = 3;
                        v23 = 5;
                        v32 = 6;
                        v33 = 5;
                    } else {
                        v12 = 5;
                        v13 = 5;
                        v23 = 10;
                        v32 = 3;
                        v33 = 10;
                    }
                }
                112 | 113 => {
                    v00 = 3;
                    v01 = 7;
                    v02 = 7;
                    v03 = 3;
                    v10 = 7;
                    v11 = 4;
                    v12 = 4;
                    v13 = 7;
                    v20 = 6;
                    v21 = 5;
                    v30 = 6;
                    v31 = 5;
                    if diff(w[6], w[8]) {
                        v22 = 3;
                        v23 = 3;
                        v32 = 1;
                        v33 = 1;
                    } else {
                        v22 = 5;
                        v23 = 10;
                        v32 = 5;
                        v33 = 10;
                    }
                }
                200 | 204 => {
                    v00 = 5;
                    v01 = 8;
                    v02 = 7;
                    v03 = 3;
                    v11 = 3;
                    v12 = 4;
                    v13 = 7;
                    v22 = 5;
                    v23 = 9;
                    v32 = 6;
                    v33 = 7;
                    if diff(w[8], w[4]) {
                        v10 = 1;
                        v20 = 1;
                        v21 = 3;
                        v30 = 1;
                        v31 = 1;
                    } else {
                        v10 = 6;
                        v20 = 5;
                        v21 = 6;
                        v30 = 5;
                        v31 = 10;
                    }
                }
                73 | 77 => {
                    v01 = 8;
                    v02 = 7;
                    v03 = 3;
                    v11 = 3;
                    v12 = 4;
                    v13 = 7;
                    v22 = 4;
                    v23 = 7;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[8], w[4]) {
                        v00 = 1;
                        v10 = 1;
                        v20 = 6;
                        v21 = 5;
                        v30 = 9;
                        v31 = 7;
                    } else {
                        v00 = 5;
                        v10 = 6;
                        v20 = 4;
                        v21 = 6;
                        v30 = 3;
                        v31 = 10;
                    }
                }
                42 | 170 => {
                    v02 = 8;
                    v03 = 5;
                    v12 = 3;
                    v13 = 2;
                    v21 = 3;
                    v22 = 2;
                    v23 = 9;
                    v31 = 2;
                    v32 = 3;
                    v33 = 6;
                    if diff(w[4], w[2]) {
                        v00 = 6;
                        v01 = 7;
                        v10 = 9;
                        v11 = 7;
                        v20 = 1;
                        v30 = 1;
                    } else {
                        v00 = 3;
                        v01 = 10;
                        v10 = 4;
                        v11 = 10;
                        v20 = 6;
                        v30 = 10;
                    }
                }
                14 | 142 => {
                    v00 = 5;
                    v10 = 1;
                    v20 = 1;
                    v21 = 3;
                    v22 = 4;
                    v23 = 7;
                    v30 = 5;
                    v31 = 8;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[4], w[2]) {
                        v01 = 1;
                        v02 = 8;
                        v03 = 5;
                        v11 = 3;
                        v12 = 3;
                        v13 = 2;
                    } else {
                        v01 = 6;
                        v02 = 3;
                        v03 = 10;
                        v11 = 6;
                        v12 = 5;
                        v13 = 10;
                    }
                }
                67 => {
                    v00 = 6;
                    v01 = 5;
                    v02 = 2;
                    v03 = 5;
                    v10 = 9;
                    v11 = 1;
                    v12 = 3;
                    v13 = 8;
                    v20 = 8;
                    v21 = 3;
                    v22 = 4;
                    v23 = 8;
                    v30 = 5;
                    v31 = 2;
                    v32 = 2;
                    v33 = 5;
                }
                70 => {
                    v00 = 5;
                    v01 = 2;
                    v02 = 5;
                    v03 = 7;
                    v10 = 8;
                    v11 = 3;
                    v12 = 5;
                    v13 = 9;
                    v20 = 8;
                    v21 = 3;
                    v22 = 4;
                    v23 = 8;
                    v30 = 5;
                    v31 = 2;
                    v32 = 2;
                    v33 = 5;
                }
                28 => {
                    v00 = 5;
                    v01 = 8;
                    v02 = 8;
                    v03 = 5;
                    v10 = 2;
                    v11 = 3;
                    v12 = 3;
                    v13 = 2;
                    v20 = 1;
                    v21 = 3;
                    v22 = 4;
                    v23 = 2;
                    v30 = 5;
                    v31 = 8;
                    v32 = 8;
                    v33 = 5;
                }
                152 => {
                    v00 = 5;
                    v01 = 8;
                    v02 = 8;
                    v03 = 5;
                    v10 = 1;
                    v11 = 3;
                    v12 = 3;
                    v13 = 2;
                    v20 = 2;
                    v21 = 3;
                    v22 = 4;
                    v23 = 2;
                    v30 = 5;
                    v31 = 8;
                    v32 = 8;
                    v33 = 5;
                }
                194 => {
                    v00 = 5;
                    v01 = 1;
                    v02 = 1;
                    v03 = 5;
                    v10 = 8;
                    v11 = 3;
                    v12 = 3;
                    v13 = 8;
                    v20 = 8;
                    v21 = 3;
                    v22 = 2;
                    v23 = 9;
                    v30 = 5;
                    v31 = 2;
                    v32 = 3;
                    v33 = 6;
                }
                98 => {
                    v00 = 5;
                    v01 = 1;
                    v02 = 1;
                    v03 = 5;
                    v10 = 8;
                    v11 = 3;
                    v12 = 3;
                    v13 = 8;
                    v20 = 9;
                    v21 = 5;
                    v22 = 3;
                    v23 = 8;
                    v30 = 7;
                    v31 = 5;
                    v32 = 2;
                    v33 = 5;
                }
                56 => {
                    v00 = 5;
                    v01 = 8;
                    v02 = 8;
                    v03 = 5;
                    v10 = 2;
                    v11 = 3;
                    v12 = 3;
                    v13 = 2;
                    v20 = 6;
                    v21 = 0;
                    v22 = 0;
                    v23 = 6;
                    v30 = 6;
                    v31 = 6;
                    v32 = 6;
                    v33 = 6;
                }
                25 => {
                    v00 = 7;
                    v01 = 9;
                    v02 = 9;
                    v03 = 7;
                    v10 = 5;
                    v11 = 5;
                    v12 = 5;
                    v13 = 5;
                    v20 = 2;
                    v21 = 3;
                    v22 = 3;
                    v23 = 2;
                    v30 = 5;
                    v31 = 8;
                    v32 = 8;
                    v33 = 5;
                }
                26 | 31 => {
                    v20 = 1;
                    v21 = 3;
                    v22 = 4;
                    v23 = 2;
                    v30 = 5;
                    v31 = 8;
                    v32 = 8;
                    v33 = 5;
                    if diff(w[4], w[2]) {
                        v00 = 5;
                        v10 = 1;
                    } else {
                        v00 = 4;
                        v10 = 6;
                    }
                    v01 = 1;
                    v11 = 3;
                    if diff(w[2], w[6]) {
                        v03 = 5;
                        v13 = 2;
                    } else {
                        v03 = 4;
                        v13 = 3;
                    }
                    v02 = 1;
                    v12 = 3;
                }
                82 | 214 => {
                    v00 = 5;
                    v01 = 1;
                    v10 = 8;
                    v11 = 3;
                    v20 = 8;
                    v21 = 3;
                    v30 = 5;
                    v31 = 2;
                    if diff(w[2], w[6]) {
                        v02 = 1;
                        v03 = 5;
                        v12 = 3;
                        v13 = 8;
                    } else {
                        v02 = 6;
                        v03 = 10;
                        v12 = 0;
                        v13 = 6;
                    }
                    if diff(w[6], w[8]) {
                        v22 = 4;
                        v23 = 2;
                        v32 = 2;
                        v33 = 5;
                    } else {
                        v22 = 0;
                        v23 = 6;
                        v32 = 6;
                        v33 = 6;
                    }
                }
                88 | 248 => {
                    v00 = 5;
                    v01 = 8;
                    v02 = 8;
                    v03 = 5;
                    v11 = 3;
                    v12 = 3;
                    v13 = 2;
                    v21 = 3;
                    v23 = 2;
                    v31 = 8;
                    v32 = 8;
                    v33 = 5;
                    if diff(w[8], w[4]) {
                        v10 = 1;
                        v20 = 1;
                        v30 = 5;
                    } else {
                        v10 = 6;
                        v20 = 6;
                        v30 = 6;
                    }
                    if diff(w[6], w[8]) {
                        v22 = 4;
                    } else {
                        v22 = 0;
                    }
                }
                74 | 107 => {
                    v02 = 7;
                    v03 = 3;
                    v12 = 4;
                    v13 = 7;
                    v22 = 4;
                    v23 = 8;
                    v32 = 2;
                    v33 = 5;
                    if diff(w[4], w[2]) {
                        v00 = 5;
                        v01 = 1;
                        v10 = 1;
                        v11 = 3;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                    if diff(w[8], w[4]) {
                        v20 = 1;
                        v21 = 3;
                        v30 = 5;
                        v31 = 2;
                    } else {
                        v20 = 6;
                        v21 = 0;
                        v30 = 6;
                        v31 = 6;
                    }
                }
                27 => {
                    v02 = 7;
                    v03 = 3;
                    v12 = 4;
                    v13 = 7;
                    v20 = 1;
                    v21 = 3;
                    v22 = 4;
                    v23 = 2;
                    v30 = 5;
                    v31 = 8;
                    v32 = 8;
                    v33 = 5;
                    if diff(w[4], w[2]) {
                        v00 = 5;
                        v01 = 1;
                        v10 = 1;
                        v11 = 3;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                }
                86 => {
                    v00 = 5;
                    v01 = 1;
                    v10 = 8;
                    v11 = 3;
                    v20 = 8;
                    v21 = 3;
                    v22 = 4;
                    v23 = 8;
                    v30 = 5;
                    v31 = 2;
                    v32 = 2;
                    v33 = 5;
                    if diff(w[2], w[6]) {
                        v02 = 1;
                        v03 = 5;
                        v12 = 3;
                        v13 = 8;
                    } else {
                        v02 = 6;
                        v03 = 10;
                        v12 = 0;
                        v13 = 6;
                    }
                }
                216 => {
                    v00 = 5;
                    v01 = 8;
                    v02 = 8;
                    v03 = 5;
                    v10 = 1;
                    v11 = 3;
                    v12 = 3;
                    v13 = 2;
                    v20 = 1;
                    v21 = 3;
                    v30 = 5;
                    v31 = 8;
                    if diff(w[6], w[8]) {
                        v22 = 4;
                        v23 = 2;
                        v32 = 2;
                        v33 = 5;
                    } else {
                        v22 = 0;
                        v23 = 6;
                        v32 = 6;
                        v33 = 6;
                    }
                }
                106 => {
                    v01 = 1;
                    v02 = 1;
                    v03 = 5;
                    v11 = 3;
                    v12 = 3;
                    v13 = 8;
                    v21 = 3;
                    v22 = 4;
                    v23 = 7;
                    v31 = 8;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[8], w[4]) {
                        v00 = 5;
                        v10 = 1;
                        v20 = 1;
                        v30 = 5;
                    } else {
                        v00 = 4;
                        v10 = 6;
                        v20 = 6;
                        v30 = 6;
                    }
                }
                30 => {
                    v00 = 5;
                    v10 = 2;
                    v20 = 2;
                    v21 = 3;
                    v22 = 4;
                    v23 = 2;
                    v30 = 5;
                    v31 = 8;
                    v32 = 8;
                    v33 = 5;
                    if diff(w[2], w[6]) {
                        v01 = 8;
                        v02 = 8;
                        v03 = 5;
                        v11 = 3;
                        v12 = 3;
                        v13 = 2;
                    } else {
                        v01 = 10;
                        v02 = 10;
                        v03 = 10;
                        v11 = 6;
                        v12 = 0;
                        v13 = 6;
                    }
                }
                210 => {
                    v00 = 5;
                    v01 = 1;
                    v02 = 1;
                    v03 = 5;
                    v10 = 8;
                    v11 = 3;
                    v12 = 3;
                    v13 = 8;
                    v20 = 8;
                    v21 = 3;
                    v30 = 5;
                    v31 = 2;
                    if diff(w[6], w[8]) {
                        v22 = 4;
                        v23 = 2;
                        v32 = 2;
                        v33 = 5;
                    } else {
                        v22 = 0;
                        v23 = 6;
                        v32 = 6;
                        v33 = 6;
                    }
                }
                120 => {
                    v00 = 5;
                    v01 = 8;
                    v02 = 8;
                    v03 = 5;
                    v11 = 3;
                    v12 = 3;
                    v13 = 2;
                    v22 = 4;
                    v23 = 8;
                    v32 = 2;
                    v33 = 5;
                    if diff(w[8], w[4]) {
                        v10 = 1;
                        v20 = 1;
                        v21 = 3;
                        v30 = 5;
                        v31 = 2;
                    } else {
                        v10 = 6;
                        v20 = 6;
                        v21 = 0;
                        v30 = 6;
                        v31 = 6;
                    }
                }
                75 => {
                    v02 = 7;
                    v03 = 3;
                    v12 = 4;
                    v13 = 7;
                    v20 = 8;
                    v21 = 3;
                    v22 = 4;
                    v23 = 8;
                    v30 = 5;
                    v31 = 2;
                    v32 = 2;
                    v33 = 5;
                    if diff(w[4], w[2]) {
                        v00 = 5;
                        v01 = 1;
                        v10 = 1;
                        v11 = 3;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                }
                29 => {
                    v00 = 7;
                    v01 = 9;
                    v02 = 8;
                    v03 = 5;
                    v10 = 5;
                    v11 = 5;
                    v12 = 3;
                    v13 = 2;
                    v20 = 2;
                    v21 = 3;
                    v22 = 3;
                    v23 = 2;
                    v30 = 5;
                    v31 = 8;
                    v32 = 8;
                    v33 = 5;
                }
                198 => {
                    v00 = 5;
                    v01 = 1;
                    v02 = 5;
                    v03 = 7;
                    v10 = 8;
                    v11 = 3;
                    v12 = 5;
                    v13 = 9;
                    v20 = 8;
                    v21 = 3;
                    v22 = 2;
                    v23 = 9;
                    v30 = 5;
                    v31 = 2;
                    v32 = 3;
                    v33 = 6;
                }
                184 => {
                    v00 = 5;
                    v01 = 8;
                    v02 = 8;
                    v03 = 5;
                    v10 = 1;
                    v11 = 3;
                    v12 = 3;
                    v13 = 2;
                    v20 = 6;
                    v21 = 0;
                    v22 = 10;
                    v23 = 6;
                    v30 = 6;
                    v31 = 6;
                    v32 = 10;
                    v33 = 6;
                }
                99 => {
                    v00 = 6;
                    v01 = 5;
                    v02 = 1;
                    v03 = 5;
                    v10 = 9;
                    v11 = 1;
                    v12 = 3;
                    v13 = 8;
                    v20 = 9;
                    v21 = 5;
                    v22 = 3;
                    v23 = 8;
                    v30 = 7;
                    v31 = 5;
                    v32 = 2;
                    v33 = 5;
                }
                57 => {
                    v00 = 7;
                    v01 = 9;
                    v02 = 9;
                    v03 = 7;
                    v10 = 5;
                    v11 = 5;
                    v12 = 5;
                    v13 = 5;
                    v20 = 6;
                    v21 = 0;
                    v22 = 0;
                    v23 = 6;
                    v30 = 6;
                    v31 = 6;
                    v32 = 6;
                    v33 = 6;
                }
                71 => {
                    v00 = 6;
                    v01 = 5;
                    v02 = 5;
                    v03 = 7;
                    v10 = 9;
                    v11 = 1;
                    v12 = 5;
                    v13 = 9;
                    v20 = 8;
                    v21 = 3;
                    v22 = 4;
                    v23 = 8;
                    v30 = 5;
                    v31 = 2;
                    v32 = 2;
                    v33 = 5;
                }
                156 => {
                    v00 = 5;
                    v01 = 8;
                    v02 = 8;
                    v03 = 5;
                    v10 = 2;
                    v11 = 3;
                    v12 = 3;
                    v13 = 2;
                    v20 = 6;
                    v21 = 0;
                    v22 = 10;
                    v23 = 6;
                    v30 = 6;
                    v31 = 6;
                    v32 = 10;
                    v33 = 6;
                }
                226 => {
                    v00 = 5;
                    v01 = 1;
                    v02 = 1;
                    v03 = 5;
                    v10 = 8;
                    v11 = 3;
                    v12 = 3;
                    v13 = 8;
                    v20 = 9;
                    v21 = 5;
                    v22 = 2;
                    v23 = 9;
                    v30 = 7;
                    v31 = 5;
                    v32 = 3;
                    v33 = 6;
                }
                60 => {
                    v00 = 5;
                    v01 = 8;
                    v02 = 8;
                    v03 = 5;
                    v10 = 2;
                    v11 = 3;
                    v12 = 3;
                    v13 = 2;
                    v20 = 6;
                    v21 = 0;
                    v22 = 0;
                    v23 = 6;
                    v30 = 6;
                    v31 = 6;
                    v32 = 6;
                    v33 = 6;
                }
                195 => {
                    v00 = 6;
                    v01 = 5;
                    v02 = 2;
                    v03 = 5;
                    v10 = 9;
                    v11 = 1;
                    v12 = 3;
                    v13 = 8;
                    v20 = 8;
                    v21 = 3;
                    v22 = 2;
                    v23 = 9;
                    v30 = 5;
                    v31 = 2;
                    v32 = 3;
                    v33 = 6;
                }
                102 => {
                    v00 = 5;
                    v01 = 2;
                    v02 = 5;
                    v03 = 7;
                    v10 = 8;
                    v11 = 3;
                    v12 = 5;
                    v13 = 9;
                    v20 = 9;
                    v21 = 5;
                    v22 = 3;
                    v23 = 8;
                    v30 = 7;
                    v31 = 5;
                    v32 = 2;
                    v33 = 5;
                }
                153 => {
                    v00 = 7;
                    v01 = 9;
                    v02 = 9;
                    v03 = 7;
                    v10 = 5;
                    v11 = 5;
                    v12 = 5;
                    v13 = 5;
                    v20 = 6;
                    v21 = 0;
                    v22 = 10;
                    v23 = 6;
                    v30 = 6;
                    v31 = 6;
                    v32 = 10;
                    v33 = 6;
                }
                58 => {
                    v01 = 1;
                    v11 = 3;
                    v20 = 6;
                    v21 = 0;
                    v22 = 0;
                    v23 = 6;
                    v30 = 6;
                    v31 = 6;
                    v32 = 6;
                    v33 = 6;
                    v00 = if diff(w[4], w[2]) { 5 } else { 4 };
                    v03 = if diff(w[2], w[6]) { 5 } else { 4 };
                    v02 = 1;
                    v12 = 3;
                    v10 = 1;
                    v13 = 2;
                }
                83 => {
                    v00 = 6;
                    v01 = 5;
                    v10 = 9;
                    v11 = 1;
                    v20 = 8;
                    v21 = 3;
                    v30 = 5;
                    v31 = 2;
                    v32 = 2;
                    v33 = 5;
                    v02 = if diff(w[2], w[6]) { 1 } else { 4 };
                    v03 = 5;
                    v12 = 3;
                    v13 = 8;
                    v22 = 4;
                    v23 = 8;
                    if diff(w[6], w[8]) {
                        v23 = 2;
                        v33 = 5;
                    } else {
                        v23 = 6;
                        v33 = 6;
                    }
                }
                92 => {
                    v00 = 5;
                    v01 = 8;
                    v02 = 8;
                    v03 = 5;
                    v10 = 1;
                    v11 = 3;
                    v12 = 3;
                    v13 = 2;
                    v21 = 3;
                    v22 = 4;
                    v23 = 8;
                    v31 = 8;
                    v20 = if diff(w[8], w[4]) { 1 } else { 4 };
                    v30 = 5;
                    if diff(w[6], w[8]) {
                        v23 = 2;
                        v33 = 5;
                    } else {
                        v23 = 6;
                        v33 = 6;
                    }
                    v32 = 2;
                }
                202 => {
                    v01 = 1;
                    v02 = 1;
                    v03 = 5;
                    v11 = 3;
                    v12 = 3;
                    v13 = 8;
                    v21 = 3;
                    v22 = 4;
                    v23 = 7;
                    v31 = 8;
                    v32 = 7;
                    v33 = 3;
                    v00 = if diff(w[4], w[2]) { 5 } else { 4 };
                    v10 = 1;
                    v20 = if diff(w[8], w[4]) { 1 } else { 4 };
                    v30 = 5;
                }
                78 => {
                    v01 = 1;
                    v03 = 5;
                    v11 = 3;
                    v12 = 3;
                    v13 = 8;
                    v21 = 3;
                    v22 = 4;
                    v23 = 7;
                    v31 = 8;
                    v32 = 7;
                    v33 = 3;
                    v00 = if diff(w[4], w[2]) { 5 } else { 4 };
                    v02 = 1;
                    v10 = 1;
                    if diff(w[8], w[4]) {
                        v20 = 1;
                        v30 = 5;
                    } else {
                        v20 = 0;
                        v30 = 6;
                    }
                }
                154 => {
                    v01 = 1;
                    v11 = 3;
                    v21 = 3;
                    v30 = 5;
                    v31 = 8;
                    v32 = 8;
                    v33 = 5;
                    v00 = if diff(w[4], w[2]) { 5 } else { 4 };
                    v03 = if diff(w[2], w[6]) { 5 } else { 4 };
                    v10 = 1;
                    v20 = 2;
                    v13 = 8;
                    v23 = 2;
                    v31 = 8;
                    v32 = 8;
                    v02 = 1;
                    v12 = 3;
                    v22 = 4;
                }
                114 => {
                    v01 = 1;
                    v03 = 5;
                    v11 = 3;
                    v13 = 8;
                    v21 = 3;
                    v23 = 8;
                    v31 = 2;
                    v32 = 2;
                    v33 = 5;
                    v00 = 6;
                    v10 = 9;
                    v20 = 8;
                    v30 = 5;
                    v02 = if diff(w[2], w[6]) { 1 } else { 4 };
                    v12 = 3;
                    v22 = 4;
                }
                89 => {
                    v01 = 9;
                    v02 = 7;
                    v03 = 3;
                    v11 = 5;
                    v12 = 4;
                    v13 = 7;
                    v21 = 3;
                    v22 = 4;
                    v23 = 8;
                    v00 = 7;
                    v10 = 5;
                    v20 = if diff(w[8], w[4]) { 1 } else { 4 };
                    v30 = 5;
                    v31 = 2;
                    v32 = 2;
                    v33 = 5;
                }
                90 => {
                    v01 = 1;
                    v03 = 5;
                    v11 = 3;
                    v13 = 8;
                    v21 = 3;
                    v23 = 8;
                    v32 = 2;
                    v33 = 5;
                    v00 = if diff(w[4], w[2]) { 5 } else { 4 };
                    v02 = if diff(w[2], w[6]) { 1 } else { 4 };
                    v12 = 3;
                    v22 = 4;
                    v10 = 1;
                    v20 = if diff(w[8], w[4]) { 1 } else { 4 };
                    v30 = 5;
                    v31 = 2;
                }
                55 | 23 => {
                    v00 = 6;
                    v01 = 5;
                    v10 = 9;
                    v11 = 1;
                    v20 = 7;
                    v21 = 4;
                    v22 = 4;
                    v23 = 7;
                    v30 = 3;
                    v31 = 7;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[2], w[6]) {
                        v02 = 1;
                        v03 = 1;
                        v12 = 3;
                        v13 = 3;
                    } else {
                        v02 = 10;
                        v03 = 10;
                        v12 = 0;
                        v13 = 7;
                    }
                }
                182 | 150 => {
                    v00 = 5;
                    v01 = 1;
                    v10 = 8;
                    v11 = 3;
                    v20 = 7;
                    v21 = 4;
                    v22 = 4;
                    v23 = 7;
                    v30 = 3;
                    v31 = 7;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[2], w[6]) {
                        v02 = 1;
                        v03 = 1;
                        v12 = 3;
                        v13 = 3;
                    } else {
                        v02 = 0;
                        v03 = 6;
                        v12 = 6;
                        v13 = 6;
                    }
                }
                213 | 212 => {
                    v00 = 3;
                    v01 = 7;
                    v02 = 9;
                    v03 = 7;
                    v10 = 7;
                    v11 = 4;
                    v20 = 7;
                    v21 = 4;
                    v22 = 4;
                    v23 = 7;
                    v30 = 3;
                    v31 = 7;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[6], w[8]) {
                        v12 = 3;
                        v13 = 3;
                        v23 = 5;
                        v32 = 6;
                        v33 = 5;
                    } else {
                        v12 = 0;
                        v13 = 10;
                        v23 = 10;
                        v32 = 10;
                        v33 = 10;
                    }
                }
                241 | 240 => {
                    v00 = 3;
                    v01 = 7;
                    v02 = 7;
                    v03 = 3;
                    v10 = 7;
                    v11 = 4;
                    v12 = 4;
                    v13 = 7;
                    v20 = 6;
                    v21 = 5;
                    v30 = 6;
                    v31 = 5;
                    if diff(w[6], w[8]) {
                        v22 = 3;
                        v23 = 3;
                        v32 = 1;
                        v33 = 1;
                    } else {
                        v22 = 0;
                        v23 = 6;
                        v32 = 6;
                        v33 = 6;
                    }
                }
                236 | 232 => {
                    v00 = 5;
                    v01 = 8;
                    v02 = 7;
                    v03 = 3;
                    v11 = 3;
                    v12 = 4;
                    v13 = 7;
                    v22 = 4;
                    v23 = 7;
                    v32 = 7;
                    v33 = 3;
                    if (diff(w[8], w[4])) {
                        v10 = 1;
                        v20 = 1;
                        v21 = 3;
                        v30 = 1;
                        v31 = 1;
                    } else {
                        v10 = 0;
                        v20 = 0;
                        v21 = 0;
                        v30 = 0;
                        v31 = 0;
                    }
                }
                109 | 105 => {
                    v01 = 8;
                    v02 = 7;
                    v03 = 3;
                    v11 = 3;
                    v12 = 4;
                    v13 = 7;
                    v22 = 4;
                    v23 = 7;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[8], w[4]) {
                        v00 = 1;
                        v10 = 1;
                        v20 = 6;
                        v21 = 5;
                        v30 = 9;
                        v31 = 7;
                    } else {
                        v00 = 0;
                        v10 = 0;
                        v20 = 0;
                        v21 = 0;
                        v30 = 0;
                        v31 = 0;
                    }
                }
                171 | 43 => {
                    v02 = 8;
                    v03 = 5;
                    v12 = 3;
                    v13 = 2;
                    v21 = 3;
                    v22 = 2;
                    v23 = 9;
                    v31 = 2;
                    v32 = 3;
                    v33 = 6;
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 0;
                        v10 = 0;
                        v11 = 0;
                        v20 = 1;
                        v30 = 1;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                        v20 = 6;
                        v30 = 6;
                    }
                }
                143 | 15 => {
                    v00 = 5;
                    v10 = 1;
                    v20 = 1;
                    v21 = 3;
                    v22 = 4;
                    v23 = 7;
                    v30 = 5;
                    v31 = 8;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[4], w[2]) {
                        v01 = 0;
                        v02 = 0;
                        v03 = 0;
                        v11 = 0;
                        v12 = 0;
                        v13 = 0;
                    } else {
                        v01 = 6;
                        v02 = 3;
                        v03 = 10;
                        v11 = 6;
                        v12 = 5;
                        v13 = 10;
                    }
                }
                124 => {
                    v00 = 5;
                    v01 = 8;
                    v02 = 10;
                    v03 = 2;
                    v11 = 3;
                    v12 = 3;
                    v13 = 8;
                    v22 = 4;
                    v23 = 8;
                    v32 = 2;
                    v33 = 5;
                    if (diff(w[8], w[4])) {
                        v10 = 1;
                        v20 = 1;
                        v21 = 3;
                        v30 = 5;
                        v31 = 2;
                    } else {
                        v10 = 6;
                        v20 = 6;
                        v21 = 0;
                        v30 = 6;
                        v31 = 6;
                    }
                }
                203 => {
                    v02 = 1;
                    v03 = 5;
                    v12 = 3;
                    v13 = 8;
                    v20 = 1;
                    v21 = 3;
                    v22 = 4;
                    v30 = 5;
                    v31 = 8;
                    v32 = 7;
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 0;
                        v10 = 0;
                        v11 = 0;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                    if diff(w[6], w[8]) {
                        v23 = 7;
                        v33 = 3;
                    } else {
                        v23 = 5;
                        v33 = 7;
                    }
                }
                62 => {
                    v00 = 5;
                    v10 = 2;
                    v20 = 6;
                    v21 = 0;
                    v22 = 0;
                    v23 = 6;
                    v30 = 6;
                    v31 = 6;
                    v32 = 6;
                    v33 = 6;
                    if diff(w[2], w[6]) {
                        v01 = 8;
                        v02 = 8;
                        v03 = 5;
                        v11 = 3;
                        v12 = 3;
                        v13 = 2;
                    } else {
                        v01 = 10;
                        v02 = 10;
                        v03 = 10;
                        v11 = 6;
                        v12 = 0;
                        v13 = 6;
                    }
                }
                211 => {
                    v00 = 6;
                    v01 = 5;
                    v02 = 1;
                    v03 = 1;
                    v10 = 9;
                    v11 = 1;
                    v12 = 3;
                    v13 = 3;
                    v20 = 8;
                    v21 = 3;
                    v30 = 5;
                    v31 = 2;
                    if diff(w[6], w[8]) {
                        v22 = 4;
                        v23 = 0;
                        v32 = 2;
                        v33 = 0;
                    } else {
                        v22 = 0;
                        v23 = 6;
                        v32 = 6;
                        v33 = 6;
                    }
                }
                118 => {
                    v10 = 8;
                    v11 = 3;
                    v20 = 9;
                    v21 = 5;
                    v22 = 3;
                    v23 = 8;
                    v30 = 7;
                    v31 = 5;
                    v32 = 2;
                    v33 = 5;
                    v00 = 5;
                    v01 = 1;
                    if diff(w[2], w[6]) {
                        v02 = 1;
                        v03 = 1;
                        v12 = 3;
                        v13 = 3;
                    } else {
                        v02 = 10;
                        v03 = 10;
                        v12 = 0;
                        v13 = 6;
                    }
                }
                217 => {
                    v00 = 7;
                    v01 = 9;
                    v02 = 9;
                    v03 = 7;
                    v10 = 5;
                    v11 = 5;
                    v12 = 5;
                    v13 = 5;
                    v20 = 8;
                    v21 = 3;
                    v30 = 5;
                    v31 = 2;
                    if diff(w[6], w[8]) {
                        v22 = 4;
                        v23 = 0;
                        v32 = 2;
                        v33 = 0;
                    } else {
                        v22 = 0;
                        v23 = 6;
                        v32 = 6;
                        v33 = 6;
                    }
                }
                110 => {
                    v02 = 9;
                    v03 = 7;
                    v12 = 5;
                    v13 = 5;
                    v22 = 4;
                    v23 = 8;
                    v32 = 2;
                    v33 = 5;
                    if diff(w[8], w[4]) {
                        v20 = 0;
                        v21 = 0;
                        v30 = 0;
                        v31 = 0;
                    } else {
                        v20 = 6;
                        v21 = 0;
                        v30 = 6;
                        v31 = 6;
                    }
                    v01 = 0;
                    v11 = 0;
                    v00 = 5;
                    v10 = 1;
                }
                155 => {
                    v02 = 7;
                    v03 = 3;
                    v12 = 4;
                    v13 = 7;
                    v20 = 8;
                    v21 = 3;
                    v22 = 4;
                    v23 = 8;
                    v30 = 5;
                    v31 = 2;
                    v32 = 2;
                    v33 = 5;
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 0;
                        v10 = 0;
                        v11 = 0;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                }
                188 => {
                    v00 = 5;
                    v01 = 8;
                    v02 = 10;
                    v03 = 6;
                    v10 = 1;
                    v11 = 3;
                    v12 = 0;
                    v13 = 6;
                    v20 = 6;
                    v21 = 0;
                    v22 = 10;
                    v23 = 6;
                    v30 = 6;
                    v31 = 6;
                    v32 = 10;
                    v33 = 6;
                }
                185 => {
                    v00 = 7;
                    v01 = 9;
                    v02 = 10;
                    v03 = 6;
                    v10 = 5;
                    v11 = 5;
                    v12 = 10;
                    v13 = 6;
                    v20 = 6;
                    v21 = 0;
                    v22 = 10;
                    v23 = 6;
                    v30 = 6;
                    v31 = 6;
                    v32 = 10;
                    v33 = 6;
                }
                61 => {
                    v00 = 7;
                    v01 = 9;
                    v02 = 10;
                    v03 = 6;
                    v10 = 5;
                    v11 = 5;
                    v12 = 10;
                    v13 = 6;
                    v20 = 6;
                    v21 = 0;
                    v22 = 0;
                    v23 = 6;
                    v30 = 6;
                    v31 = 6;
                    v32 = 6;
                    v33 = 6;
                }
                157 => {
                    v00 = 7;
                    v01 = 9;
                    v02 = 10;
                    v03 = 6;
                    v10 = 5;
                    v11 = 5;
                    v12 = 10;
                    v13 = 6;
                    v20 = 2;
                    v21 = 3;
                    v22 = 10;
                    v23 = 6;
                    v30 = 5;
                    v31 = 8;
                    v32 = 10;
                    v33 = 6;
                }
                103 => {
                    v00 = 6;
                    v01 = 5;
                    v02 = 5;
                    v03 = 7;
                    v10 = 9;
                    v11 = 1;
                    v12 = 5;
                    v13 = 9;
                    v20 = 9;
                    v21 = 5;
                    v22 = 3;
                    v23 = 8;
                    v30 = 7;
                    v31 = 5;
                    v32 = 2;
                    v33 = 5;
                }
                227 => {
                    v00 = 6;
                    v01 = 5;
                    v02 = 2;
                    v03 = 5;
                    v10 = 9;
                    v11 = 1;
                    v12 = 3;
                    v13 = 8;
                    v20 = 9;
                    v21 = 5;
                    v22 = 2;
                    v23 = 9;
                    v30 = 7;
                    v31 = 5;
                    v32 = 3;
                    v33 = 6;
                }
                230 => {
                    v00 = 5;
                    v01 = 2;
                    v02 = 5;
                    v03 = 7;
                    v10 = 8;
                    v11 = 3;
                    v12 = 5;
                    v13 = 9;
                    v20 = 9;
                    v21 = 5;
                    v22 = 2;
                    v23 = 9;
                    v30 = 7;
                    v31 = 5;
                    v32 = 3;
                    v33 = 6;
                }
                199 => {
                    v00 = 6;
                    v01 = 5;
                    v02 = 5;
                    v03 = 7;
                    v10 = 9;
                    v11 = 1;
                    v12 = 5;
                    v13 = 9;
                    v20 = 8;
                    v21 = 3;
                    v22 = 2;
                    v23 = 9;
                    v30 = 5;
                    v31 = 2;
                    v32 = 3;
                    v33 = 6;
                }
                220 => {
                    v00 = 5;
                    v01 = 8;
                    v02 = 10;
                    v03 = 2;
                    v10 = 1;
                    v11 = 3;
                    v12 = 3;
                    v13 = 8;
                    v30 = 5;
                    v31 = 8;
                    if diff(w[8], w[4]) {
                        v20 = 1;
                    } else {
                        v20 = 6;
                    }
                    if diff(w[6], w[8]) {
                        v22 = 4;
                        v23 = 2;
                        v32 = 2;
                        v33 = 5;
                        v21 = 3;
                    } else {
                        v22 = 0;
                        v23 = 6;
                        v32 = 6;
                        v33 = 6;
                        v21 = 0;
                    }
                }
                158 => {
                    v10 = 1;
                    v11 = 3;
                    v20 = 1;
                    v21 = 3;
                    v22 = 4;
                    v23 = 8;
                    v30 = 5;
                    v31 = 8;
                    v32 = 8;
                    v33 = 5;
                    if diff(w[4], w[2]) {
                        v00 = 5;
                    } else {
                        v00 = 4;
                    }
                    if diff(w[2], w[6]) {
                        v01 = 8;
                        v02 = 8;
                        v03 = 5;
                        v13 = 2;
                        v12 = 3;
                    } else {
                        v01 = 10;
                        v02 = 10;
                        v03 = 10;
                        v13 = 6;
                        v12 = 0;
                    }
                }
                234 => {
                    v01 = 8;
                    v02 = 7;
                    v03 = 3;
                    v11 = 3;
                    v12 = 4;
                    v13 = 7;
                    v22 = 5;
                    v23 = 9;
                    v32 = 6;
                    v33 = 7;
                    if diff(w[4], w[2]) {
                        v00 = 5;
                        v10 = 1;
                    } else {
                        v00 = 4;
                        v10 = 6;
                    }
                    if diff(w[8], w[4]) {
                        v20 = 1;
                        v21 = 3;
                        v30 = 1;
                        v31 = 1;
                    } else {
                        v20 = 5;
                        v21 = 6;
                        v30 = 5;
                        v31 = 10;
                    }
                }
                242 => {
                    v00 = 5;
                    v01 = 1;
                    v10 = 8;
                    v11 = 3;
                    v20 = 8;
                    v21 = 3;
                    v30 = 5;
                    v31 = 2;
                    if diff(w[2], w[6]) {
                        v02 = 1;
                        v03 = 5;
                        v12 = 3;
                        v13 = 8;
                    } else {
                        v02 = 6;
                        v03 = 10;
                        v12 = 0;
                        v13 = 6;
                    }
                    if diff(w[6], w[8]) {
                        v22 = 3;
                        v23 = 3;
                        v32 = 1;
                        v33 = 1;
                    } else {
                        v22 = 5;
                        v23 = 10;
                        v32 = 5;
                        v33 = 10;
                    }
                }
                59 => {
                    v20 = 7;
                    v21 = 4;
                    v22 = 4;
                    v23 = 7;
                    v30 = 3;
                    v31 = 7;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 0;
                        v10 = 0;
                        v11 = 0;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                    if diff(w[2], w[6]) {
                        v02 = 9;
                        v03 = 7;
                        v12 = 5;
                        v13 = 5;
                    } else {
                        v02 = 10;
                        v03 = 7;
                        v12 = 0;
                        v13 = 7;
                    }
                }
                121 => {
                    v00 = 7;
                    v01 = 9;
                    v02 = 7;
                    v03 = 3;
                    v10 = 5;
                    v11 = 5;
                    v12 = 4;
                    v13 = 7;
                    v33 = 3;
                    if diff(w[8], w[4]) {
                        v20 = 1;
                        v21 = 3;
                        v30 = 9;
                        v31 = 7;
                    } else {
                        v20 = 6;
                        v21 = 0;
                        v30 = 3;
                        v31 = 10;
                    }
                    if diff(w[6], w[8]) {
                        v22 = 4;
                        v23 = 8;
                        v32 = 7;
                    } else {
                        v22 = 0;
                        v23 = 7;
                        v32 = 7;
                    }
                }
                87 => {
                    v00 = 6;
                    v01 = 5;
                    v10 = 9;
                    v11 = 1;
                    v20 = 8;
                    v21 = 3;
                    v30 = 5;
                    v31 = 2;
                    if diff(w[2], w[6]) {
                        v02 = 10;
                        v03 = 2;
                        v12 = 3;
                        v13 = 8;
                    } else {
                        v02 = 6;
                        v03 = 10;
                        v12 = 0;
                        v13 = 6;
                    }
                    if diff(w[6], w[8]) {
                        v22 = 4;
                        v23 = 8;
                        v32 = 7;
                        v33 = 3;
                    } else {
                        v22 = 0;
                        v23 = 7;
                        v32 = 7;
                        v33 = 3;
                    }
                }
                79 => {
                    v02 = 6;
                    v03 = 5;
                    v12 = 2;
                    v13 = 3;
                    v21 = 3;
                    v22 = 4;
                    v23 = 7;
                    v31 = 8;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 0;
                        v10 = 0;
                        v11 = 0;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                    if diff(w[8], w[4]) {
                        v20 = 1;
                        v30 = 9;
                    } else {
                        v20 = 6;
                        v30 = 3;
                    }
                }
                122 => {
                    v01 = 1;
                    v11 = 3;
                    v33 = 3;
                    if diff(w[4], w[2]) {
                        v00 = 5;
                        v10 = 1;
                    } else {
                        v00 = 4;
                        v10 = 6;
                    }
                    if diff(w[2], w[6]) {
                        v02 = 9;
                        v03 = 7;
                        v12 = 5;
                        v13 = 5;
                    } else {
                        v02 = 10;
                        v03 = 7;
                        v12 = 0;
                        v13 = 7;
                    }
                    if diff(w[8], w[4]) {
                        v20 = 1;
                        v21 = 3;
                        v30 = 9;
                        v31 = 7;
                    } else {
                        v20 = 6;
                        v21 = 0;
                        v30 = 3;
                        v31 = 10;
                    }
                    if diff(w[6], w[8]) {
                        v22 = 4;
                        v23 = 8;
                        v32 = 7;
                    } else {
                        v22 = 0;
                        v23 = 7;
                        v32 = 7;
                    }
                }
                94 => {
                    v01 = 1;
                    v11 = 3;
                    v22 = 4;
                    v23 = 8;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[4], w[2]) {
                        v00 = 5;
                        v10 = 1;
                    } else {
                        v00 = 4;
                        v10 = 6;
                    }
                    if diff(w[2], w[6]) {
                        v02 = 10;
                        v03 = 2;
                        v12 = 3;
                        v13 = 8;
                    } else {
                        v02 = 6;
                        v03 = 10;
                        v12 = 0;
                        v13 = 6;
                    }
                    if diff(w[6], w[8]) {
                        v21 = 3;
                        v30 = 5;
                        v31 = 2;
                    } else {
                        v21 = 0;
                        v30 = 5;
                        v31 = 2;
                    }
                    v20 = 2;
                }
                218 => {
                    v01 = 1;
                    v11 = 3;
                    v20 = 2;
                    v30 = 5;
                    v31 = 2;
                    v32 = 3;
                    v33 = 6;
                    if diff(w[4], w[2]) {
                        v00 = 5;
                        v10 = 1;
                    } else {
                        v00 = 4;
                        v10 = 6;
                    }
                    if diff(w[2], w[6]) {
                        v02 = 9;
                        v03 = 7;
                        v12 = 5;
                        v13 = 5;
                    } else {
                        v02 = 10;
                        v03 = 7;
                        v12 = 0;
                        v13 = 7;
                    }
                    if diff(w[6], w[8]) {
                        v21 = 3;
                        v22 = 2;
                        v23 = 9;
                    } else {
                        v21 = 0;
                        v22 = 0;
                        v23 = 6;
                    }
                }
                91 => {
                    v02 = 9;
                    v03 = 7;
                    v12 = 5;
                    v13 = 5;
                    v20 = 2;
                    v33 = 6;
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 0;
                        v10 = 0;
                        v11 = 0;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                    if diff(w[6], w[8]) {
                        v21 = 3;
                        v22 = 2;
                        v23 = 1;
                        v31 = 2;
                        v32 = 3;
                        v30 = 5;
                    } else {
                        v21 = 0;
                        v22 = 0;
                        v23 = 6;
                        v31 = 2;
                        v32 = 2;
                        v30 = 5;
                    }
                }
                229 => {
                    v00 = 0;
                    v01 = 0;
                    v02 = 1;
                    v03 = 1;
                    v10 = 0;
                    v11 = 0;
                    v12 = 3;
                    v13 = 3;
                    v20 = 1;
                    v21 = 1;
                    v22 = 2;
                    v23 = 2;
                    v30 = 9;
                    v31 = 7;
                    v32 = 7;
                    v33 = 3;
                }
                167 => {
                    v00 = 6;
                    v01 = 5;
                    v02 = 5;
                    v03 = 7;
                    v10 = 9;
                    v11 = 1;
                    v12 = 5;
                    v13 = 9;
                    v20 = 0;
                    v21 = 0;
                    v22 = 0;
                    v23 = 0;
                    v30 = 0;
                    v31 = 0;
                    v32 = 0;
                    v33 = 0;
                }
                173 => {
                    v00 = 6;
                    v01 = 7;
                    v02 = 10;
                    v03 = 6;
                    v10 = 9;
                    v11 = 7;
                    v12 = 10;
                    v13 = 6;
                    v20 = 1;
                    v21 = 3;
                    v22 = 10;
                    v23 = 6;
                    v30 = 1;
                    v31 = 2;
                    v32 = 3;
                    v33 = 0;
                }
                181 => {
                    v00 = 0;
                    v01 = 1;
                    v02 = 10;
                    v03 = 6;
                    v10 = 0;
                    v11 = 1;
                    v12 = 10;
                    v13 = 6;
                    v20 = 6;
                    v21 = 7;
                    v22 = 2;
                    v23 = 9;
                    v30 = 9;
                    v31 = 7;
                    v32 = 3;
                    v33 = 6;
                }
                186 => {
                    v01 = 1;
                    v11 = 3;
                    v21 = 3;
                    v31 = 8;
                    v32 = 10;
                    v33 = 6;
                    v00 = if diff(w[4], w[2]) { 5 } else { 4 };
                    v03 = if diff(w[2], w[6]) { 10 } else { 4 };
                    v02 = 1;
                    v12 = 0;
                    v10 = 1;
                    v13 = 6;
                    v20 = 6;
                    v22 = 10;
                    v23 = 6;
                    v30 = 6;
                }
                115 => {
                    v01 = 5;
                    v11 = 1;
                    v21 = 5;
                    v31 = 5;
                    v32 = 2;
                    v33 = 5;
                    v00 = 6;
                    v10 = 9;
                    v20 = 9;
                    v30 = 7;
                    v02 = if diff(w[2], w[6]) { 5 } else { 4 };
                    v03 = 7;
                    v12 = 5;
                    v13 = 9;
                    v22 = 3;
                    v23 = 8;
                }
                93 => {
                    v01 = 9;
                    v02 = 10;
                    v03 = 6;
                    v11 = 5;
                    v12 = 10;
                    v13 = 6;
                    v21 = 5;
                    v22 = 10;
                    v23 = 6;
                    v00 = 7;
                    v10 = 5;
                    v20 = if diff(w[8], w[4]) { 1 } else { 4 };
                    v30 = 5;
                    v31 = 2;
                    v32 = 2;
                    v33 = 5;
                }
                206 => {
                    v01 = 1;
                    v03 = 5;
                    v11 = 3;
                    v13 = 8;
                    v21 = 3;
                    v23 = 8;
                    v31 = 8;
                    v33 = 10;
                    v00 = if diff(w[4], w[2]) { 5 } else { 4 };
                    v02 = 1;
                    v10 = 1;
                    v20 = 1;
                    v30 = 5;
                    v20 = if diff(w[8], w[4]) { 1 } else { 4 };
                    v22 = 4;
                    v32 = 7;
                }
                205 | 201 => {
                    v00 = 7;
                    v01 = 9;
                    v02 = 9;
                    v03 = 7;
                    v10 = 5;
                    v11 = 5;
                    v12 = 5;
                    v13 = 5;
                    v21 = 0;
                    v22 = 4;
                    v23 = 7;
                    v31 = 2;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[8], w[4]) {
                        v20 = 1;
                        v30 = 1;
                    } else {
                        v20 = 6;
                        v30 = 6;
                    }
                }
                174 | 46 => {
                    v02 = 10;
                    v03 = 6;
                    v12 = 10;
                    v13 = 6;
                    v21 = 3;
                    v22 = 4;
                    v23 = 7;
                    v31 = 8;
                    v32 = 7;
                    if diff(w[4], w[2]) {
                        v00 = 5;
                        v01 = 1;
                        v10 = 1;
                        v11 = 3;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                    v20 = 2;
                    v30 = 6;
                    v33 = 0;
                }
                179 | 147 => {
                    v00 = 6;
                    v01 = 5;
                    v10 = 9;
                    v11 = 1;
                    v20 = 0;
                    v21 = 0;
                    v22 = 0;
                    v23 = 0;
                    v30 = 6;
                    v31 = 0;
                    v32 = 10;
                    v33 = 6;
                    if diff(w[2], w[6]) {
                        v02 = 8;
                        v03 = 5;
                        v12 = 3;
                        v13 = 2;
                    } else {
                        v02 = 10;
                        v03 = 10;
                        v12 = 0;
                        v13 = 0;
                    }
                }
                117 | 116 => {
                    v00 = 0;
                    v01 = 0;
                    v02 = 1;
                    v03 = 1;
                    v10 = 0;
                    v11 = 0;
                    v12 = 3;
                    v13 = 3;
                    v20 = 6;
                    v21 = 5;
                    v30 = 6;
                    v31 = 5;
                    if diff(w[6], w[8]) {
                        v22 = 3;
                        v23 = 3;
                        v32 = 1;
                        v33 = 1;
                    } else {
                        v22 = 0;
                        v23 = 7;
                        v32 = 7;
                        v33 = 10;
                    }
                }
                189 => {
                    v00 = 7;
                    v01 = 9;
                    v02 = 10;
                    v03 = 6;
                    v10 = 5;
                    v11 = 5;
                    v12 = 10;
                    v13 = 6;
                    v20 = 7;
                    v21 = 5;
                    v22 = 10;
                    v23 = 6;
                    v30 = 7;
                    v31 = 5;
                    v32 = 10;
                    v33 = 6;
                }
                231 => {
                    v00 = 6;
                    v01 = 5;
                    v02 = 5;
                    v03 = 7;
                    v10 = 9;
                    v11 = 1;
                    v12 = 5;
                    v13 = 9;
                    v20 = 9;
                    v21 = 5;
                    v22 = 2;
                    v23 = 9;
                    v30 = 7;
                    v31 = 5;
                    v32 = 3;
                    v33 = 6;
                }
                126 => {
                    v00 = 5;
                    v01 = 1;
                    v10 = 8;
                    v11 = 3;
                    v21 = 0;
                    v33 = 3;
                    if diff(w[2], w[6]) {
                        v02 = 1;
                        v03 = 5;
                        v12 = 3;
                        v13 = 8;
                    } else {
                        v02 = 6;
                        v03 = 10;
                        v12 = 0;
                        v13 = 6;
                    }
                    if diff(w[8], w[4]) {
                        v20 = 1;
                        v30 = 5;
                    } else {
                        v20 = 6;
                        v30 = 6;
                    }
                    v22 = 4;
                    v23 = 8;
                    v31 = 8;
                    v32 = 7;
                }
                219 => {
                    v02 = 1;
                    v03 = 5;
                    v12 = 3;
                    v13 = 8;
                    v20 = 8;
                    v21 = 3;
                    v30 = 5;
                    v31 = 2;
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 0;
                        v10 = 0;
                        v11 = 0;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                    if diff(w[6], w[8]) {
                        v22 = 4;
                        v23 = 2;
                        v32 = 2;
                        v33 = 5;
                    } else {
                        v22 = 0;
                        v23 = 6;
                        v32 = 6;
                        v33 = 6;
                    }
                }
                125 => {
                    v01 = 3;
                    v02 = 1;
                    v03 = 5;
                    v11 = 3;
                    v12 = 3;
                    v13 = 8;
                    v22 = 4;
                    v23 = 7;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[8], w[4]) {
                        v00 = 7;
                        v10 = 5;
                        v20 = 6;
                        v21 = 5;
                        v30 = 9;
                        v31 = 7;
                    } else {
                        v00 = 10;
                        v10 = 0;
                        v20 = 0;
                        v21 = 0;
                        v30 = 0;
                        v31 = 0;
                    }
                }
                221 => {
                    v00 = 7;
                    v01 = 3;
                    v02 = 1;
                    v03 = 1;
                    v10 = 5;
                    v11 = 3;
                    v12 = 3;
                    v13 = 3;
                    v20 = 8;
                    v21 = 3;
                    v30 = 5;
                    v31 = 2;
                    if diff(w[6], w[8]) {
                        v22 = 4;
                        v23 = 2;
                        v32 = 2;
                        v33 = 5;
                    } else {
                        v22 = 0;
                        v23 = 7;
                        v32 = 7;
                        v33 = 10;
                    }
                }
                207 => {
                    v02 = 9;
                    v03 = 7;
                    v12 = 5;
                    v13 = 5;
                    v21 = 3;
                    v22 = 4;
                    v23 = 7;
                    v31 = 8;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 0;
                        v10 = 0;
                        v11 = 0;
                        v20 = 1;
                        v30 = 1;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                        v20 = 6;
                        v30 = 6;
                    }
                }
                238 => {
                    v00 = 5;
                    v01 = 3;
                    v02 = 1;
                    v03 = 5;
                    v10 = 1;
                    v11 = 1;
                    v12 = 3;
                    v13 = 8;
                    v03 = 5;
                    v13 = 8;
                    v22 = 4;
                    v23 = 7;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[8], w[4]) {
                        v20 = 0;
                        v21 = 0;
                        v30 = 0;
                        v31 = 0;
                    } else {
                        v20 = 10;
                        v21 = 6;
                        v30 = 5;
                        v31 = 10;
                    }
                }
                190 => {
                    v00 = 5;
                    v01 = 1;
                    v10 = 8;
                    v11 = 3;
                    v20 = 8;
                    v21 = 3;
                    v30 = 5;
                    v31 = 2;
                    if diff(w[2], w[6]) {
                        v02 = 1;
                        v03 = 1;
                        v12 = 3;
                        v13 = 3;
                        v32 = 3;
                        v33 = 6;
                    } else {
                        v02 = 0;
                        v03 = 6;
                        v12 = 6;
                        v13 = 6;
                        v32 = 10;
                        v33 = 10;
                    }
                }
                187 => {
                    v01 = 1;
                    v02 = 1;
                    v03 = 5;
                    v11 = 3;
                    v12 = 3;
                    v13 = 8;
                    v22 = 4;
                    v23 = 7;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v10 = 0;
                        v20 = 1;
                        v30 = 1;
                    } else {
                        v00 = 4;
                        v10 = 6;
                        v20 = 6;
                        v30 = 6;
                    }
                    if diff(w[2], w[6]) {
                        v21 = 3;
                        v31 = 8;
                    } else {
                        v21 = 0;
                        v31 = 6;
                    }
                }
                243 => {
                    v00 = 6;
                    v01 = 5;
                    v10 = 9;
                    v11 = 1;
                    v20 = 7;
                    v21 = 4;
                    v22 = 4;
                    v23 = 7;
                    v30 = 3;
                    v31 = 7;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[6], w[8]) {
                        v02 = 1;
                        v03 = 1;
                        v12 = 3;
                        v13 = 3;
                    } else {
                        v02 = 0;
                        v03 = 7;
                        v12 = 7;
                        v13 = 10;
                    }
                }
                119 => {
                    v00 = 6;
                    v01 = 2;
                    v02 = 1;
                    v03 = 1;
                    v10 = 9;
                    v11 = 5;
                    v12 = 3;
                    v13 = 3;
                    v20 = 8;
                    v21 = 3;
                    v22 = 2;
                    v23 = 2;
                    v30 = 5;
                    v31 = 2;
                    v32 = 1;
                    v33 = 1;
                    if diff(w[2], w[6]) {
                    } else {
                        v02 = 4;
                        v12 = 3;
                        v03 = 4;
                        v13 = 3;
                    }
                    // Wait, 119 in hq4x.cpp has more complex nested if.
                }
                237 | 233 => {
                    v00 = 7;
                    v01 = 9;
                    v02 = 9;
                    v03 = 7;
                    v10 = 5;
                    v11 = 5;
                    v12 = 5;
                    v13 = 5;
                    v21 = 0;
                    v22 = 0;
                    v23 = 6;
                    v31 = 0;
                    v32 = 6;
                    v33 = 3;
                    if diff(w[8], w[4]) {
                        v20 = 0;
                        v30 = 0;
                    } else {
                        v20 = 6;
                        v30 = 6;
                    }
                }
                175 | 47 => {
                    v02 = 10;
                    v03 = 6;
                    v12 = 10;
                    v13 = 6;
                    v21 = 3;
                    v22 = 0;
                    v23 = 6;
                    v31 = 0;
                    v32 = 6;
                    v33 = 0;
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 0;
                        v10 = 0;
                        v11 = 0;
                        v20 = 2;
                        v30 = 6;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                        v20 = 6;
                        v30 = 6;
                    }
                }
                183 | 151 => {
                    v00 = 6;
                    v01 = 5;
                    v10 = 9;
                    v11 = 1;
                    v20 = 0;
                    v21 = 0;
                    v22 = 0;
                    v23 = 0;
                    v30 = 6;
                    v31 = 0;
                    v32 = 10;
                    v33 = 6;
                    if diff(w[2], w[6]) {
                        v02 = 0;
                        v03 = 0;
                        v12 = 0;
                        v13 = 0;
                    } else {
                        v02 = 10;
                        v03 = 10;
                        v12 = 0;
                        v13 = 0;
                    }
                }
                245 | 244 => {
                    v00 = 0;
                    v01 = 0;
                    v02 = 1;
                    v03 = 1;
                    v10 = 0;
                    v11 = 0;
                    v12 = 3;
                    v13 = 3;
                    v20 = 6;
                    v21 = 5;
                    v30 = 6;
                    v31 = 5;
                    if diff(w[6], w[8]) {
                        v22 = 0;
                        v23 = 0;
                        v32 = 0;
                        v33 = 0;
                    } else {
                        v22 = 0;
                        v23 = 7;
                        v32 = 7;
                        v33 = 10;
                    }
                }
                250 => {
                    v00 = 5;
                    v01 = 8;
                    v02 = 8;
                    v03 = 5;
                    v10 = 1;
                    v11 = 3;
                    v12 = 3;
                    v13 = 2;
                    v20 = 1;
                    v21 = 3;
                    v22 = 3;
                    v23 = 2;
                    v30 = 5;
                    v31 = 8;
                    v32 = 8;
                    v33 = 5;
                    if diff(w[8], w[4]) {
                        v10 = 0;
                        v20 = 0;
                        v30 = 0;
                        v31 = 0;
                        v21 = 0;
                        v11 = 0;
                    } else {
                        v10 = 6;
                        v20 = 6;
                        v30 = 6;
                        v31 = 6;
                        v21 = 3;
                        v11 = 3;
                    }
                }
                123 => {
                    v01 = 3;
                    v02 = 1;
                    v03 = 5;
                    v11 = 3;
                    v12 = 3;
                    v13 = 8;
                    v22 = 4;
                    v23 = 7;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 0;
                        v10 = 0;
                        v11 = 0;
                    } else {
                        v00 = 10;
                        v10 = 0;
                        v01 = 3;
                        v11 = 3;
                    }
                    if diff(w[8], w[4]) {
                        v20 = 0;
                        v21 = 0;
                        v30 = 0;
                        v31 = 0;
                    } else {
                        v20 = 6;
                        v21 = 6;
                        v30 = 10;
                        v31 = 7;
                    }
                }
                95 => {
                    v20 = 2;
                    v21 = 3;
                    v22 = 3;
                    v23 = 2;
                    v30 = 5;
                    v31 = 8;
                    v32 = 8;
                    v33 = 5;
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 0;
                        v10 = 0;
                        v11 = 0;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                    if diff(w[2], w[6]) {
                        v02 = 0;
                        v03 = 0;
                        v12 = 0;
                        v13 = 0;
                    } else {
                        v02 = 3;
                        v03 = 10;
                        v12 = 5;
                        v13 = 10;
                    }
                }
                222 => {
                    v00 = 5;
                    v01 = 1;
                    v10 = 8;
                    v11 = 3;
                    v20 = 8;
                    v21 = 3;
                    v30 = 5;
                    v31 = 2;
                    if diff(w[2], w[6]) {
                        v02 = 0;
                        v03 = 0;
                        v12 = 0;
                        v13 = 0;
                    } else {
                        v02 = 6;
                        v03 = 10;
                        v12 = 0;
                        v13 = 6;
                    }
                    if diff(w[6], w[8]) {
                        v22 = 0;
                        v23 = 0;
                        v32 = 0;
                        v33 = 0;
                    } else {
                        v22 = 0;
                        v23 = 6;
                        v32 = 6;
                        v33 = 6;
                    }
                }
                252 => {
                    v00 = 6;
                    v01 = 6;
                    v02 = 0;
                    v03 = 0;
                    v10 = 6;
                    v11 = 0;
                    v12 = 0;
                    v13 = 0;
                    if diff(w[8], w[4]) {
                        v20 = 0;
                        v21 = 0;
                        v30 = 0;
                        v31 = 0;
                    } else {
                        v20 = 4;
                        v21 = 6;
                        v30 = 6;
                        v31 = 6;
                    }
                    if diff(w[6], w[8]) {
                        v22 = 0;
                        v23 = 0;
                        v32 = 0;
                        v33 = 0;
                    } else {
                        v22 = 6;
                        v23 = 4;
                        v32 = 6;
                        v33 = 6;
                    }
                }
                249 => {
                    v00 = 7;
                    v01 = 9;
                    v02 = 9;
                    v03 = 7;
                    v10 = 5;
                    v11 = 5;
                    v12 = 5;
                    v13 = 5;
                    if diff(w[8], w[4]) {
                        v20 = 0;
                        v21 = 0;
                        v30 = 0;
                        v31 = 0;
                    } else {
                        v20 = 6;
                        v21 = 0;
                        v30 = 6;
                        v31 = 6;
                    }
                    if diff(w[6], w[8]) {
                        v22 = 0;
                        v23 = 0;
                        v32 = 0;
                        v33 = 0;
                    } else {
                        v22 = 0;
                        v23 = 7;
                        v32 = 7;
                        v33 = 10;
                    }
                }
                235 => {
                    v02 = 6;
                    v03 = 5;
                    v12 = 2;
                    v13 = 3;
                    v33 = 0;
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 0;
                        v10 = 0;
                        v11 = 0;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                    if diff(w[8], w[4]) {
                        v20 = 0;
                        v21 = 0;
                        v30 = 0;
                        v31 = 0;
                        v22 = 0;
                        v23 = 0;
                        v32 = 0;
                    } else {
                        v20 = 5;
                        v21 = 6;
                        v30 = 5;
                        v31 = 10;
                        v22 = 0;
                        v23 = 5;
                        v32 = 6;
                    }
                }
                111 => {
                    v20 = 2;
                    v21 = 3;
                    v22 = 4;
                    v23 = 8;
                    v30 = 5;
                    v31 = 8;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 0;
                        v10 = 0;
                        v11 = 0;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                    if diff(w[2], w[6]) {
                        v02 = 6;
                        v03 = 5;
                        v12 = 2;
                        v13 = 3;
                    } else {
                        v02 = 5;
                        v03 = 7;
                        v12 = 5;
                        v13 = 9;
                    }
                }
                63 => {
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 0;
                        v10 = 0;
                        v11 = 0;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                    if diff(w[2], w[6]) {
                        v02 = 0;
                        v03 = 0;
                        v12 = 0;
                        v13 = 0;
                    } else {
                        v02 = 3;
                        v03 = 10;
                        v12 = 5;
                        v13 = 10;
                    }
                    v20 = 6;
                    v21 = 6;
                    v22 = 0;
                    v23 = 0;
                    v30 = 6;
                    v31 = 6;
                    v32 = 0;
                    v33 = 0;
                }
                159 => {
                    v20 = 2;
                    v21 = 3;
                    v22 = 3;
                    v23 = 2;
                    v30 = 5;
                    v31 = 8;
                    v32 = 8;
                    v33 = 5;
                    if diff(w[4], w[2]) {
                        v00 = 6;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                    if diff(w[2], w[6]) {
                        v02 = 0;
                        v03 = 0;
                        v12 = 0;
                        v13 = 0;
                    } else {
                        v02 = 3;
                        v03 = 10;
                        v12 = 5;
                        v13 = 10;
                    }
                }
                215 => {
                    v00 = 6;
                    v01 = 6;
                    v10 = 6;
                    v11 = 0;
                    v20 = 8;
                    v21 = 3;
                    v30 = 5;
                    v31 = 2;
                    if diff(w[2], w[6]) {
                        v02 = 0;
                        v03 = 0;
                        v12 = 0;
                        v13 = 0;
                    } else {
                        v02 = 6;
                        v03 = 10;
                        v12 = 0;
                        v13 = 6;
                    }
                    if diff(w[6], w[8]) {
                        v22 = 0;
                        v23 = 0;
                        v32 = 0;
                        v33 = 0;
                    } else {
                        v22 = 0;
                        v23 = 10;
                        v32 = 10;
                        v33 = 10;
                    }
                }
                // Continuing patterns...
                246 => {
                    v00 = 6;
                    v01 = 5;
                    v10 = 9;
                    v11 = 1;
                    v20 = 8;
                    v21 = 3;
                    v30 = 5;
                    v31 = 2;
                    if diff(w[2], w[6]) {
                        v02 = 0;
                        v03 = 0;
                        v12 = 0;
                        v13 = 0;
                    } else {
                        v02 = 6;
                        v03 = 10;
                        v12 = 0;
                        v13 = 6;
                    }
                    if diff(w[6], w[8]) {
                        v22 = 0;
                        v23 = 0;
                        v32 = 0;
                        v33 = 0;
                    } else {
                        v22 = 0;
                        v23 = 10;
                        v32 = 10;
                        v33 = 10;
                    }
                }
                254 => {
                    v00 = 5;
                    v01 = 1;
                    v10 = 8;
                    v11 = 3;
                    v21 = 0;
                    if diff(w[2], w[6]) {
                        v02 = 0;
                        v03 = 0;
                        v12 = 0;
                        v13 = 0;
                    } else {
                        v02 = 6;
                        v03 = 10;
                        v12 = 0;
                        v13 = 6;
                    }
                    if diff(w[8], w[4]) {
                        v20 = 0;
                        v30 = 0;
                        v31 = 0;
                        v32 = 0;
                    } else {
                        v20 = 6;
                        v30 = 5;
                        v31 = 10;
                        v32 = 6;
                    }
                    if diff(w[6], w[8]) {
                        v22 = 0;
                        v23 = 0;
                        v33 = 0;
                    } else {
                        v22 = 0;
                        v23 = 6;
                        v33 = 6;
                    }
                }
                253 => {
                    v00 = 7;
                    v01 = 3;
                    v02 = 1;
                    v03 = 1;
                    v11 = 3;
                    v12 = 3;
                    v13 = 3;
                    v21 = 0;
                    if diff(w[8], w[4]) {
                        v10 = 0;
                        v20 = 0;
                        v30 = 0;
                        v31 = 0;
                    } else {
                        v10 = 5;
                        v20 = 6;
                        v30 = 9;
                        v31 = 7;
                    }
                    if diff(w[6], w[8]) {
                        v22 = 0;
                        v23 = 0;
                        v32 = 0;
                        v33 = 0;
                    } else {
                        v22 = 0;
                        v23 = 7;
                        v32 = 7;
                        v33 = 10;
                    }
                }
                251 => {
                    v01 = 3;
                    v02 = 1;
                    v03 = 5;
                    v11 = 3;
                    v12 = 3;
                    v13 = 8;
                    v22 = 4;
                    v23 = 7;
                    v32 = 7;
                    v33 = 3;
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 0;
                        v10 = 0;
                        v11 = 0;
                    } else {
                        v00 = 10;
                        v10 = 0;
                        v01 = 3;
                        v11 = 3;
                    }
                    if diff(w[8], w[4]) {
                        v20 = 0;
                        v21 = 0;
                        v30 = 0;
                        v31 = 0;
                    } else {
                        v20 = 6;
                        v21 = 6;
                        v30 = 10;
                        v31 = 7;
                    }
                    if diff(w[6], w[8]) {
                        v22 = 4;
                        v23 = 8;
                        v32 = 7;
                    } else {
                        v22 = 0;
                        v23 = 7;
                        v32 = 7;
                    }
                }
                239 => {
                    v02 = 6;
                    v03 = 5;
                    v12 = 2;
                    v13 = 3;
                    v33 = 0;
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 0;
                        v10 = 0;
                        v11 = 0;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                    if diff(w[8], w[4]) {
                        v20 = 0;
                        v21 = 0;
                        v30 = 0;
                        v31 = 0;
                        v22 = 0;
                        v23 = 0;
                        v32 = 0;
                    } else {
                        v20 = 5;
                        v21 = 6;
                        v30 = 5;
                        v31 = 10;
                        v22 = 0;
                        v23 = 5;
                        v32 = 6;
                    }
                }
                127 => {
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 0;
                        v10 = 0;
                        v11 = 0;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                    if diff(w[2], w[6]) {
                        v02 = 0;
                        v03 = 0;
                        v12 = 0;
                        v13 = 0;
                    } else {
                        v02 = 3;
                        v03 = 10;
                        v12 = 5;
                        v13 = 10;
                    }
                    if diff(w[8], w[4]) {
                        v20 = 0;
                        v21 = 0;
                        v30 = 0;
                        v31 = 0;
                    } else {
                        v20 = 5;
                        v21 = 1;
                        v30 = 2;
                        v31 = 8;
                    }
                    v22 = 4;
                    v23 = 8;
                    v32 = 7;
                    v33 = 3;
                }
                191 => {
                    v20 = 6;
                    v21 = 6;
                    v22 = 6;
                    v23 = 0;
                    v30 = 6;
                    v31 = 6;
                    v32 = 6;
                    v33 = 0;
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 0;
                        v10 = 0;
                        v11 = 0;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                    if diff(w[2], w[6]) {
                        v02 = 0;
                        v03 = 0;
                        v12 = 0;
                        v13 = 0;
                    } else {
                        v02 = 5;
                        v03 = 7;
                        v12 = 5;
                        v13 = 9;
                    }
                }
                223 => {
                    v20 = 8;
                    v21 = 3;
                    v30 = 5;
                    v31 = 2;
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 0;
                        v10 = 0;
                        v11 = 0;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                    if diff(w[2], w[6]) {
                        v02 = 0;
                        v03 = 0;
                        v12 = 0;
                        v13 = 0;
                    } else {
                        v02 = 5;
                        v03 = 7;
                        v12 = 5;
                        v13 = 9;
                    }
                    if diff(w[6], w[8]) {
                        v22 = 0;
                        v23 = 0;
                        v32 = 0;
                        v33 = 0;
                    } else {
                        v22 = 0;
                        v23 = 6;
                        v32 = 6;
                        v33 = 6;
                    }
                }
                247 => {
                    v20 = 7;
                    v21 = 5;
                    v30 = 7;
                    v31 = 5;
                    if diff(w[2], w[6]) {
                        v02 = 0;
                        v12 = 0;
                        v03 = 0;
                        v13 = 0;
                        v00 = 6;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    } else {
                        v02 = 6;
                        v12 = 0;
                        v03 = 10;
                        v13 = 6;
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                    if diff(w[6], w[8]) {
                        v22 = 0;
                        v32 = 0;
                        v23 = 0;
                        v33 = 0;
                    } else {
                        v22 = 10;
                        v32 = 10;
                        v23 = 6;
                        v33 = 6;
                    }
                }
                255 => {
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 0;
                        v10 = 0;
                        v11 = 0;
                    } else {
                        v00 = 4;
                        v01 = 6;
                        v10 = 6;
                        v11 = 0;
                    }
                    if diff(w[2], w[6]) {
                        v02 = 0;
                        v03 = 0;
                        v12 = 0;
                        v13 = 0;
                    } else {
                        v02 = 3;
                        v03 = 10;
                        v12 = 5;
                        v13 = 10;
                    }
                    if diff(w[8], w[4]) {
                        v20 = 0;
                        v21 = 0;
                        v30 = 0;
                        v31 = 0;
                    } else {
                        v20 = 5;
                        v21 = 6;
                        v30 = 5;
                        v31 = 10;
                    }
                    if diff(w[6], w[8]) {
                        v22 = 0;
                        v23 = 0;
                        v32 = 0;
                        v33 = 0;
                    } else {
                        v22 = 3;
                        v23 = 3;
                        v32 = 1;
                        v33 = 1;
                    }
                }
                _ => {}
            }

            let out_base = (j * 4) * dp_stride + (i * 4);

            render_p00(&mut dp[out_base], &w, v00);
            render_p01(&mut dp[out_base + 1], &w, v01);
            render_p02(&mut dp[out_base + 2], &w, v02);
            render_p03(&mut dp[out_base + 3], &w, v03);

            let row1 = out_base + dp_stride;
            render_p10(&mut dp[row1], &w, v10);
            render_p11(&mut dp[row1 + 1], &w, v11);
            render_p12(&mut dp[row1 + 2], &w, v12);
            render_p13(&mut dp[row1 + 3], &w, v13);

            let row2 = row1 + dp_stride;
            render_p20(&mut dp[row2], &w, v20);
            render_p21(&mut dp[row2 + 1], &w, v21);
            render_p22(&mut dp[row2 + 2], &w, v22);
            render_p23(&mut dp[row2 + 3], &w, v23);

            let row3 = row2 + dp_stride;
            render_p30(&mut dp[row3], &w, v30);
            render_p31(&mut dp[row3 + 1], &w, v31);
            render_p32(&mut dp[row3 + 2], &w, v32);
            render_p33(&mut dp[row3 + 3], &w, v33);
        }
    }
}

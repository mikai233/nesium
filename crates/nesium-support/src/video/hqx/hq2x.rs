use super::hqx_common::*;

#[inline(always)]
fn render_op(dst: &mut u32, w5: u32, wA: u32, wB: u32, wC: u32, op: u8) {
    match op {
        0 => *dst = w5,
        10 => *dst = interp1(w5, wA),
        11 => *dst = interp1(w5, wB),
        12 => *dst = interp1(w5, wC),
        20 => *dst = interp2(w5, wB, wC),
        21 => *dst = interp2(w5, wA, wC),
        22 => *dst = interp2(w5, wA, wB),
        60 => *dst = interp6(w5, wC, wB),
        61 => *dst = interp6(w5, wB, wC),
        70 => *dst = interp7(w5, wB, wC),
        90 => *dst = interp9(w5, wB, wC),
        100 => *dst = interp10(w5, wB, wC),
        _ => unreachable!(),
    }
}

pub fn hq2x_32_rb(
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
            let mut v10 = 0;
            let mut v11 = 0;

            match pattern {
                0 | 1 | 4 | 32 | 128 | 5 | 132 | 160 | 33 | 129 | 36 | 133 | 164 | 161 | 37
                | 165 => {
                    v00 = 20;
                    v01 = 20;
                    v10 = 20;
                    v11 = 20;
                }
                2 | 34 | 130 | 162 => {
                    v00 = 22;
                    v01 = 21;
                    v10 = 20;
                    v11 = 20;
                }
                16 | 17 | 48 | 49 => {
                    v00 = 20;
                    v01 = 22;
                    v10 = 20;
                    v11 = 21;
                }
                64 | 65 | 68 | 69 => {
                    v00 = 20;
                    v01 = 20;
                    v10 = 21;
                    v11 = 22;
                }
                8 | 12 | 136 | 140 => {
                    v00 = 21;
                    v01 = 20;
                    v10 = 22;
                    v11 = 20;
                }
                3 | 35 | 131 | 163 => {
                    v00 = 11;
                    v01 = 21;
                    v10 = 20;
                    v11 = 20;
                }
                6 | 38 | 134 | 166 => {
                    v00 = 22;
                    v01 = 12;
                    v10 = 20;
                    v11 = 20;
                }
                20 | 21 | 52 | 53 => {
                    v00 = 20;
                    v01 = 11;
                    v10 = 20;
                    v11 = 21;
                }
                144 | 145 | 176 | 177 => {
                    v00 = 20;
                    v01 = 22;
                    v10 = 20;
                    v11 = 12;
                }
                192 | 193 | 196 | 197 => {
                    v00 = 20;
                    v01 = 20;
                    v10 = 21;
                    v11 = 11;
                }
                96 | 97 | 100 | 101 => {
                    v00 = 20;
                    v01 = 20;
                    v10 = 12;
                    v11 = 22;
                }
                40 | 44 | 168 | 172 => {
                    v00 = 21;
                    v01 = 20;
                    v10 = 11;
                    v11 = 20;
                }
                9 | 13 | 137 | 141 => {
                    v00 = 12;
                    v01 = 20;
                    v10 = 22;
                    v11 = 20;
                }
                18 | 50 => {
                    v00 = 22;
                    v01 = if diff(w[2], w[6]) { 10 } else { 20 };
                    v10 = 20;
                    v11 = 21;
                }
                80 | 81 => {
                    v00 = 20;
                    v01 = 22;
                    v10 = 21;
                    v11 = if diff(w[6], w[8]) { 10 } else { 20 };
                }
                72 | 76 => {
                    v00 = 21;
                    v01 = 20;
                    v10 = if diff(w[8], w[4]) { 10 } else { 20 };
                    v11 = 22;
                }
                10 | 138 => {
                    v00 = if diff(w[4], w[2]) { 10 } else { 20 };
                    v01 = 21;
                    v10 = 22;
                    v11 = 20;
                }
                66 => {
                    v00 = 22;
                    v01 = 21;
                    v10 = 21;
                    v11 = 22;
                }
                24 => {
                    v00 = 21;
                    v01 = 22;
                    v10 = 22;
                    v11 = 21;
                }
                7 | 39 | 135 => {
                    v00 = 11;
                    v01 = 12;
                    v10 = 20;
                    v11 = 20;
                }
                148 | 149 | 180 => {
                    v00 = 20;
                    v01 = 11;
                    v10 = 20;
                    v11 = 12;
                }
                224 | 228 | 225 => {
                    v00 = 20;
                    v01 = 20;
                    v10 = 12;
                    v11 = 11;
                }
                41 | 169 | 45 => {
                    v00 = 12;
                    v01 = 20;
                    v10 = 11;
                    v11 = 20;
                }
                22 | 54 => {
                    v00 = 22;
                    v01 = if diff(w[2], w[6]) { 0 } else { 20 };
                    v10 = 20;
                    v11 = 21;
                }
                208 | 209 => {
                    v00 = 20;
                    v01 = 22;
                    v10 = 21;
                    v11 = if diff(w[6], w[8]) { 0 } else { 20 };
                }
                104 | 108 => {
                    v00 = 21;
                    v01 = 20;
                    v10 = if diff(w[8], w[4]) { 0 } else { 20 };
                    v11 = 22;
                }
                11 | 139 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 20 };
                    v01 = 21;
                    v10 = 22;
                    v11 = 20;
                }
                19 | 51 => {
                    if diff(w[2], w[6]) {
                        v00 = 11;
                        v01 = 10;
                    } else {
                        v00 = 60;
                        v01 = 90;
                    }
                    v10 = 20;
                    v11 = 21;
                }
                146 | 178 => {
                    v00 = 22;
                    if diff(w[2], w[6]) {
                        v01 = 10;
                        v11 = 12;
                    } else {
                        v01 = 90;
                        v11 = 61;
                    }
                    v10 = 20;
                }
                84 | 85 => {
                    v00 = 20;
                    if diff(w[6], w[8]) {
                        v01 = 11;
                        v11 = 10;
                    } else {
                        v01 = 60;
                        v11 = 90;
                    }
                    v10 = 21;
                }
                112 | 113 => {
                    v00 = 20;
                    v01 = 22;
                    if diff(w[6], w[8]) {
                        v10 = 12;
                        v11 = 10;
                    } else {
                        v10 = 61;
                        v11 = 90;
                    }
                }
                200 | 204 => {
                    v00 = 21;
                    v01 = 20;
                    if diff(w[8], w[4]) {
                        v10 = 10;
                        v11 = 11;
                    } else {
                        v10 = 90;
                        v11 = 60;
                    }
                }
                73 | 77 => {
                    if diff(w[8], w[4]) {
                        v00 = 12;
                        v10 = 10;
                    } else {
                        v00 = 61;
                        v10 = 90;
                    }
                    v01 = 20;
                    v11 = 22;
                }
                42 | 170 => {
                    if diff(w[4], w[2]) {
                        v00 = 10;
                        v10 = 11;
                    } else {
                        v00 = 90;
                        v10 = 60;
                    }
                    v01 = 21;
                    v11 = 20;
                }
                14 | 142 => {
                    if diff(w[4], w[2]) {
                        v00 = 10;
                        v01 = 12;
                    } else {
                        v00 = 90;
                        v01 = 61;
                    }
                    v10 = 22;
                    v11 = 20;
                }
                67 => {
                    v00 = 11;
                    v01 = 21;
                    v10 = 21;
                    v11 = 22;
                }
                70 => {
                    v00 = 22;
                    v01 = 12;
                    v10 = 21;
                    v11 = 22;
                }
                28 => {
                    v00 = 21;
                    v01 = 11;
                    v10 = 22;
                    v11 = 21;
                }
                152 => {
                    v00 = 21;
                    v01 = 22;
                    v10 = 22;
                    v11 = 12;
                }
                194 => {
                    v00 = 22;
                    v01 = 21;
                    v10 = 21;
                    v11 = 11;
                }
                98 => {
                    v00 = 22;
                    v01 = 21;
                    v10 = 12;
                    v11 = 22;
                }
                56 => {
                    v00 = 21;
                    v01 = 22;
                    v10 = 11;
                    v11 = 21;
                }
                25 => {
                    v00 = 12;
                    v01 = 22;
                    v10 = 22;
                    v11 = 21;
                }
                26 | 31 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 20 };
                    v01 = if diff(w[2], w[6]) { 0 } else { 20 };
                    v10 = 22;
                    v11 = 21;
                }
                82 | 214 => {
                    v00 = 22;
                    v01 = if diff(w[2], w[6]) { 0 } else { 20 };
                    v10 = 21;
                    v11 = if diff(w[6], w[8]) { 0 } else { 20 };
                }
                88 | 248 => {
                    v00 = 21;
                    v01 = 22;
                    v10 = if diff(w[8], w[4]) { 0 } else { 20 };
                    v11 = if diff(w[6], w[8]) { 0 } else { 20 };
                }
                74 | 107 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 20 };
                    v01 = 21;
                    v10 = if diff(w[8], w[4]) { 0 } else { 20 };
                    v11 = 22;
                }
                27 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 20 };
                    v01 = 10;
                    v10 = 22;
                    v11 = 21;
                }
                79 | 86 => {
                    v00 = 22;
                    v01 = if diff(w[2], w[6]) { 0 } else { 20 };
                    v10 = 21;
                    v11 = 10;
                }
                216 => {
                    v00 = 21;
                    v01 = 22;
                    v10 = 10;
                    v11 = if diff(w[6], w[8]) { 0 } else { 20 };
                }
                106 => {
                    v00 = 10;
                    v01 = 21;
                    v10 = if diff(w[8], w[4]) { 0 } else { 20 };
                    v11 = 22;
                }
                30 => {
                    v00 = 10;
                    v01 = if diff(w[2], w[6]) { 0 } else { 20 };
                    v10 = 22;
                    v11 = 21;
                }
                210 => {
                    v00 = 22;
                    v01 = 10;
                    v10 = 21;
                    v11 = if diff(w[6], w[8]) { 0 } else { 20 };
                }
                120 => {
                    v00 = 21;
                    v01 = 22;
                    v10 = if diff(w[8], w[4]) { 0 } else { 20 };
                    v11 = 10;
                }
                75 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 20 };
                    v01 = 21;
                    v10 = 10;
                    v11 = 22;
                }
                29 => {
                    v00 = 12;
                    v01 = 11;
                    v10 = 22;
                    v11 = 21;
                }
                198 => {
                    v00 = 22;
                    v01 = 12;
                    v10 = 21;
                    v11 = 11;
                }
                184 => {
                    v00 = 21;
                    v01 = 22;
                    v10 = 11;
                    v11 = 12;
                }
                99 => {
                    v00 = 11;
                    v01 = 21;
                    v10 = 12;
                    v11 = 22;
                }
                57 => {
                    v00 = 12;
                    v01 = 22;
                    v10 = 11;
                    v11 = 21;
                }
                71 => {
                    v00 = 11;
                    v01 = 12;
                    v10 = 21;
                    v11 = 22;
                }
                156 => {
                    v00 = 21;
                    v01 = 11;
                    v10 = 22;
                    v11 = 12;
                }
                226 => {
                    v00 = 22;
                    v01 = 21;
                    v10 = 12;
                    v11 = 11;
                }
                60 => {
                    v00 = 21;
                    v01 = 11;
                    v10 = 11;
                    v11 = 21;
                }
                195 => {
                    v00 = 11;
                    v01 = 21;
                    v10 = 21;
                    v11 = 11;
                }
                102 => {
                    v00 = 22;
                    v01 = 12;
                    v10 = 12;
                    v11 = 22;
                }
                153 => {
                    v00 = 12;
                    v01 = 22;
                    v10 = 22;
                    v11 = 12;
                }
                58 => {
                    v00 = if diff(w[4], w[2]) { 10 } else { 70 };
                    v01 = if diff(w[2], w[6]) { 10 } else { 70 };
                    v10 = 11;
                    v11 = 21;
                }
                83 => {
                    v00 = 11;
                    v01 = if diff(w[2], w[6]) { 10 } else { 70 };
                    v10 = 21;
                    v11 = if diff(w[6], w[8]) { 10 } else { 70 };
                }
                92 => {
                    v00 = 21;
                    v01 = 11;
                    v10 = if diff(w[8], w[4]) { 10 } else { 70 };
                    v11 = if diff(w[6], w[8]) { 10 } else { 70 };
                }
                202 => {
                    v00 = if diff(w[4], w[2]) { 10 } else { 70 };
                    v01 = 21;
                    v10 = if diff(w[8], w[4]) { 10 } else { 70 };
                    v11 = 11;
                }
                78 => {
                    v00 = if diff(w[4], w[2]) { 10 } else { 70 };
                    v01 = 12;
                    v10 = if diff(w[8], w[4]) { 10 } else { 70 };
                    v11 = 22;
                }
                154 => {
                    v00 = if diff(w[4], w[2]) { 10 } else { 70 };
                    v01 = if diff(w[2], w[6]) { 10 } else { 70 };
                    v10 = 22;
                    v11 = 12;
                }
                114 => {
                    v00 = 22;
                    v01 = if diff(w[2], w[6]) { 10 } else { 70 };
                    v10 = 12;
                    v11 = if diff(w[6], w[8]) { 10 } else { 70 };
                }
                89 => {
                    v00 = 12;
                    v01 = 22;
                    v10 = if diff(w[8], w[4]) { 10 } else { 70 };
                    v11 = if diff(w[6], w[8]) { 10 } else { 70 };
                }
                90 => {
                    v00 = if diff(w[4], w[2]) { 10 } else { 70 };
                    v01 = if diff(w[2], w[6]) { 10 } else { 70 };
                    v10 = if diff(w[8], w[4]) { 10 } else { 70 };
                    v11 = if diff(w[6], w[8]) { 10 } else { 70 };
                }
                55 | 23 => {
                    if diff(w[2], w[6]) {
                        v00 = 11;
                        v01 = 0;
                    } else {
                        v00 = 60;
                        v01 = 90;
                    }
                    v10 = 20;
                    v11 = 21;
                }
                182 | 150 => {
                    v00 = 22;
                    if diff(w[2], w[6]) {
                        v01 = 0;
                        v11 = 12;
                    } else {
                        v01 = 90;
                        v11 = 61;
                    }
                    v10 = 20;
                }
                213 | 212 => {
                    v00 = 20;
                    if diff(w[6], w[8]) {
                        v01 = 11;
                        v11 = 0;
                    } else {
                        v01 = 60;
                        v11 = 90;
                    }
                    v10 = 21;
                }
                241 | 240 => {
                    v00 = 20;
                    v01 = 22;
                    if diff(w[6], w[8]) {
                        v10 = 12;
                        v11 = 0;
                    } else {
                        v10 = 61;
                        v11 = 90;
                    }
                }
                236 | 232 => {
                    v00 = 21;
                    v01 = 20;
                    if diff(w[8], w[4]) {
                        v10 = 0;
                        v11 = 11;
                    } else {
                        v10 = 90;
                        v11 = 60;
                    }
                }
                109 | 105 => {
                    if diff(w[8], w[4]) {
                        v00 = 12;
                        v10 = 0;
                    } else {
                        v00 = 61;
                        v10 = 90;
                    }
                    v01 = 20;
                    v11 = 22;
                }
                171 | 43 => {
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v10 = 11;
                    } else {
                        v00 = 90;
                        v10 = 60;
                    }
                    v01 = 21;
                    v11 = 20;
                }
                143 | 15 => {
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 12;
                    } else {
                        v00 = 90;
                        v01 = 61;
                    }
                    v10 = 22;
                    v11 = 20;
                }
                124 => {
                    v00 = 21;
                    v01 = 11;
                    v10 = if diff(w[8], w[4]) { 0 } else { 20 };
                    v11 = 10;
                }
                203 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 20 };
                    v01 = 21;
                    v10 = 10;
                    v11 = 11;
                }
                62 => {
                    v00 = 10;
                    v01 = if diff(w[2], w[6]) { 0 } else { 20 };
                    v10 = 11;
                    v11 = 21;
                }
                211 => {
                    v00 = 11;
                    v01 = 10;
                    v10 = 21;
                    v11 = if diff(w[6], w[8]) { 0 } else { 20 };
                }
                118 => {
                    v00 = 22;
                    v01 = if diff(w[2], w[6]) { 0 } else { 20 };
                    v10 = 12;
                    v11 = 10;
                }
                217 => {
                    v00 = 12;
                    v01 = 22;
                    v10 = 10;
                    v11 = if diff(w[6], w[8]) { 0 } else { 20 };
                }
                110 => {
                    v00 = 10;
                    v01 = 12;
                    v10 = if diff(w[8], w[4]) { 0 } else { 20 };
                    v11 = 22;
                }
                155 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 20 };
                    v01 = 10;
                    v10 = 22;
                    v11 = 12;
                }
                188 => {
                    v00 = 21;
                    v01 = 11;
                    v10 = 11;
                    v11 = 12;
                }
                185 => {
                    v00 = 12;
                    v01 = 22;
                    v10 = 11;
                    v11 = 12;
                }
                61 => {
                    v00 = 12;
                    v01 = 11;
                    v10 = 11;
                    v11 = 21;
                }
                157 => {
                    v00 = 12;
                    v01 = 11;
                    v10 = 22;
                    v11 = 12;
                }
                103 => {
                    v00 = 11;
                    v01 = 12;
                    v10 = 12;
                    v11 = 22;
                }
                227 => {
                    v00 = 11;
                    v01 = 21;
                    v10 = 12;
                    v11 = 11;
                }
                230 => {
                    v00 = 22;
                    v01 = 12;
                    v10 = 12;
                    v11 = 11;
                }
                199 => {
                    v00 = 11;
                    v01 = 12;
                    v10 = 21;
                    v11 = 11;
                }
                220 => {
                    v00 = 21;
                    v01 = 11;
                    v10 = if diff(w[8], w[4]) { 10 } else { 70 };
                    v11 = if diff(w[6], w[8]) { 0 } else { 20 };
                }
                158 => {
                    v00 = if diff(w[4], w[2]) { 10 } else { 70 };
                    v01 = if diff(w[2], w[6]) { 0 } else { 20 };
                    v10 = 22;
                    v11 = 12;
                }
                234 => {
                    v00 = if diff(w[4], w[2]) { 10 } else { 70 };
                    v01 = 21;
                    v10 = if diff(w[8], w[4]) { 0 } else { 20 };
                    v11 = 11;
                }
                242 => {
                    v00 = 22;
                    v01 = if diff(w[2], w[6]) { 10 } else { 70 };
                    v10 = 12;
                    v11 = if diff(w[6], w[8]) { 0 } else { 20 };
                }
                59 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 20 };
                    v01 = if diff(w[2], w[6]) { 10 } else { 70 };
                    v10 = 11;
                    v11 = 21;
                }
                121 => {
                    v00 = 12;
                    v01 = 22;
                    v10 = if diff(w[8], w[4]) { 0 } else { 20 };
                    v11 = if diff(w[6], w[8]) { 10 } else { 70 };
                }
                87 => {
                    v00 = 11;
                    v01 = if diff(w[2], w[6]) { 0 } else { 20 };
                    v10 = 21;
                    v11 = if diff(w[6], w[8]) { 10 } else { 70 };
                }
                79 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 20 };
                    v01 = 12;
                    v10 = if diff(w[8], w[4]) { 10 } else { 70 };
                    v11 = 22;
                }
                122 => {
                    v00 = if diff(w[4], w[2]) { 10 } else { 70 };
                    v01 = if diff(w[2], w[6]) { 10 } else { 70 };
                    v10 = if diff(w[8], w[4]) { 0 } else { 20 };
                    v11 = if diff(w[6], w[8]) { 10 } else { 70 };
                }
                94 => {
                    v00 = if diff(w[4], w[2]) { 10 } else { 70 };
                    v01 = if diff(w[2], w[6]) { 0 } else { 20 };
                    v10 = if diff(w[8], w[4]) { 10 } else { 70 };
                    v11 = if diff(w[6], w[8]) { 10 } else { 70 };
                }
                218 => {
                    v00 = if diff(w[4], w[2]) { 10 } else { 70 };
                    v01 = if diff(w[2], w[6]) { 10 } else { 70 };
                    v10 = if diff(w[8], w[4]) { 10 } else { 70 };
                    v11 = if diff(w[6], w[8]) { 0 } else { 20 };
                }
                91 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 20 };
                    v01 = if diff(w[2], w[6]) { 10 } else { 70 };
                    v10 = if diff(w[8], w[4]) { 10 } else { 70 };
                    v11 = if diff(w[6], w[8]) { 10 } else { 70 };
                }
                229 => {
                    v00 = 20;
                    v01 = 20;
                    v10 = 12;
                    v11 = 11;
                }
                167 => {
                    v00 = 11;
                    v01 = 12;
                    v10 = 20;
                    v11 = 20;
                }
                173 => {
                    v00 = 12;
                    v01 = 20;
                    v10 = 11;
                    v11 = 20;
                }
                181 => {
                    v00 = 20;
                    v01 = 11;
                    v10 = 20;
                    v11 = 12;
                }
                186 => {
                    v00 = if diff(w[4], w[2]) { 10 } else { 70 };
                    v01 = if diff(w[2], w[6]) { 10 } else { 70 };
                    v10 = 11;
                    v11 = 12;
                }
                115 => {
                    v00 = 11;
                    v01 = if diff(w[2], w[6]) { 10 } else { 70 };
                    v10 = 12;
                    v11 = if diff(w[6], w[8]) { 10 } else { 70 };
                }
                93 => {
                    v00 = 12;
                    v01 = 11;
                    v10 = if diff(w[8], w[4]) { 10 } else { 70 };
                    v11 = if diff(w[6], w[8]) { 10 } else { 70 };
                }
                206 => {
                    v00 = if diff(w[4], w[2]) { 10 } else { 70 };
                    v01 = 12;
                    v10 = if diff(w[8], w[4]) { 10 } else { 70 };
                    v11 = 11;
                }
                205 | 201 => {
                    v00 = 12;
                    v01 = 20;
                    v10 = if diff(w[8], w[4]) { 10 } else { 70 };
                    v11 = 11;
                }
                174 | 46 => {
                    v00 = if diff(w[4], w[2]) { 10 } else { 70 };
                    v01 = 12;
                    v10 = 11;
                    v11 = 20;
                }
                179 | 147 => {
                    v00 = 11;
                    v01 = if diff(w[2], w[6]) { 10 } else { 70 };
                    v10 = 20;
                    v11 = 12;
                }
                117 | 116 => {
                    v00 = 20;
                    v01 = 11;
                    v10 = 12;
                    v11 = if diff(w[6], w[8]) { 10 } else { 70 };
                }
                189 => {
                    v00 = 12;
                    v01 = 11;
                    v10 = 11;
                    v11 = 12;
                }
                231 => {
                    v00 = 11;
                    v01 = 12;
                    v10 = 12;
                    v11 = 11;
                }
                126 => {
                    v00 = 10;
                    v01 = if diff(w[2], w[6]) { 0 } else { 20 };
                    v10 = if diff(w[8], w[4]) { 0 } else { 20 };
                    v11 = 10;
                }
                219 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 20 };
                    v01 = 10;
                    v10 = 10;
                    v11 = if diff(w[6], w[8]) { 0 } else { 20 };
                }
                125 => {
                    if diff(w[8], w[4]) {
                        v00 = 12;
                        v10 = 0;
                    } else {
                        v00 = 61;
                        v10 = 90;
                    }
                    v01 = 11;
                    v11 = 10;
                }
                221 => {
                    v00 = 12;
                    if diff(w[6], w[8]) {
                        v01 = 11;
                        v11 = 0;
                    } else {
                        v01 = 60;
                        v11 = 90;
                    }
                    v10 = 10;
                }
                207 => {
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v01 = 12;
                    } else {
                        v00 = 90;
                        v01 = 61;
                    }
                    v10 = 10;
                    v11 = 11;
                }
                238 => {
                    v00 = 10;
                    v01 = 12;
                    if diff(w[8], w[4]) {
                        v10 = 0;
                        v11 = 11;
                    } else {
                        v10 = 90;
                        v11 = 60;
                    }
                }
                190 => {
                    v00 = 10;
                    if diff(w[2], w[6]) {
                        v01 = 0;
                        v11 = 12;
                    } else {
                        v01 = 90;
                        v11 = 61;
                    }
                    v10 = 11;
                }
                187 => {
                    if diff(w[4], w[2]) {
                        v00 = 0;
                        v10 = 11;
                    } else {
                        v00 = 90;
                        v10 = 60;
                    }
                    v01 = 10;
                    v11 = 12;
                }
                243 => {
                    v00 = 11;
                    v01 = 10;
                    if diff(w[6], w[8]) {
                        v10 = 12;
                        v11 = 0;
                    } else {
                        v10 = 61;
                        v11 = 90;
                    }
                }
                119 => {
                    if diff(w[2], w[6]) {
                        v00 = 11;
                        v01 = 0;
                    } else {
                        v00 = 60;
                        v01 = 90;
                    }
                    v10 = 12;
                    v11 = 10;
                }
                237 | 233 => {
                    v00 = 12;
                    v01 = 20;
                    v10 = if diff(w[8], w[4]) { 0 } else { 100 };
                    v11 = 11;
                }
                175 | 47 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 100 };
                    v01 = 12;
                    v10 = 11;
                    v11 = 20;
                }
                183 | 151 => {
                    v00 = 11;
                    v01 = if diff(w[2], w[6]) { 0 } else { 100 };
                    v10 = 20;
                    v11 = 12;
                }
                245 | 244 => {
                    v00 = 20;
                    v01 = 11;
                    v10 = 12;
                    v11 = if diff(w[6], w[8]) { 0 } else { 100 };
                }
                250 => {
                    v00 = 10;
                    v01 = 10;
                    v10 = if diff(w[8], w[4]) { 0 } else { 20 };
                    v11 = if diff(w[6], w[8]) { 0 } else { 20 };
                }
                123 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 20 };
                    v01 = 10;
                    v10 = if diff(w[8], w[4]) { 0 } else { 20 };
                    v11 = 10;
                }
                95 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 20 };
                    v01 = if diff(w[2], w[6]) { 0 } else { 20 };
                    v10 = 10;
                    v11 = 10;
                }
                222 => {
                    v00 = 10;
                    v01 = if diff(w[2], w[6]) { 0 } else { 20 };
                    v10 = 10;
                    v11 = if diff(w[6], w[8]) { 0 } else { 20 };
                }
                252 => {
                    v00 = 21;
                    v01 = 11;
                    v10 = if diff(w[8], w[4]) { 0 } else { 20 };
                    v11 = if diff(w[6], w[8]) { 0 } else { 100 };
                }
                249 => {
                    v00 = 12;
                    v01 = 22;
                    v10 = if diff(w[8], w[4]) { 0 } else { 100 };
                    v11 = if diff(w[6], w[8]) { 0 } else { 20 };
                }
                235 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 20 };
                    v01 = 21;
                    v10 = if diff(w[8], w[4]) { 0 } else { 100 };
                    v11 = 11;
                }
                111 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 100 };
                    v01 = 12;
                    v10 = if diff(w[8], w[4]) { 0 } else { 20 };
                    v11 = 22;
                }
                63 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 100 };
                    v01 = if diff(w[2], w[6]) { 0 } else { 20 };
                    v10 = 11;
                    v11 = 21;
                }
                159 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 20 };
                    v01 = if diff(w[2], w[6]) { 0 } else { 100 };
                    v10 = 22;
                    v11 = 12;
                }
                215 => {
                    v00 = 11;
                    v01 = if diff(w[2], w[6]) { 0 } else { 100 };
                    v10 = 21;
                    v11 = if diff(w[6], w[8]) { 0 } else { 20 };
                }
                246 => {
                    v00 = 22;
                    v01 = if diff(w[2], w[6]) { 0 } else { 20 };
                    v10 = 12;
                    v11 = if diff(w[6], w[8]) { 0 } else { 100 };
                }
                254 => {
                    v00 = 10;
                    v01 = if diff(w[2], w[6]) { 0 } else { 20 };
                    v10 = if diff(w[8], w[4]) { 0 } else { 20 };
                    v11 = if diff(w[6], w[8]) { 0 } else { 100 };
                }
                253 => {
                    v00 = 12;
                    v01 = 11;
                    v10 = if diff(w[8], w[4]) { 0 } else { 100 };
                    v11 = if diff(w[6], w[8]) { 0 } else { 100 };
                }
                251 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 20 };
                    v01 = 10;
                    v10 = if diff(w[8], w[4]) { 0 } else { 100 };
                    v11 = if diff(w[6], w[8]) { 0 } else { 20 };
                }
                239 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 100 };
                    v01 = 12;
                    v10 = if diff(w[8], w[4]) { 0 } else { 100 };
                    v11 = 11;
                }
                127 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 100 };
                    v01 = if diff(w[2], w[6]) { 0 } else { 20 };
                    v10 = if diff(w[8], w[4]) { 0 } else { 20 };
                    v11 = 10;
                }
                191 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 100 };
                    v01 = if diff(w[2], w[6]) { 0 } else { 100 };
                    v10 = 11;
                    v11 = 12;
                }
                223 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 20 };
                    v01 = if diff(w[2], w[6]) { 0 } else { 100 };
                    v10 = 10;
                    v11 = if diff(w[6], w[8]) { 0 } else { 20 };
                }
                247 => {
                    v00 = 11;
                    v01 = if diff(w[2], w[6]) { 0 } else { 100 };
                    v10 = 12;
                    v11 = if diff(w[6], w[8]) { 0 } else { 100 };
                }
                255 => {
                    v00 = if diff(w[4], w[2]) { 0 } else { 100 };
                    v01 = if diff(w[2], w[6]) { 0 } else { 100 };
                    v10 = if diff(w[8], w[4]) { 0 } else { 100 };
                    v11 = if diff(w[6], w[8]) { 0 } else { 100 };
                }
                _ => {
                    v00 = 20;
                    v01 = 20;
                    v10 = 20;
                    v11 = 20;
                }
            }

            let out_idx0 = (j * 2) * dp_stride + (i * 2);
            let out_idx1 = out_idx0 + 1;
            let out_idx2 = (j * 2 + 1) * dp_stride + (i * 2);
            let out_idx3 = out_idx2 + 1;

            render_op(&mut dp[out_idx0], w[5], w[1], w[4], w[2], v00);
            render_op(&mut dp[out_idx1], w[5], w[3], w[2], w[6], v01);
            render_op(&mut dp[out_idx2], w[5], w[7], w[8], w[4], v10);
            render_op(&mut dp[out_idx3], w[5], w[9], w[6], w[8], v11);
        }
    }
}

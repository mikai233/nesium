// NTSC filter based on Bisqwit's code/algorithm (as used in Mesen2).
// Forum reference: http://forums.nesdev.com/viewtopic.php?p=172329

#include <algorithm>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <mutex>

namespace {
constexpr std::uint16_t kBitmaskLut[12] = {
    0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x100, 0x200, 0x400, 0x800,
};

constexpr int kSignalsPerPixel = 8;

std::once_flag g_init_once;
std::int8_t g_signal_low[0x80];
std::int8_t g_signal_high[0x80];

void init_signal_tables() {
    // from https://forums.nesdev.org/viewtopic.php?p=159266#p159266
    const double signal_luma_low[2][4] = {
        {0.228, 0.312, 0.552, 0.880},
        {0.192, 0.256, 0.448, 0.712},
    };
    const double signal_luma_high[2][4] = {
        {0.616, 0.840, 1.100, 1.100},
        {0.500, 0.676, 0.896, 0.896},
    };
    const double signal_blank = signal_luma_low[0][1];
    const double signal_white = signal_luma_high[0][3];

    for (int h = 0; h <= 1; h++) {
        for (int i = 0; i <= 0x3F; i++) {
            double m = signal_luma_low[h][i / 0x10];
            double q = signal_luma_high[h][i / 0x10];

            if ((i & 0x0F) == 0x0D) {
                q = m;
            } else if ((i & 0x0F) == 0) {
                m = q;
            } else if ((i & 0x0F) >= 0x0E) {
                // colors $xE and $xF are not affected by emphasis
                // https://forums.nesdev.org/viewtopic.php?p=160669#p160669
                m = signal_luma_low[0][1];
                q = signal_luma_low[0][1];
            }

            const double low =
                std::floor(((m - signal_blank) / (signal_white - signal_blank)) * 100.0);
            const double high =
                std::floor(((q - signal_blank) / (signal_white - signal_blank)) * 100.0);

            g_signal_low[(h ? 0x40 : 0) | i] = static_cast<std::int8_t>(low);
            g_signal_high[(h ? 0x40 : 0) | i] = static_cast<std::int8_t>(high);
        }
    }
}

struct Coeff {
    int y_width = 12;
    int i_width = 12;
    int q_width = 12;

    int y = 0;
    int ir = 0, ig = 0, ib = 0;
    int qr = 0, qg = 0, qb = 0;

    int brightness = 0;
    std::int8_t sinetable[27]{};
};

Coeff compute_coeffs(double brightness,
                     double contrast,
                     double hue,
                     double saturation,
                     double y_filter_length,
                     double i_filter_length,
                     double q_filter_length) {
    Coeff c;

    const double pi = std::atan(1.0) * 4.0;
    const int contrast_i = static_cast<int>(
        (contrast + 1.0) * (contrast + 1.0) * 167941.0);
    const int saturation_i = static_cast<int>(
        (saturation + 1.0) * (saturation + 1.0) * 144044.0);

    c.brightness = static_cast<int>(brightness * 750.0);

    for (int i = 0; i < 27; i++) {
        c.sinetable[i] = static_cast<std::int8_t>(
            8.0 * std::sin(i * 2.0 * pi / 12.0 + hue * pi));
    }

    c.y_width = std::max(1, static_cast<int>(12.0 + y_filter_length * 24.0));
    c.i_width = std::max(12, static_cast<int>(12.0 + i_filter_length * 24.0));
    c.q_width = std::max(12, static_cast<int>(12.0 + q_filter_length * 24.0));

    c.y = contrast_i / c.y_width;

    c.ir = static_cast<int>(contrast_i * 1.994681e-6 * saturation_i / c.i_width);
    c.qr = static_cast<int>(contrast_i * 9.915742e-7 * saturation_i / c.q_width);

    c.ig = static_cast<int>(contrast_i * 9.151351e-8 * saturation_i / c.i_width);
    c.qg = static_cast<int>(contrast_i * -6.334805e-7 * saturation_i / c.q_width);

    c.ib = static_cast<int>(contrast_i * -1.012984e-6 * saturation_i / c.i_width);
    c.qb = static_cast<int>(contrast_i * 1.667217e-6 * saturation_i / c.q_width);

    return c;
}

inline std::int8_t read_signal(const std::int8_t* signal, int width, int pos) {
    return (pos >= 0 && pos < width) ? signal[pos] : 0;
}

inline std::int8_t cos_sample(const Coeff& c, int pos, int phase0) {
    return c.sinetable[((pos + 36) % 12) + phase0];
}

inline std::int8_t sin_sample(const Coeff& c, int pos, int phase0) {
    return c.sinetable[((pos + 36) % 12) + 3 + phase0];
}

void ntsc_decode_line(const Coeff& c,
                      int width,
                      const std::int8_t* signal,
                      std::uint32_t* target,
                      int phase0,
                      int res_divider) {
    int ysum = c.brightness, isum = 0, qsum = 0;

    const int max_filter = std::max(c.y_width, std::max(c.i_width, c.q_width)) / 2;

    for (int s = -max_filter; s < width; s++) {
        const int sy = s + c.y_width / 2;
        const int si = s + c.i_width / 2;
        const int sq = s + c.q_width / 2;

        ysum += read_signal(signal, width, sy) - read_signal(signal, width, sy - c.y_width);
        isum += read_signal(signal, width, si) * cos_sample(c, si, phase0) -
                read_signal(signal, width, si - c.i_width) * cos_sample(c, si - c.i_width, phase0);
        qsum += read_signal(signal, width, sq) * sin_sample(c, sq, phase0) -
                read_signal(signal, width, sq - c.q_width) * sin_sample(c, sq - c.q_width, phase0);

        if (s >= 0 && (s % res_divider) == 0) {
            const int r = std::min(255, std::max(0, (ysum * c.y + isum * c.ir + qsum * c.qr) / 65536));
            const int g = std::min(255, std::max(0, (ysum * c.y + isum * c.ig + qsum * c.qg) / 65536));
            const int b = std::min(255, std::max(0, (ysum * c.y + isum * c.ib + qsum * c.qb) / 65536));

            *target = 0xFF000000u | (static_cast<std::uint32_t>(r) << 16) |
                      (static_cast<std::uint32_t>(g) << 8) | static_cast<std::uint32_t>(b);
            target++;
        }
    }
}

void recursive_blend(int iteration_count,
                     std::uint64_t* output,
                     const std::uint64_t* current_line,
                     const std::uint64_t* next_line,
                     std::uint32_t width_qwords,
                     bool vertical_blend) {
    if (vertical_blend) {
        for (std::uint32_t x = 0; x < width_qwords; x++) {
            output[x] = ((((current_line[x] ^ next_line[x]) & 0xfefefefefefefefeULL) >> 1) +
                         (current_line[x] & next_line[x]));
        }
    } else {
        std::memcpy(output, current_line, width_qwords * sizeof(std::uint64_t));
    }

    iteration_count /= 2;
    if (iteration_count > 0) {
        recursive_blend(iteration_count,
                        output - width_qwords * iteration_count,
                        current_line,
                        output,
                        width_qwords,
                        vertical_blend);
        recursive_blend(iteration_count,
                        output + width_qwords * iteration_count,
                        output,
                        next_line,
                        width_qwords,
                        vertical_blend);
    }
}

void generate_ntsc_signal(const std::uint16_t* ppu,
                          int ppu_width,
                          int row,
                          std::int8_t* ntsc_signal,
                          std::int64_t& phase) {
    static constexpr std::uint16_t emphasis_lut[8] = {
        // R: 0b000000111111, G: 0b001111110000, B: 0b111100000011
        0,
        0b000000111111,
        0b001111110000,
        0b001111111111,
        0b111100000011,
        0b111100111111,
        0b111111110011,
        0b111111111111,
    };

    for (int x = 0; x < ppu_width; x++) {
        const std::uint16_t ppu_data = ppu[row * ppu_width + x];

        const std::uint16_t pixel_color = ppu_data & 0x3F;
        const std::uint8_t emphasis = static_cast<std::uint8_t>(ppu_data >> 6);
        const std::uint8_t hue = static_cast<std::uint8_t>(ppu_data & 0x0F);

        std::uint16_t emphasis_wave = 0;
        if (emphasis) {
            emphasis_wave = static_cast<std::uint16_t>(
                ((emphasis_lut[emphasis] >> (hue % 12)) |
                 (emphasis_lut[emphasis] << (12 - (hue % 12)))) &
                0xFFFF);
        }

        const int phase_mod = static_cast<int>((std::llabs(phase - hue) % 12));
        std::uint16_t phase_bitmask = kBitmaskLut[phase_mod];
        for (int j = 0; j < kSignalsPerPixel; j++) {
            phase_bitmask <<= 1;

            const std::uint8_t color =
                static_cast<std::uint8_t>(pixel_color | ((phase_bitmask & emphasis_wave) ? 0x40 : 0));
            std::int8_t voltage = g_signal_high[color];

            if (phase_bitmask >= (1 << 12)) {
                phase_bitmask = 1;
            } else if (phase_bitmask >= (1 << 6)) {
                voltage = g_signal_low[color];
            }
            ntsc_signal[(x << 3) | j] = voltage;
        }

        phase += kSignalsPerPixel;
    }

    phase += (341 - ppu_width) * kSignalsPerPixel;
}
} // namespace

extern "C" {
void nesium_ntsc_bisqwit_apply_argb8888(const std::uint16_t* ppu,
                                        int ppu_width,
                                        int ppu_height,
                                        std::uint32_t* dst,
                                        int scale,
                                        double brightness,
                                        double contrast,
                                        double hue,
                                        double saturation,
                                        double y_filter_length,
                                        double i_filter_length,
                                        double q_filter_length,
                                        int phase_offset) {
    std::call_once(g_init_once, init_signal_tables);

    if (!ppu || !dst || ppu_width <= 0 || ppu_height <= 0) {
        return;
    }

    if (!(scale == 2 || scale == 4 || scale == 8)) {
        return;
    }

    const int res_divider = 8 / scale;
    const int pixels_per_cycle = scale;

    const int out_width = ppu_width * scale;
    const std::uint32_t row_gap = static_cast<std::uint32_t>(out_width * pixels_per_cycle);
    const std::uint32_t out_height = static_cast<std::uint32_t>(ppu_height * scale);

    const Coeff coeffs = compute_coeffs(
        brightness,
        contrast,
        hue,
        saturation,
        y_filter_length,
        i_filter_length,
        q_filter_length);

    std::int64_t phase = static_cast<std::int64_t>(phase_offset);

    // Generate base lines (one per PPU row) at y*scale.
    std::int8_t row_signal[256 * kSignalsPerPixel];
    for (int y = 0; y < ppu_height; y++) {
        const int start_cycle = static_cast<int>(phase % 12);
        generate_ntsc_signal(ppu, ppu_width, y, row_signal, phase);
        ntsc_decode_line(coeffs,
                         256 * kSignalsPerPixel,
                         row_signal,
                         dst + static_cast<std::size_t>(y) * row_gap,
                         (start_cycle + 7) % 12,
                         res_divider);
    }

    // Fill missing vertical lines by recursive blending.
    const std::uint32_t width_qwords = static_cast<std::uint32_t>(out_width / 2);
    const int iteration_count = scale / 2;
    const bool vertical_blend = false;

    for (int y = 0; y < ppu_height; y++) {
        std::uint32_t* base = dst + static_cast<std::size_t>(y) * row_gap;
        std::uint64_t* current_line = reinterpret_cast<std::uint64_t*>(base);
        std::uint64_t* next_line =
            (y == (ppu_height - 1))
                ? current_line
                : reinterpret_cast<std::uint64_t*>(dst + static_cast<std::size_t>(y + 1) * row_gap);

        std::uint64_t* buffer = reinterpret_cast<std::uint64_t*>(base + (row_gap / 2));
        recursive_blend(iteration_count,
                        buffer,
                        current_line,
                        next_line,
                        width_qwords,
                        vertical_blend);
    }

    (void)out_height;
}
}

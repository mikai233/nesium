#include <cstdint>

static inline uint32_t apply_brightness(uint32_t argb, uint8_t brightness)
{
    const uint8_t r = static_cast<uint8_t>(((argb & 0x00FF0000u) >> 16) * brightness / 255);
    const uint8_t g = static_cast<uint8_t>(((argb & 0x0000FF00u) >> 8) * brightness / 255);
    const uint8_t b = static_cast<uint8_t>((argb & 0x000000FFu) * brightness / 255);
    return 0xFF000000u | (static_cast<uint32_t>(r) << 16) | (static_cast<uint32_t>(g) << 8) |
           static_cast<uint32_t>(b);
}

extern "C" void nesium_lcd_grid_2x_argb8888(
    const uint32_t* src,
    uint32_t width,
    uint32_t height,
    uint32_t src_stride,
    uint32_t* dst,
    uint32_t dst_stride,
    uint8_t top_left,
    uint8_t top_right,
    uint8_t bottom_left,
    uint8_t bottom_right)
{
    for (uint32_t y = 0; y < height; y++) {
        for (uint32_t x = 0; x < width; x++) {
            const uint32_t c = src[y * src_stride + x];
            const uint32_t out_y = y * 2;
            const uint32_t out_x = x * 2;
            const uint32_t pos = out_y * dst_stride + out_x;
            dst[pos] = apply_brightness(c, top_left);
            dst[pos + 1] = apply_brightness(c, top_right);
            dst[pos + dst_stride] = apply_brightness(c, bottom_left);
            dst[pos + dst_stride + 1] = apply_brightness(c, bottom_right);
        }
    }
}


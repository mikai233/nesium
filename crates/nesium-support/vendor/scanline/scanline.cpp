#include <algorithm>
#include <cstdint>

static inline uint32_t apply_scanline_effect(uint32_t argb, uint8_t brightness)
{
    const uint8_t r = static_cast<uint8_t>(((argb & 0x00FF0000u) >> 16) * brightness / 255);
    const uint8_t g = static_cast<uint8_t>(((argb & 0x0000FF00u) >> 8) * brightness / 255);
    const uint8_t b = static_cast<uint8_t>((argb & 0x000000FFu) * brightness / 255);
    return 0xFF000000u | (static_cast<uint32_t>(r) << 16) | (static_cast<uint32_t>(g) << 8) |
           static_cast<uint32_t>(b);
}

extern "C" void nesium_scanline_apply_argb8888(
    uint32_t* buffer,
    uint32_t width,
    uint32_t height,
    uint8_t brightness,
    uint8_t scale)
{
    if (brightness >= 255) {
        return;
    }

    scale = std::max<uint8_t>(2, scale);
    const uint32_t lines_to_skip = static_cast<uint32_t>(scale - 1);
    const uint32_t groups = height / scale;

    for (uint32_t i = 0; i < groups; i++) {
        buffer += width * lines_to_skip;
        for (uint32_t j = 0; j < width; j++) {
            buffer[j] = apply_scanline_effect(buffer[j], brightness);
        }
        buffer += width;
    }
}


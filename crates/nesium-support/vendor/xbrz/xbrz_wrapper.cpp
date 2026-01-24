#include <cstdint>
#include <cstddef>

#include "xbrz.h"

extern "C" {
void nesium_xbrz_scale_argb8888(std::size_t scale,
                               const std::uint32_t* src,
                               int src_width,
                               int src_height,
                               std::uint32_t* dst) {
    xbrz::scale(scale, src, dst, src_width, src_height, xbrz::ColorFormat::ARGB);
}
}


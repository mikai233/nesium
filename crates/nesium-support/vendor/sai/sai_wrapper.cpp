// C ABI wrappers for the Kreed/2xSaI family of scalers.
// This is used by the Rust `nesium-support` crate to provide bit-exact output.

#include <cstdint>

extern void twoxsai_generic_xrgb8888(unsigned width, unsigned height, uint32_t* src, unsigned src_stride, uint32_t* dst, unsigned dst_stride);
extern void supertwoxsai_generic_xrgb8888(unsigned width, unsigned height, uint32_t* src, unsigned src_stride, uint32_t* dst, unsigned dst_stride);
extern void supereagle_generic_xrgb8888(unsigned width, unsigned height, uint32_t* src, unsigned src_stride, uint32_t* dst, unsigned dst_stride);

extern "C" {
void nesium_sai_2xsai_xrgb8888(unsigned width, unsigned height, const uint32_t* src, unsigned src_stride, uint32_t* dst, unsigned dst_stride)
{
    // The original implementations operate on mutable pointers but do not mutate the source.
    twoxsai_generic_xrgb8888(width, height, const_cast<uint32_t*>(src), src_stride, dst, dst_stride);
}

void nesium_sai_super2xsai_xrgb8888(unsigned width, unsigned height, const uint32_t* src, unsigned src_stride, uint32_t* dst, unsigned dst_stride)
{
    supertwoxsai_generic_xrgb8888(width, height, const_cast<uint32_t*>(src), src_stride, dst, dst_stride);
}

void nesium_sai_supereagle_xrgb8888(unsigned width, unsigned height, const uint32_t* src, unsigned src_stride, uint32_t* dst, unsigned dst_stride)
{
    supereagle_generic_xrgb8888(width, height, const_cast<uint32_t*>(src), src_stride, dst, dst_stride);
}
}

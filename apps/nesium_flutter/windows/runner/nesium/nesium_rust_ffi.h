#pragma once

#include <cstddef>
#include <cstdint>

#if defined(_WIN32)
#define NESIUM_RUST_IMPORT __declspec(dllimport)
#else
#define NESIUM_RUST_IMPORT
#endif

#if defined(_MSC_VER)
#define NESIUM_CALLCONV __cdecl
#else
#define NESIUM_CALLCONV
#endif

using NesiumFrameReadyCallback = void(NESIUM_CALLCONV *)(
    uint32_t /*bufferIndex*/, uint32_t /*width*/, uint32_t /*height*/,
    uint32_t /*pitch*/, void * /*user*/);

extern "C" {
NESIUM_RUST_IMPORT void nesium_runtime_start();
NESIUM_RUST_IMPORT void
nesium_set_frame_ready_callback(NesiumFrameReadyCallback cb, void *user);
NESIUM_RUST_IMPORT void nesium_copy_frame(uint32_t bufferIndex, uint8_t *dst,
                                          uint32_t dstPitch,
                                          uint32_t dstHeight);
NESIUM_RUST_IMPORT void nesium_set_color_format(bool use_bgra);

NESIUM_RUST_IMPORT void nesium_aux_create(uint32_t id, uint32_t width,
                                          uint32_t height);
NESIUM_RUST_IMPORT std::size_t nesium_aux_copy(uint32_t id, uint8_t *dst,
                                               uint32_t dst_pitch,
                                               uint32_t dst_height);
NESIUM_RUST_IMPORT void nesium_aux_destroy(uint32_t id);
}

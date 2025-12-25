#pragma once

#include <flutter_linux/flutter_linux.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque handle for Nesium method channels and external texture bridge.
typedef struct _NesiumChannels NesiumChannels;

// Creates and wires up the Nesium platform channel and external texture.
// The returned pointer must be freed with nesium_channels_free().
NesiumChannels *nesium_channels_new(FlView *view);

// Releases all resources associated with the Nesium platform bridge.
void nesium_channels_free(NesiumChannels *channels);

#ifdef __cplusplus
} // extern "C"
#endif

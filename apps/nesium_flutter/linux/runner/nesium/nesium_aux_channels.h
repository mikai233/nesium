#pragma once

#include <flutter_linux/flutter_linux.h>

G_BEGIN_DECLS

typedef struct _NesiumAuxChannels NesiumAuxChannels;

// Creates a new auxiliary texture channel manager.
NesiumAuxChannels *nesium_aux_channels_new(FlView *view);

// Frees the auxiliary texture channel manager.
void nesium_aux_channels_free(NesiumAuxChannels *self);

G_END_DECLS

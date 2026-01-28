# nes_ntsc

This directory contains the `nes_ntsc` library, a high-quality NTSC video filter for NES emulators. The implementation simulates the NTSC composite video signal to produce authentic-looking graphics, including artifacts like color bleeding and crawling dots.

## Source & Upstream

- **Upstream Version**: 0.2.2
- **Author**: Shay Green ([http://www.slack.net/~ant/](http://www.slack.net/~ant/))
- **Source**: Vendored from the [Mesen2](https://github.com/SourMesen/Mesen2) project.

## Local Modifications

The following modifications have been made to the original source to integrate with the Nesium project:

- **Standalone Build Support**: Removed `#include "pch.h"` from `nes_ntsc.cpp` to eliminate dependency on precompiled headers.

## Licensing

This module is licensed under the **GNU Lesser General Public License (LGPL), version 2.1 or later**.

For the full license text and copyright notice, please refer to the header in [nes_ntsc.cpp](./nes_ntsc.cpp).


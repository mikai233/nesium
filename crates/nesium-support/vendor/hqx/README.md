# HQX Upscaler

This directory contains the HQX (High Quality Magnification) family of pixel-art upscaling algorithms, including `hq2x`, `hq3x`, and `hq4x`. These filters are designed to increase the resolution of low-resolution graphics while preserving sharp edges and smooth gradients.

## Source & Upstream

- **Author**: Maxim Stepin
- **Source**: Vendored from the [Mesen2](https://github.com/SourMesen/Mesen2) project.

## Local Modifications

The implementation has been modified to support standalone compilation within the Nesium project:

- **Standalone Build Support**: Removed `#include "../pch.h"` dependencies from the source files to eliminate external project requirements.

## Licensing

This module is licensed under the **GNU Lesser General Public License (LGPL), version 2.1 or later**.

For the full license text and copyright notice, please refer to the header in [hqx.h](./hqx.h).


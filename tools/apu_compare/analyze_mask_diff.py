#!/usr/bin/env python3
"""Analyze differences between two 1bpp bit-packed frame masks."""

from __future__ import annotations

import argparse
from pathlib import Path


def unpack_mask(data: bytes, width: int, height: int) -> list[int]:
    total = width * height
    pixels = [0] * total
    bit_index = 0
    for byte in data:
        for bit in range(8):
            if bit_index >= total:
                break
            pixels[bit_index] = (byte >> bit) & 1
            bit_index += 1
    return pixels


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--a", required=True, type=Path, help="mask A path")
    parser.add_argument("--b", required=True, type=Path, help="mask B path")
    parser.add_argument("--width", type=int, default=256)
    parser.add_argument("--height", type=int, default=240)
    parser.add_argument(
        "--top-rows",
        type=int,
        default=20,
        help="number of rows with largest diffs to print",
    )
    args = parser.parse_args()

    data_a = args.a.read_bytes()
    data_b = args.b.read_bytes()
    if len(data_a) != len(data_b):
        print(f"size mismatch: len(a)={len(data_a)} len(b)={len(data_b)}")
        return 1

    expected_size = (args.width * args.height + 7) // 8
    if len(data_a) != expected_size:
        print(
            f"warning: file size {len(data_a)} differs from expected {expected_size} for {args.width}x{args.height}"
        )

    pixels_a = unpack_mask(data_a, args.width, args.height)
    pixels_b = unpack_mask(data_b, args.width, args.height)

    total = args.width * args.height
    diff_positions = [i for i in range(total) if pixels_a[i] != pixels_b[i]]
    diff_count = len(diff_positions)
    print(f"total_pixels={total} diff_pixels={diff_count}")

    if diff_count == 0:
        return 0

    min_x = args.width
    max_x = -1
    min_y = args.height
    max_y = -1
    row_counts = [0] * args.height

    for idx in diff_positions:
        y, x = divmod(idx, args.width)
        row_counts[y] += 1
        if x < min_x:
            min_x = x
        if x > max_x:
            max_x = x
        if y < min_y:
            min_y = y
        if y > max_y:
            max_y = y

    print(f"bbox=x[{min_x},{max_x}] y[{min_y},{max_y}]")
    nonzero_rows = [(y, c) for y, c in enumerate(row_counts) if c > 0]
    print(f"rows_with_diff={len(nonzero_rows)}")

    ranked = sorted(nonzero_rows, key=lambda t: (-t[1], t[0]))
    print("top_rows:")
    for y, c in ranked[: args.top_rows]:
        print(f"  y={y} diff_pixels={c}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())

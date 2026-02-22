#!/usr/bin/env python3
"""Compare Mesen2/NESium RGB24 frame dumps and report pixel-level deltas.

Usage example:
  uv run python tools/diff_rgb24_frames.py ^
    --mesen-prefix target/compare/mesen_flowing_palette ^
    --nesium-prefix target/compare/nesium_flowing_palette ^
    --frames 60,180,360,600
"""

from __future__ import annotations

import argparse
from collections import Counter
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, List, Sequence, Tuple


NES_WIDTH = 256
NES_HEIGHT = 240
BYTES_PER_PIXEL = 3


@dataclass
class FrameDiffStats:
    frame: int
    pixels_total: int
    pixels_diff: int
    pct_diff: float
    mean_abs_r: float
    mean_abs_g: float
    mean_abs_b: float
    max_abs_r: int
    max_abs_g: int
    max_abs_b: int
    bbox: Tuple[int, int, int, int] | None
    top_deltas: List[Tuple[Tuple[int, int, int], int]]
    top_rows: List[Tuple[int, int]]
    top_cols: List[Tuple[int, int]]


def parse_frames_csv(value: str) -> List[int]:
    out = []
    for token in value.split(","):
        token = token.strip()
        if not token:
            continue
        out.append(int(token))
    out = sorted(set(out))
    if len(out) < 2:
        raise ValueError(f"need at least 2 distinct frames, got {len(out)}")
    return out


def read_rgb24(path: Path, expected_bytes: int) -> bytes:
    data = path.read_bytes()
    if len(data) != expected_bytes:
        raise ValueError(
            f"{path}: rgb24 size mismatch, expected {expected_bytes}, got {len(data)}"
        )
    return data


def iter_pixels(buf: bytes) -> Iterable[Tuple[int, int, int]]:
    for i in range(0, len(buf), BYTES_PER_PIXEL):
        yield (buf[i], buf[i + 1], buf[i + 2])


def compute_frame_diff(frame: int, a: bytes, b: bytes, width: int, height: int) -> FrameDiffStats:
    pixels_total = width * height
    pixels_diff = 0

    sum_abs_r = 0
    sum_abs_g = 0
    sum_abs_b = 0
    max_abs_r = 0
    max_abs_g = 0
    max_abs_b = 0

    delta_counts: Counter[Tuple[int, int, int]] = Counter()
    row_counts = [0 for _ in range(height)]
    col_counts = [0 for _ in range(width)]

    min_x = width
    min_y = height
    max_x = -1
    max_y = -1

    for idx, (pa, pb) in enumerate(zip(iter_pixels(a), iter_pixels(b))):
        dr = pb[0] - pa[0]
        dg = pb[1] - pa[1]
        db = pb[2] - pa[2]
        if dr == 0 and dg == 0 and db == 0:
            continue

        pixels_diff += 1
        delta_counts[(dr, dg, db)] += 1

        ar = abs(dr)
        ag = abs(dg)
        ab = abs(db)
        sum_abs_r += ar
        sum_abs_g += ag
        sum_abs_b += ab
        if ar > max_abs_r:
            max_abs_r = ar
        if ag > max_abs_g:
            max_abs_g = ag
        if ab > max_abs_b:
            max_abs_b = ab

        y, x = divmod(idx, width)
        row_counts[y] += 1
        col_counts[x] += 1
        if x < min_x:
            min_x = x
        if y < min_y:
            min_y = y
        if x > max_x:
            max_x = x
        if y > max_y:
            max_y = y

    pct_diff = (pixels_diff / pixels_total * 100.0) if pixels_total else 0.0
    if pixels_diff == 0:
        bbox = None
        mean_abs_r = 0.0
        mean_abs_g = 0.0
        mean_abs_b = 0.0
    else:
        bbox = (min_x, min_y, max_x, max_y)
        mean_abs_r = sum_abs_r / pixels_diff
        mean_abs_g = sum_abs_g / pixels_diff
        mean_abs_b = sum_abs_b / pixels_diff

    top_deltas = delta_counts.most_common(10)
    top_rows = sorted(
        ((i, c) for i, c in enumerate(row_counts) if c > 0), key=lambda x: (-x[1], x[0])
    )[:10]
    top_cols = sorted(
        ((i, c) for i, c in enumerate(col_counts) if c > 0), key=lambda x: (-x[1], x[0])
    )[:10]

    return FrameDiffStats(
        frame=frame,
        pixels_total=pixels_total,
        pixels_diff=pixels_diff,
        pct_diff=pct_diff,
        mean_abs_r=mean_abs_r,
        mean_abs_g=mean_abs_g,
        mean_abs_b=mean_abs_b,
        max_abs_r=max_abs_r,
        max_abs_g=max_abs_g,
        max_abs_b=max_abs_b,
        bbox=bbox,
        top_deltas=top_deltas,
        top_rows=top_rows,
        top_cols=top_cols,
    )


def print_stats(stats: FrameDiffStats) -> None:
    print(
        "FRAME {f}: diff={d}/{t} ({p:.2f}%), mean_abs=(R:{mr:.2f}, G:{mg:.2f}, B:{mb:.2f}), "
        "max_abs=(R:{xr}, G:{xg}, B:{xb})".format(
            f=stats.frame,
            d=stats.pixels_diff,
            t=stats.pixels_total,
            p=stats.pct_diff,
            mr=stats.mean_abs_r,
            mg=stats.mean_abs_g,
            mb=stats.mean_abs_b,
            xr=stats.max_abs_r,
            xg=stats.max_abs_g,
            xb=stats.max_abs_b,
        )
    )
    if stats.bbox is None:
        print("  bbox: none (frames identical)")
    else:
        x0, y0, x1, y1 = stats.bbox
        print(f"  bbox: x={x0}..{x1}, y={y0}..{y1}")

    if stats.top_deltas:
        top = ", ".join(
            f"(dr={d[0]},dg={d[1]},db={d[2]}):{c}" for d, c in stats.top_deltas[:6]
        )
        print(f"  top_deltas: {top}")
    if stats.top_rows:
        top = ", ".join(f"y{y}:{c}" for y, c in stats.top_rows[:6])
        print(f"  top_rows: {top}")
    if stats.top_cols:
        top = ", ".join(f"x{x}:{c}" for x, c in stats.top_cols[:6])
        print(f"  top_cols: {top}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mesen-prefix", required=True, help="Prefix for Mesen rgb24 dumps")
    parser.add_argument("--nesium-prefix", required=True, help="Prefix for NESium rgb24 dumps")
    parser.add_argument(
        "--frames", default="60,180,360,600", help="CSV frame list (at least 2 distinct)"
    )
    parser.add_argument("--width", type=int, default=NES_WIDTH)
    parser.add_argument("--height", type=int, default=NES_HEIGHT)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    frames = parse_frames_csv(args.frames)

    expected_bytes = args.width * args.height * BYTES_PER_PIXEL
    mesen_prefix = Path(args.mesen_prefix)
    nesium_prefix = Path(args.nesium_prefix)

    all_stats: List[FrameDiffStats] = []
    for frame in frames:
        mesen_path = Path(f"{mesen_prefix}_f{frame}.rgb24")
        nesium_path = Path(f"{nesium_prefix}_f{frame}.rgb24")
        if not mesen_path.exists():
            raise FileNotFoundError(f"missing mesen dump: {mesen_path}")
        if not nesium_path.exists():
            raise FileNotFoundError(f"missing nesium dump: {nesium_path}")

        mesen = read_rgb24(mesen_path, expected_bytes)
        nesium = read_rgb24(nesium_path, expected_bytes)
        stats = compute_frame_diff(frame, mesen, nesium, args.width, args.height)
        all_stats.append(stats)
        print_stats(stats)

    total_pixels = sum(s.pixels_total for s in all_stats)
    total_diff = sum(s.pixels_diff for s in all_stats)
    pct = (total_diff / total_pixels * 100.0) if total_pixels else 0.0
    print(
        "TOTAL: diff={d}/{t} ({p:.2f}%) across {n} frames".format(
            d=total_diff, t=total_pixels, p=pct, n=len(all_stats)
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

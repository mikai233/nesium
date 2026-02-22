#!/usr/bin/env python3
"""Locate the first per-pixel palette-index/emphasis mismatch.

This compares:
1) Mesen RGB24 frame dumps (`..._f{frame}.rgb24`)
2) NESium canonical planes (`..._f{frame}.idx8` + `..._f{frame}.emph8`)

Mesen RGB is reverse-mapped to candidate `(index, emphasis)` pairs using the
Mesen 2C02 palette plus the same emphasis attenuation model used by NESium's
default post-process path.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Sequence, Tuple


NES_WIDTH = 256
NES_HEIGHT = 240
RGB24_BPP = 3


# Mesen2 RP2C02 palette (same values as crates/nesium-core/src/ppu/palette.rs).
MESEN_2C02_64: Sequence[Tuple[int, int, int]] = (
    (0x66, 0x66, 0x66),
    (0x00, 0x2A, 0x88),
    (0x14, 0x12, 0xA7),
    (0x3B, 0x00, 0xA4),
    (0x5C, 0x00, 0x7E),
    (0x6E, 0x00, 0x40),
    (0x6C, 0x06, 0x00),
    (0x56, 0x1D, 0x00),
    (0x33, 0x35, 0x00),
    (0x0B, 0x48, 0x00),
    (0x00, 0x52, 0x00),
    (0x00, 0x4F, 0x08),
    (0x00, 0x40, 0x4D),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
    (0xAD, 0xAD, 0xAD),
    (0x15, 0x5F, 0xD9),
    (0x42, 0x40, 0xFF),
    (0x75, 0x27, 0xFE),
    (0xA0, 0x1A, 0xCC),
    (0xB7, 0x1E, 0x7B),
    (0xB5, 0x31, 0x20),
    (0x99, 0x4E, 0x00),
    (0x6B, 0x6D, 0x00),
    (0x38, 0x87, 0x00),
    (0x0C, 0x93, 0x00),
    (0x00, 0x8F, 0x32),
    (0x00, 0x7C, 0x8D),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
    (0xFF, 0xFE, 0xFF),
    (0x64, 0xB0, 0xFF),
    (0x92, 0x90, 0xFF),
    (0xC6, 0x76, 0xFF),
    (0xF3, 0x6A, 0xFF),
    (0xFE, 0x6E, 0xCC),
    (0xFE, 0x81, 0x70),
    (0xEA, 0x9E, 0x22),
    (0xBC, 0xBE, 0x00),
    (0x88, 0xD8, 0x00),
    (0x5C, 0xE4, 0x30),
    (0x45, 0xE0, 0x82),
    (0x48, 0xCD, 0xDE),
    (0x4F, 0x4F, 0x4F),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
    (0xFF, 0xFE, 0xFF),
    (0xC0, 0xDF, 0xFF),
    (0xD3, 0xD2, 0xFF),
    (0xE8, 0xC8, 0xFF),
    (0xFB, 0xC2, 0xFF),
    (0xFE, 0xC4, 0xEA),
    (0xFE, 0xCC, 0xC5),
    (0xF7, 0xD8, 0xA5),
    (0xE4, 0xE5, 0x94),
    (0xCF, 0xEF, 0x96),
    (0xBD, 0xF4, 0xAB),
    (0xB3, 0xF3, 0xCC),
    (0xB5, 0xEB, 0xF2),
    (0xB8, 0xB8, 0xB8),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
)


@dataclass
class FirstMismatch:
    frame: int
    pixel_index: int
    x: int
    y: int
    mesen_rgb: Tuple[int, int, int]
    candidates: Sequence[Tuple[int, int]]
    actual_idx: int
    actual_emph: int
    reason: str


def parse_frames_csv(value: str) -> List[int]:
    frames: List[int] = []
    for token in value.split(","):
        token = token.strip()
        if not token:
            continue
        frames.append(int(token))
    frames = sorted(set(frames))
    if not frames:
        raise ValueError("frame list must not be empty")
    return frames


def apply_emphasis(rgb: Tuple[int, int, int], color_index: int, emphasis: int) -> Tuple[int, int, int]:
    emphasis &= 0x07
    if emphasis == 0 or (color_index & 0x0F) > 0x0D:
        return rgb

    r = float(rgb[0])
    g = float(rgb[1])
    b = float(rgb[2])
    if (emphasis & 0x01) != 0:
        g *= 0.84
        b *= 0.84
    if (emphasis & 0x02) != 0:
        r *= 0.84
        b *= 0.84
    if (emphasis & 0x04) != 0:
        r *= 0.84
        g *= 0.84

    # Match Rust `as u8` semantics after clamping: truncate toward zero.
    return (int(max(0.0, min(255.0, r))), int(max(0.0, min(255.0, g))), int(max(0.0, min(255.0, b))))


def build_inverse_rgb_map() -> Dict[Tuple[int, int, int], List[Tuple[int, int]]]:
    inverse: Dict[Tuple[int, int, int], List[Tuple[int, int]]] = {}
    for idx in range(64):
        base = MESEN_2C02_64[idx]
        for emph in range(8):
            rgb = apply_emphasis(base, idx, emph)
            inverse.setdefault(rgb, []).append((idx, emph))
    return inverse


def load_rgb24(path: Path, width: int, height: int) -> bytes:
    data = path.read_bytes()
    expected = width * height * RGB24_BPP
    if len(data) != expected:
        raise ValueError(f"{path}: expected {expected} bytes, got {len(data)}")
    return data


def load_plane(path: Path, width: int, height: int) -> bytes:
    data = path.read_bytes()
    expected = width * height
    if len(data) != expected:
        raise ValueError(f"{path}: expected {expected} bytes, got {len(data)}")
    return data


def find_first_mismatch(
    frame: int,
    mesen_rgb24: bytes,
    nesium_idx: bytes,
    nesium_emph: bytes,
    inverse: Dict[Tuple[int, int, int], List[Tuple[int, int]]],
    width: int,
) -> Tuple[Optional[FirstMismatch], int, int, int]:
    unknown_colors = 0
    idx_mismatches = 0
    emph_mismatches = 0

    for i in range(len(nesium_idx)):
        rgb = (
            mesen_rgb24[i * 3],
            mesen_rgb24[i * 3 + 1],
            mesen_rgb24[i * 3 + 2],
        )
        candidates = inverse.get(rgb)
        if not candidates:
            unknown_colors += 1
            continue

        actual_idx = nesium_idx[i] & 0x3F
        actual_emph = nesium_emph[i] & 0x07
        if (actual_idx, actual_emph) in candidates:
            continue

        expected_idxs = {idx for idx, _ in candidates}
        if actual_idx not in expected_idxs:
            idx_mismatches += 1
            reason = "palette_index"
        else:
            emph_mismatches += 1
            reason = "emphasis_bits"

        y, x = divmod(i, width)
        return (
            FirstMismatch(
                frame=frame,
                pixel_index=i,
                x=x,
                y=y,
                mesen_rgb=rgb,
                candidates=candidates,
                actual_idx=actual_idx,
                actual_emph=actual_emph,
                reason=reason,
            ),
            unknown_colors,
            idx_mismatches,
            emph_mismatches,
        )

    return None, unknown_colors, idx_mismatches, emph_mismatches


def fmt_candidates(candidates: Iterable[Tuple[int, int]], limit: int = 8) -> str:
    out = []
    for idx, emph in list(candidates)[:limit]:
        out.append(f"(idx={idx:02X},emph={emph})")
    return ", ".join(out)


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser()
    p.add_argument("--mesen-prefix", required=True, help="Prefix for Mesen RGB24 dump files")
    p.add_argument("--nesium-prefix", required=True, help="Prefix for NESium idx8/emph8 dump files")
    p.add_argument("--frames", default="60", help="CSV frame list")
    p.add_argument("--width", type=int, default=NES_WIDTH)
    p.add_argument("--height", type=int, default=NES_HEIGHT)
    return p.parse_args()


def main() -> int:
    args = parse_args()
    frames = parse_frames_csv(args.frames)
    inverse = build_inverse_rgb_map()

    found_any = False
    for frame in frames:
        mesen_path = Path(f"{args.mesen_prefix}_f{frame}.rgb24")
        idx_path = Path(f"{args.nesium_prefix}_f{frame}.idx8")
        emph_path = Path(f"{args.nesium_prefix}_f{frame}.emph8")
        if not mesen_path.exists():
            raise FileNotFoundError(f"missing Mesen frame: {mesen_path}")
        if not idx_path.exists():
            raise FileNotFoundError(f"missing NESium index frame: {idx_path}")
        if not emph_path.exists():
            raise FileNotFoundError(f"missing NESium emphasis frame: {emph_path}")

        mesen_rgb24 = load_rgb24(mesen_path, args.width, args.height)
        nesium_idx = load_plane(idx_path, args.width, args.height)
        nesium_emph = load_plane(emph_path, args.width, args.height)

        mismatch, unknown_colors, idx_mismatches, emph_mismatches = find_first_mismatch(
            frame=frame,
            mesen_rgb24=mesen_rgb24,
            nesium_idx=nesium_idx,
            nesium_emph=nesium_emph,
            inverse=inverse,
            width=args.width,
        )

        if mismatch is None:
            print(
                f"FRAME {frame}: no mismatch found (unknown_colors={unknown_colors}, "
                f"idx_mismatches={idx_mismatches}, emph_mismatches={emph_mismatches})"
            )
            continue

        found_any = True
        print(
            "FRAME {f}: first mismatch at (x={x}, y={y}, pixel={p}) reason={reason}".format(
                f=mismatch.frame,
                x=mismatch.x,
                y=mismatch.y,
                p=mismatch.pixel_index,
                reason=mismatch.reason,
            )
        )
        print(
            f"  mesen_rgb={mismatch.mesen_rgb} -> candidates: {fmt_candidates(mismatch.candidates)}"
        )
        print(
            "  nesium_actual=(idx={idx:02X}, emph={emph})".format(
                idx=mismatch.actual_idx, emph=mismatch.actual_emph
            )
        )
        print(
            f"  counters_before_first: unknown_colors={unknown_colors}, "
            f"idx_mismatches={idx_mismatches}, emph_mismatches={emph_mismatches}"
        )

    return 1 if found_any else 0


if __name__ == "__main__":
    raise SystemExit(main())


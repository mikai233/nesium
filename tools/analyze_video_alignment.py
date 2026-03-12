#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import os
import re
import subprocess
import sys
from pathlib import Path


def run_cmd(
    cmd: list[str], cwd: Path, env: dict[str, str] | None = None
) -> subprocess.CompletedProcess[str]:
    merged = os.environ.copy()
    if env:
        merged.update(env)
    proc_raw = subprocess.run(
        cmd,
        cwd=str(cwd),
        env=merged,
        capture_output=True,
    )
    stdout = proc_raw.stdout.decode("utf-8", errors="replace")
    stderr = proc_raw.stderr.decode("utf-8", errors="replace")
    proc = subprocess.CompletedProcess(
        args=proc_raw.args,
        returncode=proc_raw.returncode,
        stdout=stdout,
        stderr=stderr,
    )
    if proc.returncode != 0:
        sys.stderr.write(stdout)
        sys.stderr.write(stderr)
        raise RuntimeError(f"command failed ({proc.returncode}): {' '.join(cmd)}")
    return proc


def make_safe_rom_key(rom_path: str) -> str:
    stem = Path(rom_path).stem
    safe = "".join(ch if (ch.isascii() and (ch.isalnum() or ch in "._-")) else "_" for ch in stem)
    if safe.strip("_"):
        return safe
    return "rom_" + hashlib.sha1(rom_path.encode("utf-8")).hexdigest()[:8]


def build_frames(start: int, end: int, step: int) -> list[int]:
    if start < 0 or end <= start:
        raise RuntimeError(f"invalid frame range [{start}..{end})")
    if step <= 0:
        raise RuntimeError(f"invalid frame step {step}")
    frames = list(range(start, end, step))
    if len(frames) < 2:
        raise RuntimeError(
            f"need at least 2 frames to compare, got {len(frames)} from [{start}..{end}) step {step}"
        )
    return frames


def clean_mesen_persistent_state(rom_path: Path) -> None:
    mesen_home = Path.home() / "Documents" / "Mesen2"
    saves_dir = mesen_home / "Saves"
    states_dir = mesen_home / "SaveStates"
    base = rom_path.stem

    save_path = saves_dir / f"{base}.sav"
    if save_path.exists():
        save_path.unlink()

    if states_dir.exists():
        for p in states_dir.glob(f"{base}*.mss"):
            try:
                p.unlink()
            except FileNotFoundError:
                pass


def fnv1a32_rgb24(data: bytes) -> str:
    h = 0x811C9DC5
    for b in data:
        h ^= b
        h = (h * 0x01000193) & 0xFFFFFFFF
    return f"{h:08x}"


def parse_video_probe_hashes(stdout: str) -> dict[int, str]:
    re_line = re.compile(r"\[video-probe\]\s+frame=(\d+)\s+hash=([0-9a-fA-F]{8})")
    out: dict[int, str] = {}
    for line in stdout.splitlines():
        m = re_line.search(line)
        if not m:
            continue
        out[int(m.group(1))] = m.group(2).lower()
    if not out:
        raise RuntimeError("no [video-probe] hash lines found in Nesium output")
    return out


def parse_mesen_hashes(stdout: str) -> dict[int, str]:
    re_line = re.compile(r"RGBHASH\|frame=(\d+)\|hash=([0-9a-fA-F]{8})")
    out: dict[int, str] = {}
    for line in stdout.splitlines():
        m = re_line.search(line)
        if not m:
            continue
        out[int(m.group(1))] = m.group(2).lower()
    if not out:
        raise RuntimeError("no RGBHASH lines found in Mesen output")
    return out


def parse_mesen_hashes_file(path: Path) -> dict[int, str]:
    if not path.exists():
        raise RuntimeError(f"Mesen hash output missing: {path}")
    out: dict[int, str] = {}
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        line = line.strip()
        if not line or line.startswith("frame,"):
            continue
        parts = [p.strip() for p in line.split(",", 1)]
        if len(parts) != 2:
            continue
        if not parts[0].isdigit():
            continue
        if not re.fullmatch(r"[0-9a-fA-F]{8}", parts[1]):
            continue
        out[int(parts[0])] = parts[1].lower()
    if not out:
        raise RuntimeError(f"no valid hashes found in Mesen hash output: {path}")
    return out


def collect_mesen_hashes_from_rgb24(prefix: Path) -> dict[int, str]:
    out: dict[int, str] = {}
    pattern = re.compile(r"_f(\d+)\.rgb24$")
    for p in prefix.parent.glob(f"{prefix.name}_f*.rgb24"):
        m = pattern.search(p.name)
        if not m:
            continue
        frame = int(m.group(1))
        out[frame] = fnv1a32_rgb24(p.read_bytes())
    if not out:
        raise RuntimeError(f"no Mesen frame dumps found under {prefix.parent}")
    return out


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rom", required=True)
    parser.add_argument("--start-frame", type=int, default=0)
    parser.add_argument("--end-frame", type=int, default=600)
    parser.add_argument("--frame-step", type=int, default=1)
    parser.add_argument(
        "--mesen-dll",
        default=r"F:\CLionProjects\nesium\Mesen2\bin\win-x64\Release\Mesen.dll",
    )
    parser.add_argument(
        "--out-dir",
        default=r"F:\CLionProjects\nesium\target\compare\video_align_py",
    )
    parser.add_argument("--timeout-sec", type=int, default=900)
    parser.add_argument(
        "--mesen-hash-mode",
        choices=["log", "rgb24"],
        default="log",
        help="Mesen hash source: 'log' parses RGBHASH output (faster), 'rgb24' uses frame dumps.",
    )
    parser.add_argument("--input-events-csv")
    parser.add_argument(
        "--no-clean-mesen-state",
        action="store_true",
        help="Do not delete Mesen save/save-state files matching ROM basename before run.",
    )
    parser.add_argument(
        "--frame-shift",
        type=int,
        help="Compare Mesen frame F against Nesium frame F+shift. If omitted, auto-detect in [-4,4].",
    )
    parser.add_argument(
        "--force-zero-cpu-ram",
        action="store_true",
        help="Force-zero Mesen CPU RAM/PRG-RAM before capture (slow; use only when needed).",
    )
    parser.add_argument(
        "--no-force-zero-cpu-ram",
        action="store_true",
        help="Deprecated compatibility flag; force-zero is already disabled by default.",
    )
    args = parser.parse_args()

    repo = Path(__file__).resolve().parents[1]
    rom = Path(args.rom)
    if not rom.exists():
        raise RuntimeError(f"rom not found: {rom}")
    mesen_dll = Path(args.mesen_dll)
    if not mesen_dll.exists():
        raise RuntimeError(f"Mesen.dll not found: {mesen_dll}")

    requested_frames = build_frames(args.start_frame, args.end_frame, args.frame_step)
    frames_csv = ",".join(str(f) for f in requested_frames)

    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    key = make_safe_rom_key(str(rom))
    rom_dir = out_dir / key
    rom_dir.mkdir(parents=True, exist_ok=True)
    prefix = rom_dir / "mesen_frame"
    mesen_hash_out = rom_dir / "mesen_frame_hashes.csv"
    mesen_script = (
        "tools/mesen_dump_frame_hash.lua"
        if args.mesen_hash_mode == "log"
        else "tools/mesen_dump_frame_rgb.lua"
    )

    print(
        f"[1/3] dump Mesen frame hashes ({args.mesen_hash_mode}): {rom} [{args.start_frame}..{args.end_frame}) step={args.frame_step}"
    )
    if not args.no_clean_mesen_state:
        clean_mesen_persistent_state(rom)
    mesen_env = {
        "NESIUM_MESEN_RGB_FRAMES": frames_csv,
        "NESIUM_MESEN_TRACE_FRAMES": str(args.end_frame),
    }
    if args.mesen_hash_mode == "rgb24":
        mesen_env["NESIUM_MESEN_RGB_OUT_PREFIX"] = str(prefix)
    else:
        mesen_env["NESIUM_MESEN_HASH_OUT"] = str(mesen_hash_out)
    force_zero_cpu_ram = args.force_zero_cpu_ram and not args.no_force_zero_cpu_ram
    if force_zero_cpu_ram:
        mesen_env["NESIUM_MESEN_FORCE_ZERO_CPU_RAM"] = "1"
    if args.input_events_csv:
        mesen_env["NESIUM_MESEN_INPUT_EVENTS"] = args.input_events_csv

    mesen_proc = run_cmd(
        [
            "dotnet",
            str(mesen_dll),
            "--debug.scriptWindow.allowIoOsAccess=true",
            f"--timeout={args.timeout_sec}",
            "--testRunner",
            mesen_script,
            str(rom),
        ],
        cwd=repo,
        env=mesen_env,
    )
    mesen_log_path = rom_dir / "mesen_dump.log"
    mesen_log_path.write_text(mesen_proc.stdout + mesen_proc.stderr, encoding="utf-8")
    if args.mesen_hash_mode == "log":
        if mesen_hash_out.exists():
            mesen_hashes = parse_mesen_hashes_file(mesen_hash_out)
        else:
            mesen_hashes = parse_mesen_hashes(mesen_proc.stdout + mesen_proc.stderr)
    else:
        mesen_hashes = collect_mesen_hashes_from_rgb24(prefix)

    common_frames = sorted(set(requested_frames).intersection(mesen_hashes.keys()))
    if len(common_frames) < 2:
        available = sorted(mesen_hashes.keys())
        raise RuntimeError(
            f"too few overlapping frames between request and Mesen dump; requested={len(requested_frames)} "
            f"mesen_available={len(available)} sample_available={available[:8]}"
        )

    common_csv = ",".join(str(f) for f in common_frames)
    print(f"[2/3] dump Nesium frame hashes (frames={len(common_frames)})")
    nesium_env = {
        "NESIUM_VIDEO_PROBE_ROM": str(rom),
        "NESIUM_VIDEO_PROBE_FRAMES": common_csv,
    }
    if args.input_events_csv:
        nesium_env["NESIUM_VIDEO_PROBE_INPUT_EVENTS"] = args.input_events_csv
    nesium_proc = run_cmd(
        [
            "cargo",
            "test",
            "-p",
            "nesium-core",
            "--test",
            "video_probe",
            "video_rgb24_hash_probe",
            "--",
            "--ignored",
            "--nocapture",
        ],
        cwd=repo,
        env=nesium_env,
    )
    nesium_log_path = rom_dir / "nesium_probe.log"
    nesium_log_path.write_text(nesium_proc.stdout + nesium_proc.stderr, encoding="utf-8")
    nesium_hashes = parse_video_probe_hashes(nesium_proc.stdout + nesium_proc.stderr)

    print("[3/3] compare frame hashes")
    if args.frame_shift is None:
        best_shift = 0
        best_score = -1.0
        for shift in range(-4, 5):
            aligned = 0
            matches = 0
            for frame in common_frames:
                n = nesium_hashes.get(frame + shift)
                if n is None:
                    continue
                aligned += 1
                if mesen_hashes.get(frame) == n:
                    matches += 1
            if aligned == 0:
                continue
            score = matches / aligned
            if score > best_score:
                best_score = score
                best_shift = shift
        frame_shift = best_shift
    else:
        frame_shift = args.frame_shift

    first_mismatch: tuple[int, int, str, str] | None = None
    compared_aligned = 0
    for frame in common_frames:
        nesium_frame = frame + frame_shift
        if nesium_frame not in nesium_hashes:
            continue
        compared_aligned += 1
        m = mesen_hashes.get(frame)
        n = nesium_hashes.get(nesium_frame)
        if m != n:
            first_mismatch = (frame, nesium_frame, m or "<missing>", n or "<missing>")
            break

    print()
    print(f"ROM: {rom}")
    print(f"Requested frames: {len(requested_frames)}")
    print(f"Compared frames: {len(common_frames)}")
    print(f"Aligned compare frames: {compared_aligned}")
    print(f"Frame shift (Mesen F vs Nesium F+shift): {frame_shift:+d}")
    if common_frames and common_frames[0] != requested_frames[0]:
        print(
            f"Note: first comparable frame is {common_frames[0]} (requested starts at {requested_frames[0]})."
        )
    print(f"Artifacts: {rom_dir}")
    if first_mismatch is None:
        print("Result: hashes match for all compared frames.")
        return 0

    frame, nesium_frame, expected, actual = first_mismatch
    print(f"Result: mismatch at Mesen frame {frame} (Nesium frame {nesium_frame})")
    print(f"Mesen:  {expected}")
    print(f"Nesium: {actual}")
    return 2


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(1)

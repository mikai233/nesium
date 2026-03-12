#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import os
import subprocess
import sys
import wave
from dataclasses import dataclass
from pathlib import Path

import numpy as np


def run_cmd(cmd: list[str], cwd: Path, env: dict[str, str] | None = None) -> None:
    merged = os.environ.copy()
    if env:
        merged.update(env)
    proc = subprocess.run(cmd, cwd=str(cwd), env=merged, text=True)
    if proc.returncode != 0:
        raise RuntimeError(f"command failed ({proc.returncode}): {' '.join(cmd)}")


def read_wav_pcm16_stereo(path: Path, sample_rate: int) -> np.ndarray:
    with wave.open(str(path), "rb") as wf:
        if wf.getnchannels() != 2:
            raise ValueError(f"unexpected channels: {wf.getnchannels()} (expect 2)")
        if wf.getsampwidth() != 2:
            raise ValueError(f"unexpected sample width: {wf.getsampwidth()} (expect 2 bytes)")
        if wf.getframerate() != sample_rate:
            raise ValueError(f"unexpected sample rate: {wf.getframerate()} (expect {sample_rate})")
        frames = wf.readframes(wf.getnframes())
    return np.frombuffer(frames, dtype="<i2")


def read_raw_pcm16(path: Path) -> np.ndarray:
    data = path.read_bytes()
    if len(data) % 2 != 0:
        raise ValueError(f"raw pcm16 byte length must be even: {len(data)}")
    return np.frombuffer(data, dtype="<i2")


def trim_leading_silence(data: np.ndarray, threshold: int) -> tuple[np.ndarray, int]:
    if threshold <= 0 or data.size == 0:
        return data, 0

    idx = np.flatnonzero(np.abs(data) >= threshold)
    if idx.size == 0:
        return data, 0

    start = int(idx[0]) & ~1  # preserve stereo alignment
    return data[start:], start


@dataclass
class LagError:
    lag: int
    mae: float
    active: int
    count: int


def lag_error(
    mesen: np.ndarray,
    nesium: np.ndarray,
    lag: int,
    window: int,
    signal_threshold: int,
) -> LagError | None:
    if lag >= 0:
        m = mesen[lag:]
        n = nesium
    else:
        m = mesen
        n = nesium[-lag:]

    count = min(len(m), len(n))
    if count <= 0:
        return None

    if window > 0:
        count = min(count, window)
    if count <= 0:
        return None

    m = m[:count].astype(np.int32, copy=False)
    n = n[:count].astype(np.int32, copy=False)

    if signal_threshold > 0:
        mask = (np.abs(m) + np.abs(n)) >= signal_threshold
        active = int(mask.sum())
        if active == 0:
            return None
        diff = np.abs(m[mask] - n[mask])
    else:
        active = count
        diff = np.abs(m - n)

    mae = float(diff.mean())
    return LagError(lag=lag, mae=mae, active=active, count=count)


def find_best_lag(
    mesen: np.ndarray,
    nesium: np.ndarray,
    max_lag: int,
    coarse_step: int,
    window: int,
    signal_threshold: int,
    min_active: int,
) -> LagError:
    coarse_step = max(2, coarse_step)
    if coarse_step % 2:
        coarse_step += 1

    def better(candidate: LagError, current: LagError | None) -> bool:
        if current is None:
            return True
        if candidate.mae < current.mae:
            return True
        if candidate.mae > current.mae:
            return False
        if candidate.active > current.active:
            return True
        if candidate.active < current.active:
            return False
        return abs(candidate.lag) < abs(current.lag)

    best: LagError | None = None
    for lag in range(-max_lag, max_lag + 1, coarse_step):
        e = lag_error(mesen, nesium, lag, window, signal_threshold)
        if e is None or e.active < min_active:
            continue
        if better(e, best):
            best = e

    if best is None:
        raise RuntimeError("failed to find valid lag in coarse scan")

    fine_start = max(-max_lag, best.lag - coarse_step)
    fine_end = min(max_lag, best.lag + coarse_step)
    for lag in range(fine_start, fine_end + 1, 2):
        e = lag_error(mesen, nesium, lag, window, signal_threshold)
        if e is None or e.active < min_active:
            continue
        if better(e, best):
            best = e

    return best


def full_metrics(mesen: np.ndarray, nesium: np.ndarray, lag: int) -> tuple[int, float, float, float, float]:
    if lag >= 0:
        m = mesen[lag:]
        n = nesium
    else:
        m = mesen
        n = nesium[-lag:]
    count = min(len(m), len(n))
    if count <= 0:
        raise RuntimeError("no overlap after lag")
    m = m[:count].astype(np.int32, copy=False)
    n = n[:count].astype(np.int32, copy=False)
    diff = (m - n).astype(np.float64, copy=False)
    mae = float(np.mean(np.abs(diff)))
    rmse = float(np.sqrt(np.mean(diff * diff)))
    left = diff[0::2]
    right = diff[1::2]
    mae_l = float(np.mean(np.abs(left))) if left.size else 0.0
    mae_r = float(np.mean(np.abs(right))) if right.size else 0.0
    return count, mae, rmse, mae_l, mae_r


def make_safe_rom_key(rom_path: str) -> str:
    stem = Path(rom_path).stem
    safe = "".join(ch if (ch.isascii() and (ch.isalnum() or ch in "._-")) else "_" for ch in stem)
    if safe.strip("_"):
        return safe
    return "rom_" + hashlib.sha1(rom_path.encode("utf-8")).hexdigest()[:8]

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


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rom", required=True)
    parser.add_argument("--start-frame", type=int, default=0)
    parser.add_argument("--end-frame", type=int, default=600)
    parser.add_argument("--max-lag", type=int, default=4096)
    parser.add_argument("--coarse-step", type=int, default=32)
    parser.add_argument("--signal-threshold", type=int, default=8)
    parser.add_argument("--min-active", type=int, default=1024)
    parser.add_argument("--window", type=int, default=96000)
    parser.add_argument("--forced-lag", type=int)
    parser.add_argument("--input-file")
    parser.add_argument("--input-events-csv")
    parser.add_argument("--mesen-input-frame-offset", type=int, default=0)
    parser.add_argument("--nesium-input-frame-offset", type=int, default=0)
    parser.add_argument(
        "--mesen-dll",
        default=r"F:\CLionProjects\nesium\Mesen2\bin\win-x64\Release\Mesen.dll",
    )
    parser.add_argument(
        "--out-dir",
        default=r"F:\CLionProjects\nesium\target\compare\audio_align_py",
    )
    parser.add_argument("--timeout-sec", type=int, default=900)
    parser.add_argument("--sample-rate", type=int, default=48000)
    parser.add_argument(
        "--no-clean-mesen-state",
        action="store_true",
        help="Do not delete Mesen save/save-state files matching ROM basename before run.",
    )
    args = parser.parse_args()

    if args.start_frame < 0 or args.end_frame <= args.start_frame:
        raise RuntimeError("invalid frame range")

    repo = Path(__file__).resolve().parents[1]
    rom = Path(args.rom)
    if not rom.exists():
        raise RuntimeError(f"rom not found: {rom}")

    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    key = make_safe_rom_key(str(rom))
    rom_dir = out_dir / key
    rom_dir.mkdir(parents=True, exist_ok=True)

    mesen_work = out_dir / "_mesen_work"
    mesen_work.mkdir(parents=True, exist_ok=True)
    mesen_sha = rom_dir / f"mesen_{args.start_frame}_{args.end_frame}.sha1"
    nesium_raw = rom_dir / f"nesium_{args.start_frame}_{args.end_frame}.pcm16le"
    mesen_wav = mesen_work / "mesen_audio_capture.wav"

    if not args.no_clean_mesen_state:
        clean_mesen_persistent_state(rom)

    print(f"[1/3] dump Mesen wav: {rom} [{args.start_frame}..{args.end_frame})")
    baseline_cmd = [
        "powershell",
        "-ExecutionPolicy",
        "Bypass",
        "-File",
        "tools/gen_audio_baseline.ps1",
        "-RomPath",
        str(rom),
        "-StartFrame",
        str(args.start_frame),
        "-EndFrame",
        str(args.end_frame),
        "-OutFile",
        str(mesen_sha),
        "-MesenDllPath",
        str(Path(args.mesen_dll)),
        "-WorkDir",
        str(mesen_work),
        "-TimeoutSec",
        str(args.timeout_sec),
        "-SampleRate",
        str(args.sample_rate),
        "-AllowEmptyCapture",
    ]
    if args.input_file:
        baseline_cmd.extend(["-InputFile", str(args.input_file)])
    elif args.input_events_csv:
        baseline_cmd.extend(["-InputEventsCsv", str(args.input_events_csv)])
    if args.mesen_input_frame_offset != 0:
        baseline_cmd.extend(["-InputFrameOffset", str(args.mesen_input_frame_offset)])
    run_cmd(baseline_cmd, cwd=repo)
    if not mesen_wav.exists():
        raise RuntimeError(f"mesen wav missing: {mesen_wav}")

    print(f"[2/3] dump Nesium raw: {nesium_raw}")
    probe_env = {
        "NESIUM_AUDIO_PROBE_ROM": str(rom),
        "NESIUM_AUDIO_PROBE_START": str(args.start_frame),
        "NESIUM_AUDIO_PROBE_END": str(args.end_frame),
        "NESIUM_AUDIO_PROBE_RAW_OUT": str(nesium_raw),
        "NESIUM_AUDIO_PROBE_SAMPLE_RATE": str(args.sample_rate),
    }
    if args.input_file:
        probe_env["NESIUM_AUDIO_PROBE_INPUT_FILE"] = str(args.input_file)
    elif args.input_events_csv:
        probe_env["NESIUM_AUDIO_PROBE_INPUT_EVENTS"] = str(args.input_events_csv)
    if args.nesium_input_frame_offset != 0:
        probe_env["NESIUM_AUDIO_PROBE_INPUT_FRAME_OFFSET"] = str(
            args.nesium_input_frame_offset
        )

    run_cmd(
        [
            "cargo",
            "test",
            "-p",
            "nesium-core",
            "--test",
            "audio_probe",
            "audio_raw_dump_probe",
            "--",
            "--ignored",
            "--nocapture",
        ],
        cwd=repo,
        env=probe_env,
    )
    if not nesium_raw.exists():
        raise RuntimeError(f"nesium raw missing: {nesium_raw}")

    print("[3/3] analyze alignment")
    mesen = read_wav_pcm16_stereo(mesen_wav, args.sample_rate)
    nesium = read_raw_pcm16(nesium_raw)
    trim_threshold = max(1, args.signal_threshold)
    mesen_for_lag, mesen_trim_start = trim_leading_silence(mesen, trim_threshold)
    nesium_for_lag, nesium_trim_start = trim_leading_silence(nesium, trim_threshold)

    if args.forced_lag is not None:
        e = lag_error(
            mesen, nesium, args.forced_lag, args.window, args.signal_threshold
        )
        if e is None or e.active < args.min_active:
            raise RuntimeError(
                f"forced lag {args.forced_lag} invalid (active={0 if e is None else e.active})"
            )
        best = e
    else:
        best = find_best_lag(
            mesen=mesen_for_lag,
            nesium=nesium_for_lag,
            max_lag=args.max_lag,
            coarse_step=args.coarse_step,
            window=args.window,
            signal_threshold=args.signal_threshold,
            min_active=args.min_active,
        )
        # Convert lag from trimmed arrays back to full-array coordinates.
        best = LagError(
            lag=best.lag + (mesen_trim_start - nesium_trim_start),
            mae=best.mae,
            active=best.active,
            count=best.count,
        )

    overlap, mae, rmse, mae_l, mae_r = full_metrics(mesen, nesium, best.lag)
    print()
    print(f"ROM: {rom}")
    print(f"Range: [{args.start_frame}..{args.end_frame})")
    print(f"Samples: mesen={len(mesen)} nesium={len(nesium)}")
    print(f"Best lag (i16 samples, + means mesen delayed): {best.lag}")
    print(f"Window MAE: {best.mae:.3f} (active={best.active})")
    print(f"Full overlap samples: {overlap}")
    print(f"Full MAE: {mae:.3f}")
    print(f"Full RMSE: {rmse:.3f}")
    print(f"Channel MAE: left={mae_l:.3f} right={mae_r:.3f}")
    print(f"Artifacts: {rom_dir}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(1)

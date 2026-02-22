#!/usr/bin/env python3
"""Compare Mesen/NESium PPU register write traces.

Input format (one event per line):
  PPUREG|src=...|ev=write|cpu_cycle=...|frame=...|scanline=...|dot=...|addr=....|value=..
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from pathlib import Path
from typing import List, Tuple


@dataclass
class Event:
    raw: str
    src: str
    ev: str
    cpu_cycle: int
    frame: int
    scanline: int
    dot: int
    addr: int
    value: int
    v: int
    t: int
    x: int


def parse_line(line: str) -> Event | None:
    line = line.strip()
    if not line or not line.startswith("PPUREG|"):
        return None
    fields = {}
    for part in line.split("|")[1:]:
        if "=" not in part:
            continue
        k, v = part.split("=", 1)
        fields[k] = v
    if fields.get("ev") != "write":
        return None

    def get_int(name: str, base: int = 10) -> int:
        v = fields.get(name)
        if v is None:
            return -1
        try:
            return int(v, base)
        except ValueError:
            return -1

    return Event(
        raw=line,
        src=fields.get("src", ""),
        ev=fields.get("ev", ""),
        cpu_cycle=get_int("cpu_cycle"),
        frame=get_int("frame"),
        scanline=get_int("scanline"),
        dot=get_int("dot"),
        addr=get_int("addr", 16),
        value=get_int("value", 16),
        v=get_int("v", 16),
        t=get_int("t", 16),
        x=get_int("x", 16),
    )


def load_events(path: Path) -> List[Event]:
    events: List[Event] = []
    for line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
        evt = parse_line(line)
        if evt is not None:
            events.append(evt)
    return events


def event_field_value(e: Event, field: str) -> int:
    return {
        "cpu_cycle": e.cpu_cycle,
        "frame": e.frame,
        "scanline": e.scanline,
        "dot": e.dot,
        "addr": e.addr,
        "value": e.value,
        "v": e.v,
        "t": e.t,
        "x": e.x,
    }[field]


def first_mismatch(
    a: List[Event], b: List[Event], fields: List[str]
) -> Tuple[int, Event | None, Event | None]:
    n = min(len(a), len(b))
    for i in range(n):
        if any(event_field_value(a[i], f) != event_field_value(b[i], f) for f in fields):
            return i, a[i], b[i]
    if len(a) != len(b):
        i = n
        return i, (a[i] if i < len(a) else None), (b[i] if i < len(b) else None)
    return -1, None, None


def print_window(label: str, events: List[Event], center: int, radius: int) -> None:
    print(f"{label} window [{max(0, center-radius)}..{min(len(events)-1, center+radius)}]:")
    for i in range(max(0, center - radius), min(len(events), center + radius + 1)):
        e = events[i]
        mark = ">>" if i == center else "  "
        print(
            f"{mark}#{i:06d} frame={e.frame:4d} sl={e.scanline:4d} dot={e.dot:3d} "
            f"cpu={e.cpu_cycle:9d} addr={e.addr:04X} val={e.value:02X} "
            f"v={e.v:04X} t={e.t:04X} x={e.x:02X}"
        )


def main() -> int:
    parser = argparse.ArgumentParser(description="Compare PPU register traces")
    parser.add_argument("--mesen", required=True, help="Mesen trace path")
    parser.add_argument("--nesium", required=True, help="NESium trace path")
    parser.add_argument("--window", type=int, default=8, help="context radius around mismatch")
    parser.add_argument(
        "--fields",
        default="addr,value",
        help=(
            "Comma-separated fields to compare. "
            "Supported: cpu_cycle,frame,scanline,dot,addr,value,v,t,x"
        ),
    )
    parser.add_argument(
        "--strict-timing",
        action="store_true",
        help="Shortcut for --fields cpu_cycle,frame,scanline,dot,addr,value,v,t",
    )
    args = parser.parse_args()

    mesen_events = load_events(Path(args.mesen))
    nesium_events = load_events(Path(args.nesium))
    if args.strict_timing:
        fields = ["cpu_cycle", "frame", "scanline", "dot", "addr", "value", "v", "t"]
    else:
        fields = [f.strip() for f in args.fields.split(",") if f.strip()]
    valid_fields = {"cpu_cycle", "frame", "scanline", "dot", "addr", "value", "v", "t", "x"}
    invalid = [f for f in fields if f not in valid_fields]
    if invalid:
        parser.error(f"unsupported fields: {', '.join(invalid)}")
    if not fields:
        parser.error("at least one comparison field is required")

    print(f"mesen writes : {len(mesen_events)}")
    print(f"nesium writes: {len(nesium_events)}")
    print(f"compare fields: {','.join(fields)}")

    idx, a, b = first_mismatch(mesen_events, nesium_events, fields)
    if idx < 0:
        print("selected field sequence: exact match")
        return 0

    print(f"first divergence at event #{idx}")
    if a is None:
        print("mesen: <end of stream>")
    else:
        print(
            f"mesen:  frame={a.frame} sl={a.scanline} dot={a.dot} cpu={a.cpu_cycle} "
            f"addr={a.addr:04X} value={a.value:02X} v={a.v:04X} t={a.t:04X} x={a.x:02X}"
        )
    if b is None:
        print("nesium: <end of stream>")
    else:
        print(
            f"nesium: frame={b.frame} sl={b.scanline} dot={b.dot} cpu={b.cpu_cycle} "
            f"addr={b.addr:04X} value={b.value:02X} v={b.v:04X} t={b.t:04X} x={b.x:02X}"
        )

    print_window("mesen", mesen_events, idx, args.window)
    print_window("nesium", nesium_events, idx, args.window)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())

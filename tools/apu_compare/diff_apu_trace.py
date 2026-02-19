#!/usr/bin/env python3
"""Compare APU trace logs from Mesen2 and NESium and report first mismatch."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List


@dataclass
class TraceEvent:
    index: int
    ev: str
    addr: str
    value: str
    raw: str
    fields: Dict[str, str]


def parse_fields(line: str) -> Dict[str, str]:
    fields: Dict[str, str] = {}
    for segment in line.strip().split("|"):
        if "=" not in segment:
            continue
        key, value = segment.split("=", 1)
        fields[key] = value
    return fields


def normalize_hex(value: str, width: int) -> str:
    value = value.strip().lower()
    if value.startswith("0x"):
        value = value[2:]
    if value == "":
        return "0" * width
    return f"{int(value, 16):0{width}X}"


def parse_events(path: Path, include_read_mem: bool) -> List[TraceEvent]:
    events: List[TraceEvent] = []
    allowed_events = {"read", "write"}
    if include_read_mem:
        allowed_events.add("read_mem")
    with path.open("r", encoding="utf-8", errors="replace") as f:
        for line in f:
            if not line.startswith("APUTRACE|"):
                continue
            fields = parse_fields(line)
            ev = fields.get("ev", "")
            if ev not in allowed_events:
                continue

            addr_raw = fields.get("addr", "")
            if not addr_raw:
                continue
            addr = normalize_hex(addr_raw, 4)

            value_raw = fields.get("value", "00")
            value = normalize_hex(value_raw, 2)

            events.append(
                TraceEvent(
                    index=len(events),
                    ev=ev,
                    addr=addr,
                    value=value,
                    raw=line.rstrip("\n"),
                    fields=fields,
                )
            )
    return events


def print_context(name: str, events: List[TraceEvent], center: int, radius: int) -> None:
    start = max(0, center - radius)
    end = min(len(events), center + radius + 1)
    print(f"{name} context [{start}:{end}):")
    for idx in range(start, end):
        marker = ">>" if idx == center else "  "
        evt = events[idx]
        print(f"  {marker} #{idx:04d} {evt.ev} {evt.addr}={evt.value} | {evt.raw}")


def compare_events(
    mesen_events: List[TraceEvent], nesium_events: List[TraceEvent], context: int
) -> int:
    common = min(len(mesen_events), len(nesium_events))
    for idx in range(common):
        m = mesen_events[idx]
        n = nesium_events[idx]
        if (m.ev, m.addr, m.value) != (n.ev, n.addr, n.value):
            print("First mismatch found:")
            print(f"  index: {idx}")
            print(f"  mesen : {m.ev} {m.addr}={m.value}")
            print(f"  nesium: {n.ev} {n.addr}={n.value}")
            print()
            print_context("mesen", mesen_events, idx, context)
            print()
            print_context("nesium", nesium_events, idx, context)
            return 1

    if len(mesen_events) != len(nesium_events):
        print("Event count mismatch:")
        print(f"  mesen events : {len(mesen_events)}")
        print(f"  nesium events: {len(nesium_events)}")
        return 1

    print(f"No mismatch found across {common} comparable events.")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mesen", required=True, type=Path, help="Mesen trace log path")
    parser.add_argument("--nesium", required=True, type=Path, help="NESium trace log path")
    parser.add_argument(
        "--context",
        type=int,
        default=3,
        help="number of events to show before/after mismatch",
    )
    parser.add_argument(
        "--include-read-mem",
        action="store_true",
        help="include `ev=read_mem` events in the comparison",
    )
    args = parser.parse_args()

    mesen_events = parse_events(args.mesen, args.include_read_mem)
    nesium_events = parse_events(args.nesium, args.include_read_mem)

    print(f"Loaded comparable events: mesen={len(mesen_events)} nesium={len(nesium_events)}")
    return compare_events(mesen_events, nesium_events, args.context)


if __name__ == "__main__":
    raise SystemExit(main())

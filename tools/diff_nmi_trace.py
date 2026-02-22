#!/usr/bin/env python3
"""Compare NMI timing traces from Mesen2 and NESium and report first mismatch."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Tuple


@dataclass
class TraceEvent:
    index: int
    ev: str
    addr: Optional[str]
    value: Optional[str]
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


def normalize_event(src: str, ev: str) -> Optional[str]:
    if ev in {"read", "write"}:
        return ev
    if src == "mesen" and ev == "nmi_event":
        return "nmi_take"
    if src == "nesium" and ev == "nmi_take":
        return "nmi_take"
    return None


def parse_events(path: Path) -> List[TraceEvent]:
    events: List[TraceEvent] = []
    with path.open("r", encoding="utf-8", errors="replace") as f:
        for line in f:
            if not line.startswith("NMITRACE|"):
                continue
            fields = parse_fields(line)
            src = fields.get("src", "")
            ev = fields.get("ev", "")
            norm_ev = normalize_event(src, ev)
            if norm_ev is None:
                continue

            addr = fields.get("addr")
            value = fields.get("value")
            if norm_ev in {"read", "write"}:
                if addr is None or value is None:
                    continue
                addr = normalize_hex(addr, 4)
                value = normalize_hex(value, 2)
            else:
                addr = None
                value = None

            events.append(
                TraceEvent(
                    index=len(events),
                    ev=norm_ev,
                    addr=addr,
                    value=value,
                    raw=line.rstrip("\n"),
                    fields=fields,
                )
            )
    return events


def key_of(event: TraceEvent) -> Tuple[str, Optional[str], Optional[str]]:
    return event.ev, event.addr, event.value


def print_context(name: str, events: List[TraceEvent], center: int, radius: int) -> None:
    start = max(0, center - radius)
    end = min(len(events), center + radius + 1)
    print(f"{name} context [{start}:{end}):")
    for idx in range(start, end):
        marker = ">>" if idx == center else "  "
        evt = events[idx]
        av = (
            f"{evt.addr}={evt.value}"
            if evt.addr is not None and evt.value is not None
            else "-"
        )
        print(f"  {marker} #{idx:04d} {evt.ev} {av} | {evt.raw}")


def compare_events(mesen_events: List[TraceEvent], nesium_events: List[TraceEvent], context: int) -> int:
    common = min(len(mesen_events), len(nesium_events))
    for idx in range(common):
        m = mesen_events[idx]
        n = nesium_events[idx]
        if key_of(m) != key_of(n):
            print("First mismatch found:")
            print(f"  index: {idx}")
            print(f"  mesen : {key_of(m)}")
            print(f"  nesium: {key_of(n)}")
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
        default=4,
        help="number of events to show before/after mismatch",
    )
    args = parser.parse_args()

    mesen_events = parse_events(args.mesen)
    nesium_events = parse_events(args.nesium)
    print(f"Loaded comparable events: mesen={len(mesen_events)} nesium={len(nesium_events)}")
    return compare_events(mesen_events, nesium_events, args.context)


if __name__ == "__main__":
    raise SystemExit(main())

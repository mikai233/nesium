# Mapper / Expansion Audio Refactor Plan

## Goal

Perform a deliberately breaking refactor that moves the current mapper architecture away from:

- one large self-contained implementation per mapper
- mapper-local expansion audio implementations
- duplicated IRQ/banking logic across mapper variants

Toward:

- a stable top-level `Mapper` interface
- reusable chip-family cores for banking / IRQ / mapper-side timing
- reusable expansion audio chip implementations
- thin board-specific mapper wrappers responsible mainly for address wiring and configuration

The refactor is allowed to temporarily break compilation between phases. The objective is to reach a cleaner long-term structure, then immediately validate behavior against Mesen using the existing audio/video comparison tools.

## Non-Goals

- No compatibility shim for the old internal structure.
- No attempt to fully generalize all NES mappers into one universal base type.
- No large redesign of the public emulator runtime API unless required by mapper composition.

## Design Direction

Keep the current top-level `Mapper` trait as the integration boundary.

Refactor underneath it into two layers:

1. `board` layer
- concrete mapper files remain the final integration point
- responsible for board-specific address decode, register wiring, feature presence, and reset defaults

2. `core/chip` layer
- reusable banking/IRQ cores for mapper families
- reusable expansion audio chip implementations
- no direct knowledge of ROM header mapper IDs

This avoids building a single giant `BaseMapperState`, while still removing duplicated logic.

## Target Structure

Proposed internal split:

- `cartridge/mapper/core/`
  - reusable mapper-family logic
  - examples: `vrc_irq`, `vrc_core`, `mmc3_core`, later `namco163_core`

- `apu/expansion/`
  - reusable expansion audio chips
  - examples: `vrc6`, `vrc7`, `sunsoft5b`, `namco163`, `mmc5`, `fds`

- `cartridge/mapper/mapperXX.rs`
  - thin wrappers that combine:
    - ROM/RAM storage
    - one core or a small set of cores
    - one optional expansion audio chip
    - board-specific decode/wiring rules

## Phases

## Current Status

- Phase 1: completed
- Phase 2: completed
- Phase 3: completed
- Phase 4: current cleanup phase

Validated mapper 85 regressions against Mesen with the restored input-recording
audio/video tooling:

- `Lagrange Point`
  - video frame hashes aligned
  - PCM comparison aligned
  - VRC7 expansion audio observed active on the recorded input path
- `Tiny Toon Adventures 2 - Montana Land he Youkoso`
  - video frame hashes aligned
  - PCM comparison aligned
- `兔宝宝历险记2`
  - video frame hashes aligned
  - PCM comparison aligned

### Phase 1: Extract reusable expansion audio chips

Objective:
- move mapper-local audio implementations out of concrete mapper files

Scope:
- extract chip state/clock/output logic from:
  - mapper 19 `Namco163`
  - mapper 69 `Sunsoft 5B`
  - later mapper 85 `VRC7`
- keep the existing `ExpansionAudio` trait for now
- mapper files should hold audio chip instances instead of embedding chip logic directly

Deliverable:
- `apu/expansion/namco163.rs`
- `apu/expansion/sunsoft5b.rs`
- `apu/expansion/vrc7.rs` scaffold, even if not complete yet

Stop condition:
- mapper 19 and mapper 69 compile and still produce identical deltas to pre-refactor behavior

Regression after phase:
- audio comparison:
  - `赤影战士`
  - `吉米克`
  - `热血格斗传说`
  - one mapper 19 title
- if a suitable ROM is available, also run one mapper 85/VRC7 case

### Phase 2: Extract Konami family cores

Objective:
- remove duplicated Konami banking/IRQ logic from mapper 21/23/25/26/85

Scope:
- extract a reusable `VrcIrq`
- extract a reusable `VrcCore` or equivalent split:
  - PRG banking state
  - CHR banking state
  - mirroring control
  - address translation hooks
- keep board wrappers responsible for:
  - address line wiring
  - submapper heuristics
  - whether audio exists
  - whether a given register range is meaningful

Deliverable:
- mapper 21/23/25/26/85 share a common Konami-family internal core

Stop condition:
- these mappers no longer each maintain their own full IRQ/banking implementation

Regression after phase:
- video/audio compare for:
  - mapper 26 known-good ROM
  - mapper 85 ROM if available
  - existing MMC3/Namco163/Sunsoft5B baseline ROMs to guard against accidental hook/mixer regressions

### Phase 3: Rebuild mapper 85 around `VrcCore + Vrc7Audio`

Status:
- completed

Objective:
- remove the remaining README caveat for mapper 85 by wiring real VRC7 audio

Scope:
- implement VRC7 OPLL-backed expansion audio chip as a reusable expansion module
- attach it to mapper 85 through the same expansion-audio path used by other chips
- align register semantics, timing, and mixer output against Mesen

Deliverable:
- mapper 85 no longer treats audio registers as muted placeholders
- README caveat for VRC7 can be removed only after regression parity is demonstrated

Outcome:
- native `emu2413`-backed `Vrc7Audio` is wired through the reusable
  `ExpansionAudio` path
- mapper 85 audio register writes now drive the real OPLL path
- README caveat for mapper 85 / VRC7 has been removed after Mesen parity checks

Stop condition:
- at least one mapper 85 title matches Mesen closely enough in both:
  - frame hash / RGB dump
  - PCM comparison

Result:
- satisfied

Regression after phase:
- mapper 85 target ROM:
  - `Lagrange Point`
- also rerun:
  - `赤影战士`
  - `吉米克`
  - `热血格斗传说`
  - one mapper 19 title

### Phase 4: Cleanup and convergence

Objective:
- remove temporary glue introduced during the breaking refactor

Scope:
- delete obsolete mapper-local audio structs
- collapse duplicated helper code that became unnecessary
- simplify mapper docs/comments to match the new structure
- ensure README caveats only reflect real remaining gaps

Stop condition:
- codebase builds cleanly
- final structure is understandable without transitional comments everywhere

Regression after phase:
- run the full currently-available audio/video comparison set
- run normal cargo tests and selected ROM suites

## Working Rules During Refactor

1. Prefer structural progress over temporary build cleanliness
- it is acceptable for intermediate commits to fail to compile inside a phase
- each phase should end in a restorable, testable state

2. Keep board wrappers thin
- if a board file starts re-implementing core logic again, stop and move that logic down

3. Keep chip logic independent from mapper IDs
- expansion audio modules should not know which mapper number owns them

4. Keep timing sources explicit
- CPU bus events, CPU clocks, and PPU events should remain separate concepts
- avoid reintroducing ambiguous mixed hooks

5. Do not remove a README caveat until Mesen comparison says it is safe

## Immediate Execution Order

Start with Phase 1.

Reason:
- it gives the cleanest architectural win with the smallest blast radius
- it is also the prerequisite for making mapper 85 audio real without embedding another chip model directly in the mapper file

Once Phase 1 is stable, proceed directly to Phase 2, then Phase 3.

# NESium vs Mesen2 Gap Tracking

## 1. Purpose

- This document tracks only high-value status: validated milestones, current blockers, and next actions.
- Keep solved items as short summaries.
- Keep detailed traces/logs in scripts and output artifacts, not in this document.

## 2. Snapshot (2026-02-22)

- Mesen2 comparison workflow is operational (`--testRunner + Lua` + Python diff scripts).
- `cargo test -p nesium-core` currently reports:
  - Core tests: `72 passed / 0 failed / 3 ignored`
  - ROM suites: `41 passed / 0 failed / 19 ignored`
- Major APU and MMC3 suites that were previously failing are now passing by default (see Section 5).
- Remaining work is concentrated in incomplete APU corner behavior and platform/coverage gaps (PAL/expansion audio).

## 3. Completed Milestones (Condensed)

- APU IRQ semantics aligned:
  - `$4015` read clears frame IRQ only.
  - DMC IRQ is cleared by `$4015` write.
- DMC timing alignment (major round complete):
  - NTSC rate table fix (`idx=13 -> 84`), enable/disable delay, DMA abort/request ordering, DMA-complete update timing.
- Length counter timing aligned to delayed-commit model:
  - pending `halt`, delayed `reload`, same-cycle conflict handling.
- CPU DMA internal-register glitch handling improved:
  - single-steal dual read behavior, open-bus behavior for `$4000-$401F`, `$4016/$4017` suppression/merge behavior.
- MMC3 IRQ behavior converged:
  - Mapper4 revision auto-detection (`NESIUM_MMC3_IRQ_REV=A|B|AUTO`), and one-of-two pass policy for RevA/RevB-exclusive tests.
- NMI trace pipeline established and validated for timing/register sequences (CPU-side behavior aligns for compared windows).

## 4. Open Gaps (Current)

### P0

1. DMC one-byte sample glitch coverage
- `sample_length == 1` edge behavior is still not fully modeled.
- Prior experimental branch did not converge and was reverted.

### P1

1. Noise timer period edge semantics
- Potential off-by-one (`-1`) mismatch vs Mesen2 timing model.

2. Triangle channel behavior differences
- `$400B` write sequence-position handling mismatch.
- Output-hold behavior after gating mismatch.

3. Frame-counter residual risk
- Major frame-counter refactor landed, but this area remains timing-sensitive and needs continued regression protection.

### P2

1. PAL/Dendy coverage is incomplete
- PAL-specific APU tables/timing are not fully implemented yet.

2. Expansion-audio coverage gap
- Current implementation scope is narrower than Mesen2's expansion-audio coverage.

## 5. Regression Baselines To Keep

- APU suites now passing:
  - `apu_mixer_suite`
  - `apu_reset_suite`
  - `apu_test_suite`
  - `blargg_apu_2005_07_30_suite`
  - `dmc_tests_suite` (Mesen2 RAM baseline)
  - `dmc_dma_during_read4_suite` (Mesen2 serial baseline)
  - `sprdma_and_dmc_dma_suite`
  - `full_palette_suite` (Mesen2 RGB24 multi-frame baseline)
  - `scanline_suite` (Mesen2 RGB24 multi-frame baseline)
- MMC3 suites now passing:
  - `mmc3_irq_tests_suite` (RevA/RevB judged with one-of-two rule)
  - `mmc3_test_suite`
  - `mmc3_test_2_suite`
- NMI baseline tracking kept as diagnostics:
  - `nmi_sync_ntsc_mesen_baseline` is enabled in default test runs.
  - Last verified behavior: stable alternating Mesen2 hashes in tracked frame windows.

## 6. Build/Environment Notes (Minimal)

- On this machine, Mesen2 UI build requires disabling vcpkg autolink:

```powershell
msbuild Mesen2\Mesen.sln /restore /m /t:UI /p:Configuration=Release /p:Platform=x64 /p:VcpkgAutoLink=false
```

- Root cause of prior duplicate Lua symbols: local Lua + globally injected vcpkg Lua library conflict.

## 7. Next Actions

1. Re-open DMC one-byte glitch with dual-end trace comparison (`read_mem` enabled where needed).
2. Decide PAL scope (defer or implement) and split PAL-specific tests accordingly.
3. Keep this document concise: move newly solved items from Section 4 to Section 3 immediately.

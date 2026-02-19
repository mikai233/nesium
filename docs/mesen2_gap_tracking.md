# NESium vs Mesen2 关键差距追踪

## 1. 文档原则

- 只记录关键节点、已验证结论、当前阻塞和下一步关键动作。
- 不记录逐次操作流水账。
- 已解决问题仅保留一行摘要，细节日志不在此文档展开。

## 2. 当前状态（2026-02-19）

- Mesen2 `Core` 可构建（`Release/x64`）。
- Mesen2 `UI` 可构建，但在当前机器需禁用 vcpkg 自动注入库：`/p:VcpkgAutoLink=false`。
- `--testRunner + Lua` 路线可用于后续参考日志生成。
- 双端 APU 事件对比链路已打通：
  - Mesen2: `tools/apu_compare/mesen_trace_irq.lua`
  - NESium: `NESIUM_APU_TRACE_PATH` 控制的 `apu.rs` 埋点
  - 对齐脚本: `tools/apu_compare/diff_apu_trace.py`（建议用 `uv run python`）
- 在 `apu_test/rom_singles/6-irq_flag_timing.nes` 上，`$4015/$4017` 可比对事件序列已对齐（31/31）。
- NESium 与 Mesen2 APU 的关键差异已整理（P0/P1/P2）；“同 ROM、同窗口”日志对齐已完成首个关键样例，尚未覆盖其余 P0/P1 场景。
- DMC 已完成一轮关键对齐（非最终）：
  - `$4015` 读只清 frame IRQ，DMC IRQ 改为 `$4015` 写清。
  - DMC NTSC rate 表 `idx=13` 修正为 `84`。
  - `$4015` enable/disable 引入 2/3 cycle 延迟与 DMA abort/request 时序。
  - `bytesRemaining` 递减与 loop/IRQ 触发迁移到 DMA 完成时刻。
  - `bitsRemaining` 回卷逻辑改为与 Mesen 一致（不再在 buffer 空时卡在 0）。
- DMC+CPU DMA 新里程碑（2026-02-19）：
  - `ProcessPendingDma` 的 GET/PUT 判定改为按“当前 CPU cycle”分类（对齐 Mesen 时序）。
  - `$4015` enable/disable 的 2/3 cycle 奇偶判定改为直接使用写入时 CPU cycle（去除 `-1` 偏移）。
  - `$4016/$4017` 端口读改为“bit0 + open-bus 掩码合成”（`0xE0`），不再固定返回 `0x40` 高位。
- Length Counter 时序新里程碑（2026-02-19）：
  - `halt` 由“立即生效”改为 pending 并在 frame-counter clock 之后提交（half-frame 先用旧值，再提交新值）。
  - `reload` 改为延迟提交，并引入 `previous` 比较以处理“与 length clock 同周期写入”的忽略/生效语义。
  - `blargg_apu_2005.07.30` 的 `10.len_halt_timing`、`11.len_reload_timing` 已通过。
- CPU DMA internal-reg glitch 已完成第一轮对齐（非最终）：
  - `ProcessDmaRead` 改为单偷周期内完成 internal/external 双读。
  - `!enableInternalRegReads` 时，`$4000-$401F` 返回 open bus。
  - `$4016/$4017` 连续读抑制与 open-bus merge 公式已补齐。
- `$4017` 写入的 frame IRQ 清除改为仅 `bit6=1` 时立即清除（避免“无条件清除”带来的行为偏差）。
- 双端对比能力扩展：
  - NESium 新增 `NESIUM_APU_TRACE_READ_ADDRS`（输出 `ev=read_mem`）用于与 Mesen `read_mem` 事件逐项比对。
  - `tools/apu_compare/diff_apu_trace.py` 新增 `--include-read-mem` 开关。
- MMC3 IRQ 关键节点（2026-02-19）：
  - `mmc3_irq_tests` 的 `1-4` 已通过（clocking/details/A12/scanline）。
  - `5.MMC3_rev_A` 与 `6.MMC3_rev_B` 属于互斥硬件行为测试（官方 README 明确“最多通过一个”）。
  - Mapper4 已实现 IRQ revision 自动识别：
    - NES2 `submapper=1` 使用 RevA 语义；
    - 其余默认 RevB；
    - 可用 `NESIUM_MMC3_IRQ_REV=A|B|AUTO` 强制覆盖（用于 iNES/对比实验）。
  - 测试套件已改为 `5/6` 至少通过一个，避免把互斥 revision 行为误判为统一失败。
  - `mmc3_test` / `mmc3_test_2` 已完成关键收敛：
    - PRG-RAM 默认可读写（`$6000` 状态协议可用）；
    - 背景抓取改为 8-dot 分拍（1/3/5/7）以修正 MMC3 scanline IRQ 时序；
    - 套件判定改为“基础项必过 + 变体项二选一通过”（`MMC3` vs `MMC6/MMC3_alt`）。

## 3. 关键差距（按优先级）

### P0（优先处理）

1. `$4015` IRQ 清除语义不一致
- 已对齐（2026-02-19）：NESium 改为读 `$4015` 仅清 frame IRQ；DMC IRQ 由写 `$4015` 清除。

2. Frame Counter 模型未对齐
- Rust 实现与 Mesen2 时点模型存在结构差异：
  - Rust: `crates/nesium-core/src/apu/frame_counter.rs:74`, `crates/nesium-core/src/apu/frame_counter.rs:79`
  - Mesen2: `Mesen2/Core/NES/APU/ApuFrameCounter.h:19`, `Mesen2/Core/NES/APU/ApuFrameCounter.h:24`, `Mesen2/Core/NES/APU/ApuFrameCounter.h:104`, `Mesen2/Core/NES/APU/ApuFrameCounter.h:115`
- Rust 缺少 Mesen2 `_blockFrameCounterTick` 路径：
  - `Mesen2/Core/NES/APU/ApuFrameCounter.h:32`
  - `Mesen2/Core/NES/APU/ApuFrameCounter.h:121`
  - `Mesen2/Core/NES/APU/ApuFrameCounter.h:166`
- `$4017` 写入延迟奇偶处理也不一致：
  - Rust: `crates/nesium-core/src/apu/frame_counter.rs:121`, `crates/nesium-core/src/apu/frame_counter.rs:123`
  - Mesen2: `Mesen2/Core/NES/APU/ApuFrameCounter.h:198`, `Mesen2/Core/NES/APU/ApuFrameCounter.h:203`
- 已修正的关键点：
  - `$4017` 不再无条件清 frame IRQ（仅 `bit6=1` 清除），与 Mesen 行为一致。
- 里程碑（已验证）：已将 Rust `frame_counter` 重构为 Mesen 风格 6-step 时点，并修正：
  - 4-step 末 3-cycle IRQ 窗口
  - `$4017` 写延迟奇偶（odd=4 / even=3）
  - `_blockFrameCounterTick` 抑制行为
  - 在 `6-irq_flag_timing` 上消除首个分歧（`$4015` 首次读值对齐）。

3. Length Counter reload 时序不一致
- 已对齐（2026-02-19）：
  - NESium 已实现 Mesen2 风格的延迟 `reload` 与 pending `halt` 提交顺序。
  - 关键文件：
    - `crates/nesium-core/src/apu/length_counter.rs`
    - `crates/nesium-core/src/apu.rs`
    - `crates/nesium-core/src/apu/pulse.rs`
    - `crates/nesium-core/src/apu/triangle.rs`
    - `crates/nesium-core/src/apu/noise.rs`
  - 对应验证：
    - `crates/nesium-core/tests/rom_suites.rs` 中 `blargg_apu_2005_07_30_suite` 已默认启用并通过。

### P1（高优先）

1. DMC DMA 边界行为不完整
- 已完成的对齐：
  - enable/disable 延迟（2/3 cycle）；
  - disable 生效时 DMA abort；
  - DMA 完成时更新 `bytesRemaining/currentAddr` 并处理 loop/IRQ；
  - output unit 的 bit counter 回卷行为。
  - DMA GET/PUT 周期判定与 `$4015` 写奇偶延迟判定修正。
  - `dmc_dma_during_read4` 五个子 ROM 在关键总线事件上已与 Mesen 对齐（见第 4 节）。
- 仍待验证/处理：
  - dmc_tests 家族（`buffer_retained/latency/status/status_irq`）已改为 Mesen2 2KB RAM 快照（frame=1800）基线判定，避免 `$6000`/串口协议不适配。
  - one-byte DMC (`sample_length==1`) 的 glitch 覆盖仍不完整；试验性分支未带来正向收敛，已回退，后续需按 Mesen 更精确复刻。

2. 频率/周期细节差异
- DMC NTSC rate 表第 13 档已修正：`84`（与 Mesen2 对齐）。
- Noise timer 可能存在 `-1` 偏差：
  - Rust: `crates/nesium-core/src/apu/noise.rs:42`, `crates/nesium-core/src/apu/noise.rs:60`
  - Mesen2: `Mesen2/Core/NES/APU/NoiseChannel.h:91`, `Mesen2/Core/NES/APU/NoiseChannel.h:118`

3. Triangle 行为差异
- `$400B` 写入后序列位置处理不同：
  - Rust: `crates/nesium-core/src/apu/triangle.rs:37`
  - Mesen2: `Mesen2/Core/NES/APU/TriangleChannel.h:90`, `Mesen2/Core/NES/APU/TriangleChannel.h:97`
- 门控后输出保持逻辑不同：
  - Rust: `crates/nesium-core/src/apu/triangle.rs:74`, `crates/nesium-core/src/apu/triangle.rs:77`
  - Mesen2: `Mesen2/Core/NES/APU/TriangleChannel.h:139`, `Mesen2/Core/NES/APU/TriangleChannel.h:141`

### P2（覆盖面）

1. PAL/Dendy 表与行为覆盖不完整
- Rust TODO：
  - `crates/nesium-core/src/apu.rs:9`
  - `crates/nesium-core/src/apu/tables.rs:3`
- Mesen2 已有 PAL 相关表：
  - `Mesen2/Core/NES/APU/ApuFrameCounter.h:21`
  - `Mesen2/Core/NES/APU/NoiseChannel.h:16`
  - `Mesen2/Core/NES/APU/DeltaModulationChannel.h:13`

2. 扩展音频覆盖差异
- Rust 当前只见 mapper19（N163）路径：
  - `crates/nesium-core/src/cartridge/mapper/mapper19.rs:524`
- Mesen2 覆盖 FDS/MMC5/N163/S5B/VRC6/VRC7：
  - `Mesen2/Core/NES/APU/ExpansionAudio/FdsAudio.h:12`
  - `Mesen2/Core/NES/APU/ExpansionAudio/Mmc5Audio.h:45`
  - `Mesen2/Core/NES/APU/ExpansionAudio/Namco163Audio.h:8`
  - `Mesen2/Core/NES/APU/ExpansionAudio/Sunsoft5bAudio.h:8`
  - `Mesen2/Core/NES/APU/ExpansionAudio/Vrc6Audio.h:9`
  - `Mesen2/Core/NES/APU/ExpansionAudio/Vrc7Audio.h:10`

## 4. 测试现状（关键结果）

- `cargo test -p nesium-core`（当前分支）：
  - `70 passed / 0 failed / 3 ignored`（单测）。
  - `39 passed / 0 failed / 20 ignored`（ROM suites）。
- APU 相关关键套件（默认回归）：
  - `apu_mixer_suite` 通过。
  - `apu_reset_suite` 通过。
  - `apu_test_suite` 通过。
  - `blargg_apu_2005_07_30_suite` 通过。
  - `dmc_tests_suite` 通过（Mesen2 RAM baseline）。
  - `dmc_dma_during_read4_suite` 通过（Mesen2 串口基线）。
  - `sprdma_and_dmc_dma_suite` 通过。
- MMC3 相关关键套件（默认回归）：
  - `mmc3_irq_tests_suite` 通过（`rev A/rev B` 采用“至少一个通过”判定）。
  - `mmc3_test_suite` 通过（`MMC3/MMC6-style` 采用“至少一个通过”判定）。
  - `mmc3_test_2_suite` 通过（`MMC3/MMC3_alt` 采用“至少一个通过”判定）。
- 当前仍保留为 ignore 的 APU 关键项：
  - `pal_apu_tests_suite`（PAL 时序/表尚未完整对齐）。
- `apu_reset_suite` 虽通过，但存在测试框架特判兜底：
  - `crates/nesium-core/tests/mod.rs:297`
  - `crates/nesium-core/tests/mod.rs:309`

## 5. 构建与环境关键结论

1. `UI/InteropDLL Lua` 重复符号根因（已定位）
- 工程本身已链接本地 Lua 项目：
  - `Mesen2/InteropDLL/InteropDLL.vcxproj:220`
- 本机启用了全局 `vcpkg integrate install`，自动注入 `*.lib`：
  - `C:/Users/dream/AppData/Local/vcpkg/vcpkg.user.props:3`
  - `C:/Users/dream/AppData/Local/vcpkg/vcpkg.user.targets:3`
  - `F:/vcpkg/scripts/buildsystems/msbuild/vcpkg.targets:116`
- 本机 `F:/vcpkg/installed/x64-windows/lib/lua.lib` 与本地 `Lua.lib` 冲突导致 `LNK2005/LNK1169`。

2. 已验证可行构建命令（关键）

```powershell
msbuild Mesen2\Mesen.sln /restore /m /t:UI /p:Configuration=Release /p:Platform=x64 /p:VcpkgAutoLink=false
```

- 构建产物已验证：
  - `Mesen2/ui_build_restore_no_vcpkg_autolink.log:10`（`MesenCore.dll`）
  - `Mesen2/ui_build_restore_no_vcpkg_autolink.log:24`（`Mesen.dll`）

3. 已解决问题（简要）
- `.git/modules/apps/nesium_flutter/assets/shaders/config` 损坏导致 `git status` 失败，已修复并恢复可用。
- Mesen `--testRunner` 在当前环境下为异步进程模型：日志采集需使用绝对输出路径，并配合 `-timeout=<sec>` 保证自动退出。
- `tools/apu_compare/mesen_dump_status.lua` 已修正为 `emu.memType.nesMemory`（此前 memType 使用错误会导致状态脚本不可用）。
- `run_rom_status` 的 RAM 关键字兜底已移除，仅保留 `$6000` 协议 + serial 兜底，避免 `mmc3_test*` 上的假阳性失败。

## 6. 下一关键节点

1. Mapper4 revision 自动识别已落地（NES2 submapper + env override）；下一步是补充外部数据库映射，降低 iNES ROM 上“二选一”策略占比。
2. 保持 `read_mem` 双端对齐链路，继续覆盖 DMC 相关 ROM（优先 one-byte glitch 与 `dpcmletterbox`）。
3. 持续把“协议不匹配”的 ROM 从 `run_rom_status` 迁移到可自动判定基线，减少误报。


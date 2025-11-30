# NES `Mapper` Abstraction Review

## 1. Goals

You want an accurate NES emulator in Rust, with a `Mapper` abstraction that:

- Comfortably covers most **mainstream mappers**: NROM, UxROM, CNROM, AxROM, MMC1, MMC3, common UNROM variants, etc.
- Can be extended to support more complex boards (VRC family, MMC5, various pirate / homebrew mappers).
- Stays reasonably simple to implement and debug, while still being precise enough to pass demanding NES test ROMs.

---

## 2. Current `Mapper` Trait

```rust
pub trait Mapper: Debug + Send + DynClone + Any + 'static {
    fn cpu_read(&self, addr: u16) -> Option<u8>;
    fn cpu_write(&mut self, addr: u16, data: u8, cpu_cycle: u64);

    fn ppu_read(&self, addr: u16) -> u8;
    fn ppu_write(&mut self, addr: u16, data: u8);

    fn irq_pending(&self) -> bool { false }
    fn clear_irq(&mut self) {}

    fn prg_rom(&self) -> Option<&[u8]> { None }
    fn prg_ram(&self) -> Option<&[u8]> { None }
    fn prg_ram_mut(&mut self) -> Option<&mut [u8]> { None }
    fn chr_rom(&self) -> Option<&[u8]> { None }
    fn chr_ram(&self) -> Option<&[u8]> { None }
    fn chr_ram_mut(&mut self) -> Option<&mut [u8]> { None }

    fn mirroring(&self) -> Mirroring;

    fn mapper_id(&self) -> u16;
    fn name(&self) -> Cow<'static, str> {
        Cow::Owned(format!("Mapper {}", self.mapper_id()))
    }
}
```

---

## 3. What the Current Design Already Handles Well

### 3.1 CPU Bus Side

- `cpu_read(&self, addr) -> Option<u8>` cleanly expresses:
  - `Some(byte)` → mapper drives the data bus.
  - `None` → open bus (CPU should reuse the last bus value).
- `cpu_write(&mut self, addr, data, cpu_cycle)` gives mappers the current CPU cycle for timing quirks.
  - You already use this for MMC1's "ignore consecutive serial writes" behavior.

This is enough for:

- Basic ROM/RAM banking (NROM, UxROM, CNROM, AxROM, etc.).
- MMC1 behavior including write timing quirks.
- Timers / IRQs that care about when writes happen.

### 3.2 PPU CHR Side

- `ppu_read/ppu_write` allow mapper-controlled CHR for `$0000-$1FFF`:
  - Suitable for CHR ROM/RAM banking (1K/2K/4K/8K windows).
  - Works for MMC1, MMC3, MMC2/4 latches, and many simple mappers.

### 3.3 IRQ Support

- `irq_pending` + `clear_irq` is enough to:
  - Implement scanline / counter IRQs in MMC3, FME-7, VRC variants, etc.
  - Have the CPU core poll and assert the IRQ line correctly.

### 3.4 Introspection and Save/Debug

- `prg_rom/prg_ram/chr_rom/chr_ram` hooks let you:
  - Implement UI-level memory viewers.
  - Implement basic battery save by dumping PRG/CHR RAM.

### 3.5 Mirroring / Nametable Wiring (Basic Case)

- `mirroring()` exposes:
  - Horizontal / vertical / single-screen / etc.
  - Compatible with the usual 2 KB CIRAM setup in the PPU.

This is sufficient for the majority of licensed games using simple mirroring schemes.

**Conclusion**: The current trait is already good enough to implement **most "normal" NES cartridges**, especially those that only touch PRG/CHR banking + simple nametable mirroring.

---

## 4. Key Limitations / Missing Capabilities

These are the main areas where the current interface will start to struggle if you aim for very high accuracy or more exotic boards.

### 4.1 Nametable / VRAM Mapping Is Too Limited

Right now:

- The mapper only controls CHR (`$0000-$1FFF`) via `ppu_read/ppu_write`.
- Nametable area (`$2000-$3EFF`) is assumed to be:
  - Handled by the PPU itself, plus
  - A global `Mirroring` enum.

This cannot express:

- **MMC5 ExRAM used as nametable** or extended attribute tables.
- Boards with **extra nametable RAM or ROM** on the cartridge.
- 4-screen VRAM and other non-standard nametable wiring beyond simple mirroring.

For those, the mapper must be able to:

- Decide where each nametable byte actually comes from (CIRAM vs mapper RAM vs ROM).
- Potentially alter values returned during attribute or pattern fetches.

### 4.2 No General PPU/CPU Timing Hooks

You have timing info on CPU writes, but not:

- A hook that runs **every CPU cycle** for mappers with cycle-based IRQ timers.
- A general hook for **every VRAM access** for PPU-side timing (e.g., MMC3 scanline IRQ based on A12 edges).

This makes high-accuracy implementations of:

- MMC3 IRQ (PPU A12 rising edges + CPU M2 gating).
- Some VRC scanline timers.
- MMC5 advanced graphics modes (vertical split / extended attributes).

much harder or forces hacks in the CPU/PPU core instead of keeping logic localized in the mapper.

### 4.3 No Way to Express PPU Open-Bus

CPU side:

- `Option<u8>` makes it clear when the bus is floating.

PPU side:

- `fn ppu_read(&self, addr: u16) -> u8` always returns *some* value, even when on real hardware:
  - The read would actually be open-bus.
  - The result might depend on previous VRAM / palette / nametable accesses.

This only matters for very strict accuracy or specific test ROMs, but it's a mismatch in API expressiveness.

### 4.4 RAM Types Are Not Distinguished

Many real boards have more than one RAM category:

- Battery-backed save RAM.
- Non-battery work RAM.
- Mapper-private RAM (sometimes not CPU-visible).
- Possibly battery-backed CHR RAM.

The current:

```rust
fn prg_ram(&self) -> Option<&[u8]>;
fn prg_ram_mut(&mut self) -> Option<&mut [u8]>;
```

combines all CPU-side RAM into one bucket. This is fine for running games, but:

- Complicates battery save (what should be persisted?).
- Limits tooling (debugger cannot easily display "this is PRG RAM vs mapper RAM").

### 4.5 No Built-In Path for Expansion Audio

Expansion audio (VRC6/VRC7/Namco 163/Sunsoft 5B/MMC5/FDS, etc.) requires:

- Mapper responding to CPU writes to audio registers.
- A per-cycle clock to update audio synth state.
- A way for the APU/mixer to get the expansion audio sample.

The current trait doesn’t mention audio at all, which is okay, but:

- You will need a separate trait or mechanism to plug expansion audio into the APU.

---

## 5. Suggested Trait Extensions (Design Ideas)

All of these can be added as **optional methods with default no-op implementations** so that existing mappers keep compiling.

### 5.1 VRAM Access Hook for PPU (MMC3/MMC5, etc.)

Add something like:

```rust
pub trait Mapper {
    // existing methods...

    /// Called on every PPU VRAM access (0x0000-0x3FFF).
    /// cpu_cycle is the current CPU cycle for M2-based gating logic.
    fn ppu_vram_access(&mut self, _addr: u16, _is_read: bool, _cpu_cycle: u64) {}
}
```

PPU integration (pseudo-code):

```rust
fn vram_read(&mut self, addr: u16, is_render: bool) -> u8 {
    let cpu_cycle = self.cpu_cycle;
    self.mapper.ppu_vram_access(addr, true, cpu_cycle);

    if addr < 0x2000 {
        self.mapper.ppu_read(addr)
    } else {
        // nametable / palette handling
    }
}
```

Use cases:

- **MMC3**: detect A12 rising edges in `ppu_vram_access` and update the IRQ counter with correct CPU/PPU timing.
- **MMC5**: track VRAM fetch patterns to support vertical split, extended attributes, etc.

### 5.2 Nametable Mapping API (CIRAM vs Mapper VRAM/ROM)

Introduce a `NametableTarget` enum:

```rust
pub enum NametableTarget {
    /// Use PPU CIRAM (internal 2 KB VRAM). `u16` is CIRAM offset.
    Ciram(u16),
    /// Use mapper-controlled VRAM/ROM. `u16` is mapper-local offset.
    MapperVram(u16),
    /// No device drives the bus (open bus).
    None,
}
```

Extend the trait:

```rust
pub trait Mapper {
    // existing methods...

    /// Map PPU $2000-$2FFF address to an underlying nametable source.
    /// Default implementation uses `mirroring()` and standard CIRAM mapping.
    fn map_nametable(&self, addr: u16) -> NametableTarget {
        let base = addr & 0x0FFF;
        let offset = match self.mirroring() {
            Mirroring::Horizontal => {
                let nt = (base >> 10) & 3;
                let within = base & 0x03FF;
                match nt {
                    0 | 1 => within,
                    _ => 0x0400 | within,
                }
            }
            Mirroring::Vertical => {
                let nt = (base >> 10) & 3;
                let within = base & 0x03FF;
                match nt {
                    0 | 2 => within,
                    _ => 0x0400 | within,
                }
            }
            Mirroring::SingleScreenLower => base & 0x03FF,
            Mirroring::SingleScreenUpper => 0x0400 | (base & 0x03FF),
            // Mirroring::FourScreen / MapperControlled could be added later.
        };
        NametableTarget::Ciram(offset)
    }

    /// Called when `map_nametable` returns `MapperVram`.
    fn mapper_nametable_read(&self, _offset: u16) -> u8 { 0 }

    fn mapper_nametable_write(&mut self, _offset: u16, _value: u8) {}
}
```

PPU nametable path:

```rust
fn nametable_read(&mut self, addr: u16) -> u8 {
    match self.mapper.map_nametable(addr) {
        NametableTarget::Ciram(off) => self.ciram[off as usize],
        NametableTarget::MapperVram(off) => self.mapper.mapper_nametable_read(off),
        NametableTarget::None => self.open_bus_value,
    }
}
```

This allows:

- Ordinary mappers to continue using `mirroring()` + CIRAM.
- MMC5 / 4-screen / ExRAM boards to override `map_nametable` and fully control nametable behavior.

### 5.3 Optional Per-CPU-Cycle Hook

If you need true per-cycle timers inside the mapper, add:

```rust
pub trait Mapper {
    // existing methods...

    /// Called once per CPU cycle.
    fn cpu_clock(&mut self, _cpu_cycle: u64) {}
}
```

Then in your CPU loop:

```rust
for _ in 0..cycles_for_this_instruction {
    cpu_cycle += 1;
    mapper.cpu_clock(cpu_cycle);
    // tick APU, PPU, etc.
}
```

Not every mapper will need this, but it’s useful for:

- Certain VRC IRQ modes.
- Mappers with hardware timers.
- Potentially expansion audio (if you choose to couple it here).

### 5.4 Optional PPU Open-Bus Expressiveness

If you decide to model PPU open-bus more accurately later, you can:

- Either change `ppu_read` to return `Option<u8>`.
- Or introduce a small enum (e.g., `PpuReadResult { Data(u8), OpenBus }`).

For now, this can be postponed if test ROMs don’t require that level of detail.

### 5.5 More Granular RAM Introspection (Save vs Work vs Mapper)

Longer-term refinement:

```rust
pub trait Mapper {
    // PRG save RAM (battery-backed)
    fn prg_save_ram(&self) -> Option<&[u8]> { None }
    fn prg_save_ram_mut(&mut self) -> Option<&mut [u8]> { None }

    // PRG work RAM (not battery-backed)
    fn prg_work_ram(&self) -> Option<&[u8]> { None }
    fn prg_work_ram_mut(&mut self) -> Option<&mut [u8]> { None }

    // Mapper-private RAM (could be used for ExRAM, etc.)
    fn mapper_ram(&self) -> Option<&[u8]> { None }
    fn mapper_ram_mut(&mut self) -> Option<&mut [u8]> { None }

    // Optional: CHR battery-backed RAM
    fn chr_battery_ram(&self) -> Option<&[u8]> { None }
    fn chr_battery_ram_mut(&mut self) -> Option<&mut [u8]> { None }
}
```

This lets:

- The save system persist only the right subsets.
- Debug tools label different RAM regions more accurately.

You can keep your existing `prg_ram/chr_ram` methods as compatibility layers or slowly migrate to the more granular version.

### 5.6 Separate Expansion Audio Trait

To keep `Mapper` focused on memory/IRQ, expansion audio can be handled via a separate trait:

```rust
pub trait ExpansionAudio {
    /// Advance the expansion audio state by one CPU cycle.
    fn clock_audio(&mut self);

    /// Produce the current expansion audio sample (e.g., linear 0.0..1.0).
    fn sample(&mut self) -> f32;
}
```

- Some mapper types implement both `Mapper + ExpansionAudio`.
- The APU/mixer holds a handle to `dyn ExpansionAudio` where applicable and:
  - Calls `clock_audio()` per CPU cycle.
  - Calls `sample()` when mixing audio.

---

## 6. Notes on the Current `Mapper1` Implementation

Your `Mapper1` implementation:

- Uses `cpu_cycle` in `cpu_write` to correctly emulate MMC1’s serial write timing quirks.
- Respects mirroring by exposing it via `mirroring()`.
- Nicely exposes PRG/CHR ROM/RAM through the introspection hooks.
- Handles PRG-RAM enable/disable and returns `None` for reads when disabled, which fits nicely with the bus-level open-bus model.

Potential future tweaks:

- Once you add VRAM hooks / nametable mapping:
  - MMC1 will mostly ignore them; only more advanced SxROM variants might use extended behaviors.
- If you extend RAM introspection (save/work/mapper RAM), you can simply map:
  - `prg_save_ram` → your existing `prg_ram` for plain MMC1 boards.
  - Leave `mapper_ram` and `chr_battery_ram` as `None`.

---

## 7. TODO Checklist

### 7.1 Core Trait & Architecture

- [x] Add `fn ppu_vram_access(&mut self, addr: u16, is_read: bool, cpu_cycle: u64)` to the `Mapper` trait with a default no-op implementation.
- [x] Update the PPU so that **every VRAM access** (pattern fetch + CPU `$2007` accesses) calls `mapper.ppu_vram_access`.
- [x] Design and implement the `NametableTarget` enum and the trait methods `map_nametable`, `mapper_nametable_read`, and `mapper_nametable_write`.
- [x] Change the PPU nametable read/write path to use `map_nametable`, selecting between CIRAM, mapper VRAM, or open-bus.
- [x] (Optional) Extend the `Mirroring` enum if needed (e.g., `FourScreen`, `MapperControlled`) and adapt the default `map_nametable` implementation accordingly.
- [x] Decide whether to upgrade `ppu_read` to an `Option<u8>` or similar to explicitly represent PPU open-bus conditions.
- [x] Evaluate whether you need a per-CPU-cycle hook `fn cpu_clock(&mut self, cpu_cycle: u64)` and, if so, integrate it into the CPU core loop.
- [x] Design a separate `ExpansionAudio` trait and integrate it into the audio pipeline (APU/mixer).
- [x] (Optional) Refine RAM introspection by splitting `prg_ram`/`chr_ram` into save/work/mapper RAM types, and adjust battery save & debugger code.

### 7.2 Specific Mapper Implementations (Future Work)

- [ ] Use `ppu_vram_access` + `cpu_cycle` to implement **MMC3 IRQ** with proper A12 edge detection and M2-based timing.
- [ ] Use `map_nametable` + `mapper_nametable_*` to implement **MMC5 ExRAM** as nametable / extended attributes / fill mode.
- [ ] For mappers with bus conflicts (CNROM, certain UNROM boards), implement `data &= prg_rom_byte` behavior in `cpu_write`.
- [ ] Extend MMC1/SxROM implementations with CHR/PRG high-bit banking and board-specific quirks using NES 2.0 submapper & board DB info.
- [ ] Implement `ExpansionAudio` for mappers that provide extra audio (VRC6/VRC7/Sunsoft 5B/MMC5/FDS) and connect them to the audio mixer.

---

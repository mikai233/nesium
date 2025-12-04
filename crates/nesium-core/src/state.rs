//! Abstract save/load interfaces for emulator components.
//!
//! This module only defines the trait surface; concrete subsystems (CPU/PPU/APU,
//! mappers, etc.) can implement it when ready. Callers can choose any backing
//! serialization format (bincode/serde/json) externally.

pub mod cpu;
pub mod ppu;
use std::convert::Infallible;

use crate::{cpu::Cpu, ppu::Ppu};

/// Common metadata attached to full/delta snapshots to aid compatibility checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotMeta {
    /// Version of the snapshot payload (per component).
    pub format_version: u32,
    /// Identifier of the baseline full snapshot this delta depends on.
    pub baseline_id: u64,
    /// Global tick/frame counter when this snapshot was captured.
    pub tick: u64,
    /// Optional ROM hash (e.g., SHA-256) for compatibility checks.
    pub rom_hash: Option<[u8; 32]>,
    /// Optional mapper id/submapper for quick cartridge validation.
    pub mapper: Option<(u16, u8)>,
}

impl Default for SnapshotMeta {
    fn default() -> Self {
        Self {
            format_version: 1,
            baseline_id: 0,
            tick: 0,
            rom_hash: None,
            mapper: None,
        }
    }
}

/// Simple wrapper bundling snapshot metadata with payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot<T, M = SnapshotMeta> {
    pub meta: M,
    pub data: T,
}

/// Minimal save/load contract with full and incremental variants.
pub trait SaveState {
    type Full;
    type Delta;
    type Error;
    type Meta: Clone;

    /// Optional format/version tag. Implementers can bump this when changing
    /// the snapshot layout to let callers reject incompatible data.
    const FORMAT_VERSION: u32 = 1;

    /// Capture a full snapshot of the component state.
    ///
    /// Callers provide metadata (e.g., tick, baseline id, rom hash); the
    /// implementation may adjust `meta.format_version` as needed.
    fn save_full(&self, meta: Self::Meta) -> Result<Snapshot<Self::Full, Self::Meta>, Self::Error>;

    /// Capture an incremental snapshot relative to a previously captured full
    /// snapshot. Implementations decide how to encode the delta.
    ///
    /// Default: fall back to a full snapshot when `Delta` can be constructed
    /// from `Full`.
    fn save_delta(
        &self,
        baseline: &Snapshot<Self::Full, Self::Meta>,
    ) -> Result<Snapshot<Self::Delta, Self::Meta>, Self::Error>
    where
        Self::Delta: From<Self::Full>,
    {
        let meta = baseline.meta.clone();
        self.save_full(meta).map(|snap| Snapshot {
            meta: snap.meta,
            data: Self::Delta::from(snap.data),
        })
    }

    /// Restore the component from a full snapshot.
    fn load_full(&mut self, snapshot: &Snapshot<Self::Full, Self::Meta>)
    -> Result<(), Self::Error>;

    /// Apply an incremental snapshot.
    ///
    /// Default: convert the delta back into a full snapshot when possible.
    fn load_delta(&mut self, delta: &Snapshot<Self::Delta, Self::Meta>) -> Result<(), Self::Error>
    where
        Self::Delta: Clone + Into<Self::Full>,
    {
        let full: Self::Full = delta.data.clone().into();
        self.load_full(&Snapshot {
            meta: delta.meta.clone(),
            data: full,
        })
    }
}

/// Optional extension that allows implementers to expose borrowed views instead
/// of owned copies. This is useful for large buffers (RAM/VRAM) where a
/// zero-copy write-out is preferable.
pub trait SaveStateBorrowed: SaveState {
    type BorrowedFull<'a>: 'a
    where
        Self: 'a;
    type BorrowedDelta<'a>: 'a
    where
        Self: 'a;

    /// Borrow a full snapshot view. Callers can choose to serialize this view
    /// directly without cloning.
    fn borrow_full<'a>(
        &'a self,
        meta: Self::Meta,
    ) -> Result<Snapshot<Self::BorrowedFull<'a>, Self::Meta>, Self::Error>;

    /// Borrow a delta view relative to a baseline.
    fn borrow_delta<'a>(
        &'a self,
        baseline: &Snapshot<Self::Full, Self::Meta>,
    ) -> Result<Snapshot<Self::BorrowedDelta<'a>, Self::Meta>, Self::Error>
    where
        Self::Delta: From<Self::Full>;
}

/// Fallback borrowed implementation: uses owned copies when a true borrowed
/// view is not provided.
impl<T> SaveStateBorrowed for T
where
    T: SaveState,
    T::Full: Clone,
    T::Delta: Clone,
{
    type BorrowedFull<'a>
        = T::Full
    where
        T: 'a,
        T::Full: 'a;
    type BorrowedDelta<'a>
        = T::Delta
    where
        T: 'a,
        T::Delta: 'a;

    fn borrow_full<'a>(
        &'a self,
        meta: Self::Meta,
    ) -> Result<Snapshot<Self::BorrowedFull<'a>, Self::Meta>, Self::Error> {
        self.save_full(meta).map(|snap| Snapshot {
            meta: snap.meta,
            data: snap.data,
        })
    }

    fn borrow_delta<'a>(
        &'a self,
        baseline: &Snapshot<Self::Full, Self::Meta>,
    ) -> Result<Snapshot<Self::BorrowedDelta<'a>, Self::Meta>, Self::Error>
    where
        Self::Delta: From<Self::Full>,
    {
        self.save_delta(baseline).map(|snap| Snapshot {
            meta: snap.meta,
            data: snap.data,
        })
    }
}

/// Aggregates component save states into a single NES snapshot that callers can
/// serialize with any format (e.g., serde/bincode) and later restore.
pub trait StateComposer {
    type FullState;
    type DeltaState;
    type Error;

    fn capture_full(&mut self, meta: SnapshotMeta) -> Result<Self::FullState, Self::Error>;
    fn capture_delta(
        &mut self,
        baseline: &Self::FullState,
    ) -> Result<Self::DeltaState, Self::Error>;
    fn apply_full(&mut self, state: &Self::FullState) -> Result<(), Self::Error>;
    fn apply_delta(&mut self, delta: &Self::DeltaState) -> Result<(), Self::Error>;
}

/// Simple aggregate of CPU/PPU snapshots (demo; extend with APU/mapper/RAM later).
#[derive(Debug, Clone)]
pub struct DefaultNesFullState<M = SnapshotMeta> {
    pub cpu: Snapshot<Cpu, M>,
    pub ppu: Snapshot<Ppu, M>,
}

#[derive(Debug, Clone)]
pub struct DefaultNesDeltaState<M = SnapshotMeta> {
    pub cpu: Snapshot<Cpu, M>,
    pub ppu: Snapshot<Ppu, M>,
}

/// Default composer wiring CPU+PPU `SaveState` into a single unit.
pub struct DefaultNesComposer<'a> {
    pub cpu: &'a mut Cpu,
    pub ppu: &'a mut Ppu,
}

#[derive(Debug)]
pub enum DefaultComposeError {
    Cpu(Infallible),
    Ppu(Infallible),
}

impl From<Infallible> for DefaultComposeError {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

impl<'a> StateComposer for DefaultNesComposer<'a> {
    type FullState = DefaultNesFullState;
    type DeltaState = DefaultNesDeltaState;
    type Error = DefaultComposeError;

    fn capture_full(&mut self, meta: SnapshotMeta) -> Result<Self::FullState, Self::Error> {
        let cpu = self
            .cpu
            .save_full(meta.clone())
            .map_err(DefaultComposeError::Cpu)?;
        let ppu = self.ppu.save_full(meta).map_err(DefaultComposeError::Ppu)?;
        Ok(DefaultNesFullState { cpu, ppu })
    }

    fn capture_delta(
        &mut self,
        baseline: &Self::FullState,
    ) -> Result<Self::DeltaState, Self::Error> {
        let cpu = self
            .cpu
            .save_delta(&baseline.cpu)
            .map_err(DefaultComposeError::Cpu)?;
        let ppu = self
            .ppu
            .save_delta(&baseline.ppu)
            .map_err(DefaultComposeError::Ppu)?;
        Ok(DefaultNesDeltaState { cpu, ppu })
    }

    fn apply_full(&mut self, state: &Self::FullState) -> Result<(), Self::Error> {
        self.cpu
            .load_full(&state.cpu)
            .map_err(DefaultComposeError::Cpu)?;
        self.ppu
            .load_full(&state.ppu)
            .map_err(DefaultComposeError::Ppu)?;
        Ok(())
    }

    fn apply_delta(&mut self, delta: &Self::DeltaState) -> Result<(), Self::Error> {
        self.cpu
            .load_delta(&delta.cpu)
            .map_err(DefaultComposeError::Cpu)?;
        self.ppu
            .load_delta(&delta.ppu)
            .map_err(DefaultComposeError::Ppu)?;
        Ok(())
    }
}

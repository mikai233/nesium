//! Abstract save/load interfaces for emulator components.
//!
//! This module defines a minimal trait surface for capturing and restoring
//! component state as a *full* snapshot.
//!
//! Incremental/typed deltas are intentionally *not* part of this API.
//! For rewind and rollback use-cases, prefer storing serialized snapshot bytes
//! using external compression/diffing (e.g., XOR + LZ4) at the runtime layer.

pub mod cpu;
pub mod nes;
pub mod ppu;

use std::convert::Infallible;

use crate::{cpu::Cpu, ppu::Ppu};

/// Common metadata attached to snapshots to aid compatibility checks.
#[cfg_attr(
    feature = "savestate-serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotMeta {
    /// Version of the snapshot payload (per component).
    pub format_version: u32,
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
            tick: 0,
            rom_hash: None,
            mapper: None,
        }
    }
}

/// Simple wrapper bundling snapshot metadata with payload.
#[cfg_attr(
    feature = "savestate-serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot<T, M = SnapshotMeta> {
    pub meta: M,
    pub data: T,
}

/// Minimal save/load contract using full snapshots.
///
/// The snapshot payload type is left to implementers. Callers are expected to
/// serialize snapshots externally (e.g., postcard/bincode/serde) and can apply
/// compression or diffing at higher layers.
pub trait SaveState {
    type State;
    type Error;
    type Meta: Clone;

    /// Optional format/version tag. Implementers can bump this when changing
    /// the snapshot layout to let callers reject incompatible data.
    const FORMAT_VERSION: u32 = 1;

    /// Capture a full snapshot of the component state.
    ///
    /// Callers provide metadata (e.g., tick, rom hash); the implementation may
    /// adjust `meta.format_version` as needed.
    fn save(&self, meta: Self::Meta) -> Result<Snapshot<Self::State, Self::Meta>, Self::Error>;

    /// Restore the component from a full snapshot.
    fn load(&mut self, snapshot: &Snapshot<Self::State, Self::Meta>) -> Result<(), Self::Error>;
}

/// Optional extension that allows implementers to expose borrowed views instead
/// of owned copies. This is useful for large buffers (RAM/VRAM) where a
/// zero-copy write-out is preferable.
pub trait SaveStateBorrowed: SaveState {
    type BorrowedState<'a>: 'a
    where
        Self: 'a;

    /// Borrow a full snapshot view. Callers can choose to serialize this view
    /// directly without cloning.
    fn borrow<'a>(
        &'a self,
        meta: Self::Meta,
    ) -> Result<Snapshot<Self::BorrowedState<'a>, Self::Meta>, Self::Error>;
}

/// Fallback borrowed implementation: uses owned copies when a true borrowed
/// view is not provided.
impl<T> SaveStateBorrowed for T
where
    T: SaveState,
    T::State: Clone,
{
    type BorrowedState<'a>
        = T::State
    where
        T: 'a,
        T::State: 'a;

    fn borrow<'a>(
        &'a self,
        meta: Self::Meta,
    ) -> Result<Snapshot<Self::BorrowedState<'a>, Self::Meta>, Self::Error> {
        self.save(meta).map(|snap| Snapshot {
            meta: snap.meta,
            data: snap.data,
        })
    }
}

/// Aggregates component save states into a single NES snapshot that callers can
/// serialize with any format and later restore.
pub trait StateComposer {
    type FullState;
    type Error;

    fn capture(&mut self, meta: SnapshotMeta) -> Result<Self::FullState, Self::Error>;
    fn apply(&mut self, state: &Self::FullState) -> Result<(), Self::Error>;
}

/// Simple aggregate of CPU/PPU snapshots (demo; extend with APU/mapper/RAM later).
#[derive(Debug, Clone)]
pub struct DefaultNesFullState<M = SnapshotMeta> {
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
    type Error = DefaultComposeError;

    fn capture(&mut self, meta: SnapshotMeta) -> Result<Self::FullState, Self::Error> {
        let cpu = self
            .cpu
            .save(meta.clone())
            .map_err(DefaultComposeError::Cpu)?;
        let ppu = self.ppu.save(meta).map_err(DefaultComposeError::Ppu)?;
        Ok(DefaultNesFullState { cpu, ppu })
    }

    fn apply(&mut self, state: &Self::FullState) -> Result<(), Self::Error> {
        self.cpu
            .load(&state.cpu)
            .map_err(DefaultComposeError::Cpu)?;
        self.ppu
            .load(&state.ppu)
            .map_err(DefaultComposeError::Ppu)?;
        Ok(())
    }
}

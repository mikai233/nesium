use std::convert::Infallible;

use crate::cpu::Cpu;
use crate::state::{SaveState, Snapshot};

/// Demo `SaveState` implementation for the CPU.
///
/// This is intentionally minimal and just clones the whole CPU struct.
/// Replace with a smaller snapshot or diff-based approach once a concrete
/// format is chosen.
impl SaveState for Cpu {
    type State = Cpu;
    type Error = Infallible;
    type Meta = crate::state::SnapshotMeta;

    fn save(&self, meta: Self::Meta) -> Result<Snapshot<Self::State, Self::Meta>, Self::Error> {
        Ok(Snapshot { meta, data: *self })
    }

    fn load(&mut self, snapshot: &Snapshot<Self::State, Self::Meta>) -> Result<(), Self::Error> {
        *self = snapshot.data;
        Ok(())
    }
}

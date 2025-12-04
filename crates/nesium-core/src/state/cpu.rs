use std::convert::Infallible;

use crate::cpu::Cpu;
use crate::state::{SaveState, Snapshot};

/// Demo `SaveState` implementation for the CPU.
///
/// This is intentionally minimal and just clones the whole CPU struct.
/// Replace with a smaller snapshot or diff-based approach once a concrete
/// format is chosen.
impl SaveState for Cpu {
    type Full = Cpu;
    type Delta = Cpu;
    type Error = Infallible;
    type Meta = crate::state::SnapshotMeta;

    fn save_full(&self, meta: Self::Meta) -> Result<Snapshot<Self::Full, Self::Meta>, Self::Error> {
        Ok(Snapshot { meta, data: *self })
    }

    fn load_full(
        &mut self,
        snapshot: &Snapshot<Self::Full, Self::Meta>,
    ) -> Result<(), Self::Error> {
        *self = snapshot.data;
        Ok(())
    }

    // Default delta behaviour (full copy) is sufficient for now.
}

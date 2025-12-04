use std::convert::Infallible;

use crate::ppu::Ppu;
use crate::state::{SaveState, Snapshot};

/// Demo `SaveState` implementation for the PPU.
///
/// This simply clones the full PPU state; swap it out for a slimmer snapshot
/// once a concrete serialization format is picked.
impl SaveState for Ppu {
    type Full = Ppu;
    type Delta = Ppu;
    type Error = Infallible;
    type Meta = crate::state::SnapshotMeta;

    fn save_full(&self, meta: Self::Meta) -> Result<Snapshot<Self::Full, Self::Meta>, Self::Error> {
        Ok(Snapshot {
            meta,
            data: self.clone(),
        })
    }

    fn load_full(
        &mut self,
        snapshot: &Snapshot<Self::Full, Self::Meta>,
    ) -> Result<(), Self::Error> {
        *self = snapshot.data.clone();
        Ok(())
    }

    // Default delta behaviour (full clone) is sufficient for now.
}

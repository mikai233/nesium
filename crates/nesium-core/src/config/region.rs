use core::fmt;

use crate::cartridge::header::TvSystem;

/// Runtime region / timing selection used by the CPU/PPU/APU.
///
/// This is derived from both user configuration and the ROM header's `TvSystem`
/// hint. Unlike `TvSystem`, this never has "unknown" or "dual" – it always
/// resolves to a concrete timing profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Region {
    /// Let the emulator pick a region based on ROM header / database.
    #[default]
    Auto,
    /// North American / general NTSC NES timing.
    Ntsc,
    /// European PAL NES timing.
    Pal,
    /// Dendy-style hybrid timing used by some Famiclones.
    Dendy,
    /// Japanese Famicom NTSC timing (used to split out JP-specific quirks).
    NtscJp,
}

impl Region {
    /// Resolve the effective region from a user-selected region and the ROM
    /// header's TV system hint.
    ///
    /// - If `config_region` is not `Auto`, it wins.
    /// - Otherwise we map `TvSystem` to a concrete region with some sensible
    ///   fallbacks.
    pub fn resolve(config_region: Region, tv: TvSystem) -> Region {
        match config_region {
            Region::Auto => match tv {
                TvSystem::Ntsc => Region::Ntsc,
                TvSystem::Pal => Region::Pal,
                TvSystem::Dual => Region::Ntsc, // pick NTSC as default for dual-region ROMs
                TvSystem::Dendy => Region::Dendy,
                TvSystem::Unknown => Region::Ntsc, // fallback if header is bogus
            },
            other => other,
        }
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Region::Auto => "auto",
            Region::Ntsc => "ntsc",
            Region::Pal => "pal",
            Region::Dendy => "dendy",
            Region::NtscJp => "ntsc-jp",
        };
        f.write_str(s)
    }
}

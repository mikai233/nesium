use crate::cartridge::header::Header;
#[cfg(feature = "cartridge-db")]
use crate::cartridge::header::RomFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CartridgeDbEntry {
    pub rom_body_crc32: u32,
    pub mapper: u16,
    pub submapper: u8,
    pub prg_rom_size: usize,
    pub chr_rom_size: usize,
    pub chr_ram_size: usize,
    pub work_ram_size: usize,
    pub save_ram_size: usize,
    pub has_battery: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CartridgeDbOverride {
    pub mapper: u16,
    pub submapper: u8,
}

#[cfg(feature = "cartridge-db")]
fn compute_rom_body_crc32(prg_rom: &[u8], chr_rom: &[u8]) -> u32 {
    use crc32fast::Hasher;

    let mut hasher = Hasher::new();
    hasher.update(prg_rom);
    hasher.update(chr_rom);
    hasher.finalize()
}

#[cfg(feature = "cartridge-db")]
mod generated {
    use super::CartridgeDbEntry;

    include!(concat!(env!("OUT_DIR"), "/cartridge_db_generated.rs"));
}

#[cfg(feature = "cartridge-db")]
pub(crate) fn lookup_entry(rom_body_crc32: u32) -> Option<&'static CartridgeDbEntry> {
    generated::NES_DB.get(&rom_body_crc32)
}

#[cfg(not(feature = "cartridge-db"))]
pub(crate) fn lookup_entry(_rom_body_crc32: u32) -> Option<&'static CartridgeDbEntry> {
    None
}

#[cfg(feature = "cartridge-db")]
fn lookup_override_for_crc(header: &Header, rom_body_crc32: u32) -> Option<CartridgeDbOverride> {
    let entry = lookup_entry(rom_body_crc32)?;

    // For now, only use the DB to upgrade legacy iNES identification.
    // This keeps the runtime behavior conservative while still allowing
    // validated board fixes such as old Datach dumps.
    if header.format() != RomFormat::INes {
        return None;
    }

    if entry.mapper == header.mapper() && entry.submapper == header.submapper() {
        return None;
    }

    Some(CartridgeDbOverride {
        mapper: entry.mapper,
        submapper: entry.submapper,
    })
}

#[cfg(feature = "cartridge-db")]
pub(crate) fn lookup_override(
    header: &Header,
    prg_rom: &[u8],
    chr_rom: &[u8],
) -> Option<CartridgeDbOverride> {
    lookup_override_for_crc(header, compute_rom_body_crc32(prg_rom, chr_rom))
}

#[cfg(not(feature = "cartridge-db"))]
pub(crate) fn lookup_override(
    _header: &Header,
    _prg_rom: &[u8],
    _chr_rom: &[u8],
) -> Option<CartridgeDbOverride> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cartridge::header::Header;

    fn ines_mapper16_header() -> Header {
        let mut rom = [0u8; 16];
        rom[0..4].copy_from_slice(b"NES\x1A");
        rom[4] = 16;
        rom[7] = 0x10;
        Header::parse(&rom).expect("valid iNES header")
    }

    #[cfg(feature = "cartridge-db")]
    #[test]
    fn finds_known_entry_by_crc() {
        let entry = lookup_entry(0x19E8_1461).expect("db entry expected");
        assert_eq!(entry.mapper, 157);
        assert_eq!(entry.submapper, 0);
        assert_eq!(entry.prg_rom_size, 256 * 1024);
        assert_eq!(entry.chr_rom_size, 0);
        assert_eq!(entry.chr_ram_size, 8 * 1024);
        assert_eq!(entry.work_ram_size, 0);
        assert_eq!(entry.save_ram_size, 0);
        assert!(!entry.has_battery);
    }

    #[cfg(feature = "cartridge-db")]
    #[test]
    fn upgrades_known_legacy_datach_crc_to_mapper157() {
        let header = ines_mapper16_header();
        let override_info =
            lookup_override_for_crc(&header, 0x19E8_1461).expect("override expected");
        assert_eq!(override_info.mapper, 157);
        assert_eq!(override_info.submapper, 0);
    }

    #[cfg(not(feature = "cartridge-db"))]
    #[test]
    fn disabled_feature_returns_no_entry_or_override() {
        let header = ines_mapper16_header();
        assert_eq!(lookup_entry(0x19E8_1461), None);
        assert_eq!(
            lookup_override(&header, &[0x0B, 0xCC, 0xB9, 0x9F], &[]),
            None
        );
    }

    #[test]
    fn ignores_unknown_crc_for_mapper16() {
        let header = ines_mapper16_header();
        assert_eq!(lookup_override(&header, &[1, 2, 3, 4], &[5, 6]), None);
    }

    #[cfg(feature = "cartridge-db")]
    #[test]
    fn computes_crc32_over_prg_and_chr_body() {
        let crc = compute_rom_body_crc32(&[1, 2, 3, 4], &[5, 6]);
        assert_eq!(crc, 0x81F6_7724);
    }
}

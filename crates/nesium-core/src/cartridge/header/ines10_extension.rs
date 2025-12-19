/// iNES 1.0-only bytes (header bytes 8..=15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct INes10Extension {
    /// Volatile PRG RAM size in 8 KiB units. iNES stores 0 for "assume 8 KiB".
    pub prg_ram_units: u8,
    /// iNES flags 9 (TV system).
    pub flags9: u8,
    /// iNES flags 10 (TV system / PRG RAM presence / bus conflicts).
    pub flags10: u8,
    /// Bytes 11..=15 are not specified by iNES 1.0 and are commonly expected
    /// to be zero; keep them around for diagnostics and strict validation.
    pub padding: [u8; 5],
}

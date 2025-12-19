//! Cartridge loading primitives.
//!
//! The first 16 bytes of every `.nes` ROM are the *iNES header*. It stores how much
//! PRG/CHR data the cartridge exposes, which mapper is required, and a few
//! compatibility flags. Modern dumps may use the extended **NES 2.0** flavour of the
//! header, so the parser in this module understands both variants and presents the
//! data as a beginner friendly [`Header`] enum.
//!
//! # Quick overview
//! - Read the first 16 bytes and pass them to [`Header::parse`].
//! - Inspect [`Header::mapper`] to pick or construct a concrete [`crate::cartridge::Mapper`]
//!   implementation and wrap it in a [`crate::cartridge::Cartridge`].
//! - Use [`Header::prg_rom_size`] / [`Header::chr_rom_size`] to slice the raw PRG/CHR
//!   sections out of the file.
//!
//! Unsupported or damaged headers turn into a descriptive [`Error`].

use bitflags::bitflags;

use crate::error::Error;

const NES_MAGIC: &[u8; 4] = b"NES\x1A";

/// Size of the fixed iNES header in bytes.
pub const NES_HEADER_LEN: usize = 16;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Flags6: u8 {
        const MIRRORING        = 0b0000_0001;
        const BATTERY          = 0b0000_0010;
        const TRAINER          = 0b0000_0100;
        const FOUR_SCREEN      = 0b0000_1000;
        const MAPPER_LOW_MASK  = 0b1111_0000;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Flags7: u8 {
        const VS_UNISYSTEM     = 0b0000_0001;
        const PLAYCHOICE_10    = 0b0000_0010;
        const NES2_DETECTION   = 0b0000_1100;
        const MAPPER_HIGH_MASK = 0b1111_0000;
    }
}

/// Layout mirroring type for the PPU nametables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mirroring {
    /// Two horizontal nametables that mirror vertically (common for NTSC games).
    Horizontal,
    /// Two vertical nametables that mirror horizontally.
    Vertical,
    /// Cartridge supplies its own four nametables.
    FourScreen,
    /// Single-screen mirroring using the first nametable (`$2000` region).
    SingleScreenLower,
    /// Single-screen mirroring using the second nametable (`$2400` region).
    SingleScreenUpper,
    /// Nametable mapping is fully controlled by the mapper via `map_nametable`.
    MapperControlled,
}

/// Identifies the header flavour encountered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RomFormat {
    /// The original iNES 1.0 specification.
    INes,
    /// NES 2.0 with extended sizing and metadata fields.
    Nes20,
    /// Rare prototypes that pre-date the iNES standard.
    Archaic,
}

impl RomFormat {
    fn from_flags7(flags7: Flags7) -> Self {
        match (flags7.bits() >> 2) & 0b11 {
            0b10 => Self::Nes20,
            0b00 => Self::INes,
            _ => Self::Archaic,
        }
    }
}

/// Video timing hints embedded in the header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TvSystem {
    /// NTSC (60Hz) timing.
    Ntsc,
    /// PAL (50Hz) timing.
    Pal,
    /// Cartridge can run on either timing without modification.
    Dual, // region free: supports NTSC and PAL timing
    Dendy, // hybrid timing used by some Famiclones
    Unknown,
}

/// NES 2.0 CPU/PPU timing mode (header byte 12 bits 0..=1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nes2CpuPpuTiming {
    /// 0: RP2C02 ("NTSC NES").
    Rp2c02,
    /// 1: RP2C07 ("Licensed PAL NES").
    Rp2c07,
    /// 2: Multiple-region.
    MultipleRegion,
    /// 3: UA6538 ("Dendy").
    Ua6538,
    /// Reserved/unknown values.
    Unknown(u8),
}

impl Nes2CpuPpuTiming {
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0b11 {
            0 => Self::Rp2c02,
            1 => Self::Rp2c07,
            2 => Self::MultipleRegion,
            3 => Self::Ua6538,
            _ => unreachable!("masked to 2 bits"),
        }
    }
}

/// Console type advertised by the iNES / NES 2.0 header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsoleType {
    /// Standard NES/Famicom cartridge.
    NesFamicom,
    /// Vs. System arcade hardware.
    VsSystem,
    /// PlayChoice-10 hardware.
    PlayChoice10,
    /// NES 2.0 extended console type (see [`Nes2ConsoleTypeData`]).
    Extended,
}

impl ConsoleType {
    fn from_bits(bits: u8) -> Self {
        match bits & 0b11 {
            0 => Self::NesFamicom,
            1 => Self::VsSystem,
            2 => Self::PlayChoice10,
            3 => Self::Extended,
            _ => unreachable!("masked to 2 bits"),
        }
    }
}

/// Vs. System PPU model id (NES 2.0 byte 13 low nibble, when console type is Vs. System).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VsPpuType {
    /// $0: Any RP2C03/RC2C03 variant.
    AnyRp2c03OrRc2c03,
    /// $2: RP2C04-0001.
    Rp2c04_0001,
    /// $3: RP2C04-0002.
    Rp2c04_0002,
    /// $4: RP2C04-0003.
    Rp2c04_0003,
    /// $5: RP2C04-0004.
    Rp2c04_0004,
    /// $8: RC2C05-01.
    Rc2c05_01,
    /// $9: RC2C05-02.
    Rc2c05_02,
    /// $A: RC2C05-03.
    Rc2c05_03,
    /// $B: RC2C05-04.
    Rc2c05_04,
    /// Reserved/unknown values.
    Unknown(u8),
}

impl VsPpuType {
    pub fn from_nibble(nibble: u8) -> Self {
        match nibble & 0x0F {
            0x0 => Self::AnyRp2c03OrRc2c03,
            0x2 => Self::Rp2c04_0001,
            0x3 => Self::Rp2c04_0002,
            0x4 => Self::Rp2c04_0003,
            0x5 => Self::Rp2c04_0004,
            0x8 => Self::Rc2c05_01,
            0x9 => Self::Rc2c05_02,
            0xA => Self::Rc2c05_03,
            0xB => Self::Rc2c05_04,
            other => Self::Unknown(other),
        }
    }
}

/// Vs. System hardware/protection type (NES 2.0 byte 13 high nibble, when console type is Vs. System).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VsHardwareType {
    /// $0: Vs. Unisystem (normal).
    UnisystemNormal,
    /// $1: Vs. Unisystem (RBI Baseball protection).
    UnisystemRbiBaseballProtection,
    /// $2: Vs. Unisystem (TKO Boxing protection).
    UnisystemTkoBoxingProtection,
    /// $3: Vs. Unisystem (Super Xevious protection).
    UnisystemSuperXeviousProtection,
    /// $4: Vs. Unisystem (Vs. Ice Climber Japan protection).
    UnisystemIceClimberJapanProtection,
    /// $5: Vs. Dual System (normal).
    DualSystemNormal,
    /// $6: Vs. Dual System (Raid on Bungeling Bay protection).
    DualSystemRaidOnBungelingBayProtection,
    /// Reserved/unknown values.
    Unknown(u8),
}

impl VsHardwareType {
    pub fn from_nibble(nibble: u8) -> Self {
        match nibble & 0x0F {
            0x0 => Self::UnisystemNormal,
            0x1 => Self::UnisystemRbiBaseballProtection,
            0x2 => Self::UnisystemTkoBoxingProtection,
            0x3 => Self::UnisystemSuperXeviousProtection,
            0x4 => Self::UnisystemIceClimberJapanProtection,
            0x5 => Self::DualSystemNormal,
            0x6 => Self::DualSystemRaidOnBungelingBayProtection,
            other => Self::Unknown(other),
        }
    }
}

/// NES 2.0 extended console type (NES 2.0 byte 13 low nibble, when console type is Extended).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtendedConsoleType {
    /// $0: Regular NES/Famicom/Dendy.
    Regular,
    /// $1: Nintendo Vs. System.
    VsSystem,
    /// $2: PlayChoice-10.
    PlayChoice10,
    /// $3: Regular Famiclone, but with CPU that supports Decimal Mode.
    FamicloneWithDecimalMode,
    /// $4: Regular NES/Famicom with EPSM module or plug-through cartridge.
    NesFamicomWithEpsm,
    /// $5: V.R. Technology VT01 with red/cyan STN palette.
    Vt01RedCyanStnPalette,
    /// $6: V.R. Technology VT02.
    Vt02,
    /// $7: V.R. Technology VT03.
    Vt03,
    /// $8: V.R. Technology VT09.
    Vt09,
    /// $9: V.R. Technology VT32.
    Vt32,
    /// $A: V.R. Technology VT369.
    Vt369,
    /// $B: UMC UM6578.
    UmcUm6578,
    /// $C: Famicom Network System.
    FamicomNetworkSystem,
    /// Reserved/unknown values.
    Unknown(u8),
}

impl ExtendedConsoleType {
    pub fn from_nibble(nibble: u8) -> Self {
        match nibble & 0x0F {
            0x0 => Self::Regular,
            0x1 => Self::VsSystem,
            0x2 => Self::PlayChoice10,
            0x3 => Self::FamicloneWithDecimalMode,
            0x4 => Self::NesFamicomWithEpsm,
            0x5 => Self::Vt01RedCyanStnPalette,
            0x6 => Self::Vt02,
            0x7 => Self::Vt03,
            0x8 => Self::Vt09,
            0x9 => Self::Vt32,
            0xA => Self::Vt369,
            0xB => Self::UmcUm6578,
            0xC => Self::FamicomNetworkSystem,
            other => Self::Unknown(other),
        }
    }
}

/// NES 2.0: interpretation of header byte 13 depends on [`ConsoleType`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nes2ConsoleTypeData {
    /// For standard NES/Famicom cartridges this byte is currently unused/reserved.
    NesFamicom { raw: u8 },
    /// Vs. System: upper nibble = hardware type, lower nibble = PPU type.
    VsSystem {
        hardware_type: VsHardwareType,
        ppu_type: VsPpuType,
    },
    /// PlayChoice-10: this byte is not specified by the NES 2.0 table; keep raw.
    PlayChoice10 { raw: u8 },
    /// Extended console type.
    Extended { console_type: ExtendedConsoleType },
}

/// NES 2.0: number of additional "miscellaneous" ROM regions.
///
/// This count is stored in header byte 14 bits 0..=1 and ranges from 0..=3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nes2MiscRomCount(pub u8);

/// NES 2.0: default expansion device id (header byte 15 bits 0..=5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nes2ExpansionDevice(pub u8);

/// NES 2.0: interpreted default expansion device (header byte 15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nes2DefaultExpansionDevice {
    Unspecified,
    StandardControllers,
    FourScoreOrSatelliteTwoExtraControllers,
    FamicomFourPlayersAdapterSimpleProtocol,
    VsSystem4016,
    VsSystem4017,
    Reserved,
    VsZapper,
    Zapper4017,
    TwoZappers,
    BandaiHyperShotLightgun,
    PowerPadSideA,
    PowerPadSideB,
    FamilyTrainerSideA,
    FamilyTrainerSideB,
    ArkanoidVausControllerNes,
    ArkanoidVausControllerFamicom,
    TwoVausControllersPlusFamicomDataRecorder,
    KonamiHyperShot,
    CoconutsPachinko,
    ExcitingBoxingPunchingBag,
    JissenMahjong,
    PartyTap,
    OekaKidsTablet,
    SunsoftBarcodeBattler,
    MiraclePianoKeyboard,
    PokkunMoguraaTapTapMat,
    TopRider,
    DoubleFisted,
    Famicom3dSystem,
    DoremikkoKeyboard,
    RobGyromite,
    FamicomDataRecorderSilentKeyboard,
    AsciiTurboFile,
    IgsBattleBox,
    FamilyBasicKeyboardPlusDataRecorder,
    PecKeyboard,
    Bit79Keyboard,
    SuborKeyboard,
    SuborKeyboardPlusMacroWinnersMouse,
    SuborKeyboardPlusSuborMouse4016,
    SnesMouse4016,
    Multicart,
    TwoSnesControllersReplacingStandard,
    RacerMateBicycle,
    UForce,
    RobStackUp,
    CityPatrolmanLightgun,
    SharpC1CassetteInterface,
    StandardControllerSwappedDirectionsAndBA,
    ExcaliburSudokuPad,
    AblPinball,
    GoldenNuggetCasinoExtraButtons,
    KedaKeyboard,
    SuborKeyboardPlusSuborMouse4017,
    PortTestController,
    BandaiMultiGamePlayerGamepadButtons,
    VenomTvDanceMat,
    LgTvRemoteControl,
    FamicomNetworkController,
    KingFishingController,
    CroakyKaraokeController,
    KingwonKeyboard,
    ZechengKeyboard,
    SuborKeyboardPlusL90RotatedPs2Mouse4017,
    Ps2KeyboardUm6578PortPlusPs2Mouse4017,
    Ps2MouseUm6578Port,
    YuxingMouse4016,
    SuborKeyboardPlusYuxingMouse4016,
    GigggleTvPump,
    BbkKeyboardPlusR90RotatedPs2Mouse4017,
    MagicalCooking,
    SnesMouse4017,
    Zapper4016,
    ArkanoidVausControllerPrototype,
    TvMahjongGameController,
    MahjongGekitouDensetsuController,
    SuborKeyboardPlusXInvertedPs2Mouse4017,
    IbmPcXtKeyboard,
    SuborKeyboardPlusMegaBookMouse,
    Unknown(u8),
}

impl Nes2DefaultExpansionDevice {
    pub fn from_id(id: u8) -> Self {
        match id {
            0x00 => Self::Unspecified,
            0x01 => Self::StandardControllers,
            0x02 => Self::FourScoreOrSatelliteTwoExtraControllers,
            0x03 => Self::FamicomFourPlayersAdapterSimpleProtocol,
            0x04 => Self::VsSystem4016,
            0x05 => Self::VsSystem4017,
            0x06 => Self::Reserved,
            0x07 => Self::VsZapper,
            0x08 => Self::Zapper4017,
            0x09 => Self::TwoZappers,
            0x0A => Self::BandaiHyperShotLightgun,
            0x0B => Self::PowerPadSideA,
            0x0C => Self::PowerPadSideB,
            0x0D => Self::FamilyTrainerSideA,
            0x0E => Self::FamilyTrainerSideB,
            0x0F => Self::ArkanoidVausControllerNes,
            0x10 => Self::ArkanoidVausControllerFamicom,
            0x11 => Self::TwoVausControllersPlusFamicomDataRecorder,
            0x12 => Self::KonamiHyperShot,
            0x13 => Self::CoconutsPachinko,
            0x14 => Self::ExcitingBoxingPunchingBag,
            0x15 => Self::JissenMahjong,
            0x16 => Self::PartyTap,
            0x17 => Self::OekaKidsTablet,
            0x18 => Self::SunsoftBarcodeBattler,
            0x19 => Self::MiraclePianoKeyboard,
            0x1A => Self::PokkunMoguraaTapTapMat,
            0x1B => Self::TopRider,
            0x1C => Self::DoubleFisted,
            0x1D => Self::Famicom3dSystem,
            0x1E => Self::DoremikkoKeyboard,
            0x1F => Self::RobGyromite,
            0x20 => Self::FamicomDataRecorderSilentKeyboard,
            0x21 => Self::AsciiTurboFile,
            0x22 => Self::IgsBattleBox,
            0x23 => Self::FamilyBasicKeyboardPlusDataRecorder,
            0x24 => Self::PecKeyboard,
            0x25 => Self::Bit79Keyboard,
            0x26 => Self::SuborKeyboard,
            0x27 => Self::SuborKeyboardPlusMacroWinnersMouse,
            0x28 => Self::SuborKeyboardPlusSuborMouse4016,
            0x29 => Self::SnesMouse4016,
            0x2A => Self::Multicart,
            0x2B => Self::TwoSnesControllersReplacingStandard,
            0x2C => Self::RacerMateBicycle,
            0x2D => Self::UForce,
            0x2E => Self::RobStackUp,
            0x2F => Self::CityPatrolmanLightgun,
            0x30 => Self::SharpC1CassetteInterface,
            0x31 => Self::StandardControllerSwappedDirectionsAndBA,
            0x32 => Self::ExcaliburSudokuPad,
            0x33 => Self::AblPinball,
            0x34 => Self::GoldenNuggetCasinoExtraButtons,
            0x35 => Self::KedaKeyboard,
            0x36 => Self::SuborKeyboardPlusSuborMouse4017,
            0x37 => Self::PortTestController,
            0x38 => Self::BandaiMultiGamePlayerGamepadButtons,
            0x39 => Self::VenomTvDanceMat,
            0x3A => Self::LgTvRemoteControl,
            0x3B => Self::FamicomNetworkController,
            0x3C => Self::KingFishingController,
            0x3D => Self::CroakyKaraokeController,
            0x3E => Self::KingwonKeyboard,
            0x3F => Self::ZechengKeyboard,
            0x40 => Self::SuborKeyboardPlusL90RotatedPs2Mouse4017,
            0x41 => Self::Ps2KeyboardUm6578PortPlusPs2Mouse4017,
            0x42 => Self::Ps2MouseUm6578Port,
            0x43 => Self::YuxingMouse4016,
            0x44 => Self::SuborKeyboardPlusYuxingMouse4016,
            0x45 => Self::GigggleTvPump,
            0x46 => Self::BbkKeyboardPlusR90RotatedPs2Mouse4017,
            0x47 => Self::MagicalCooking,
            0x48 => Self::SnesMouse4017,
            0x49 => Self::Zapper4016,
            0x4A => Self::ArkanoidVausControllerPrototype,
            0x4B => Self::TvMahjongGameController,
            0x4C => Self::MahjongGekitouDensetsuController,
            0x4D => Self::SuborKeyboardPlusXInvertedPs2Mouse4017,
            0x4E => Self::IbmPcXtKeyboard,
            0x4F => Self::SuborKeyboardPlusMegaBookMouse,
            other => Self::Unknown(other),
        }
    }
}

impl Nes2ExpansionDevice {
    pub fn kind(self) -> Nes2DefaultExpansionDevice {
        Nes2DefaultExpansionDevice::from_id(self.0)
    }
}

/// iNES-defined fields shared by both iNES 1.0 and NES 2.0.
///
/// NES 2.0 is explicitly designed to reuse the original iNES header layout for
/// the first 8 bytes (PRG/CHR LSB sizing + flags 6/7). The remaining bytes are
/// interpreted differently depending on the detected format, so they live in
/// per-format extension structures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct INesHeader {
    /// PRG ROM size least-significant byte (units of 16 KiB).
    pub prg_rom_lsb: u8,
    /// CHR ROM size least-significant byte (units of 8 KiB).
    pub chr_rom_lsb: u8,
    /// iNES flags 6.
    pub flags6: Flags6,
    /// iNES flags 7.
    pub flags7: Flags7,
}

impl INesHeader {
    fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            prg_rom_lsb: bytes[4],
            chr_rom_lsb: bytes[5],
            flags6: Flags6::from_bits_truncate(bytes[6]),
            flags7: Flags7::from_bits_truncate(bytes[7]),
        }
    }

    /// How the PPU nametables are mirrored.
    pub fn mirroring(&self) -> Mirroring {
        resolve_mirroring(self.flags6)
    }

    /// Whether the optional 512 byte trainer block is present between the header and PRG data.
    pub fn trainer_present(&self) -> bool {
        self.flags6.contains(Flags6::TRAINER)
    }
}

/// iNES 1.0-only bytes (header bytes 8..=10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct INes10Extension {
    /// Volatile PRG RAM size in 8 KiB units. iNES stores 0 for "assume 8 KiB".
    pub prg_ram_units: u8,
    /// iNES flags 9 (TV system).
    pub flags9: u8,
    /// iNES flags 10 (TV system / PRG RAM presence).
    pub flags10: u8,
    /// Bytes 11..=15 are not specified by iNES 1.0 and are commonly expected
    /// to be zero; keep them around for diagnostics and strict validation.
    pub padding: [u8; 5],
}

/// Parsed iNES 1.0 header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct INes10Header {
    pub base: INesHeader,
    pub ext: INes10Extension,
}

/// NES 2.0 extension bytes (header bytes 8..=15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nes2Extension {
    /// Byte 8 (submapper + mapper high bits).
    pub mapper_msb_submapper: u8,
    /// Byte 9 (PRG/CHR msb nibbles).
    pub prg_chr_msb: u8,
    /// Byte 10 (PRG RAM / PRG NVRAM shifts).
    pub prg_ram_shifts: u8,
    /// Byte 11 (CHR RAM / CHR NVRAM shifts).
    pub chr_ram_shifts: u8,
    /// Byte 12 (timing).
    pub timing: u8,
    /// Byte 13 (console-type dependent).
    pub console_type_data: u8,
    /// Byte 14 (misc ROMs).
    pub misc_roms: u8,
    /// Byte 15 (default expansion device).
    pub default_expansion_device: u8,
}

impl Nes2Extension {
    pub fn submapper(&self) -> u8 {
        self.mapper_msb_submapper >> 4
    }

    pub fn mapper_msb(&self) -> u8 {
        self.mapper_msb_submapper & 0x0F
    }

    pub fn prg_rom_msb(&self) -> u8 {
        self.prg_chr_msb & 0x0F
    }

    pub fn chr_rom_msb(&self) -> u8 {
        (self.prg_chr_msb >> 4) & 0x0F
    }

    pub fn prg_ram_shift(&self) -> u8 {
        self.prg_ram_shifts & 0x0F
    }

    pub fn prg_nvram_shift(&self) -> u8 {
        self.prg_ram_shifts >> 4
    }

    pub fn chr_ram_shift(&self) -> u8 {
        self.chr_ram_shifts & 0x0F
    }

    pub fn chr_nvram_shift(&self) -> u8 {
        self.chr_ram_shifts >> 4
    }

    pub fn console_type_data(&self, console_type: ConsoleType) -> Nes2ConsoleTypeData {
        match console_type {
            ConsoleType::NesFamicom => Nes2ConsoleTypeData::NesFamicom {
                raw: self.console_type_data,
            },
            ConsoleType::VsSystem => Nes2ConsoleTypeData::VsSystem {
                hardware_type: VsHardwareType::from_nibble((self.console_type_data >> 4) & 0x0F),
                ppu_type: VsPpuType::from_nibble(self.console_type_data & 0x0F),
            },
            ConsoleType::PlayChoice10 => Nes2ConsoleTypeData::PlayChoice10 {
                raw: self.console_type_data,
            },
            ConsoleType::Extended => Nes2ConsoleTypeData::Extended {
                console_type: ExtendedConsoleType::from_nibble(self.console_type_data & 0x0F),
            },
        }
    }

    pub fn misc_rom_count(&self) -> Nes2MiscRomCount {
        Nes2MiscRomCount(self.misc_roms & 0b11)
    }

    pub fn default_expansion_device(&self) -> Nes2ExpansionDevice {
        Nes2ExpansionDevice(self.default_expansion_device)
    }
}

/// Parsed NES 2.0 header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nes2Header {
    pub base: INesHeader,
    pub ext: Nes2Extension,
}

/// Parsed cartridge header, naturally distinguishing iNES 1.0 from NES 2.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Header {
    INes(INes10Header),
    Nes20(Nes2Header),
}

impl Header {
    /// Parse an iNES header from the given byte slice.
    pub fn parse(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < NES_HEADER_LEN {
            return Err(Error::TooShort {
                actual: bytes.len(),
            });
        }

        if &bytes[0..4] != NES_MAGIC {
            return Err(Error::InvalidMagic);
        }

        let base = INesHeader::from_bytes(bytes);
        let format = RomFormat::from_flags7(base.flags7);
        match format {
            RomFormat::INes => Ok(Self::INes(INes10Header {
                base,
                ext: INes10Extension {
                    prg_ram_units: bytes[8],
                    flags9: bytes[9],
                    flags10: bytes[10],
                    padding: bytes[11..16]
                        .try_into()
                        .expect("iNES padding length mismatch"),
                },
            })),
            RomFormat::Nes20 => Ok(Self::Nes20(Nes2Header {
                base,
                ext: Nes2Extension {
                    mapper_msb_submapper: bytes[8],
                    prg_chr_msb: bytes[9],
                    prg_ram_shifts: bytes[10],
                    chr_ram_shifts: bytes[11],
                    timing: bytes[12],
                    console_type_data: bytes[13],
                    misc_roms: bytes[14],
                    default_expansion_device: bytes[15],
                },
            })),
            RomFormat::Archaic => Err(Error::UnsupportedFormat(format)),
        }
    }

    /// Detected header flavour.
    pub fn format(&self) -> RomFormat {
        match self {
            Header::INes(_) => RomFormat::INes,
            Header::Nes20(_) => RomFormat::Nes20,
        }
    }

    /// Shared iNES-defined fields (bytes 4..=7).
    pub fn base(&self) -> &INesHeader {
        match self {
            Header::INes(header) => &header.base,
            Header::Nes20(header) => &header.base,
        }
    }

    /// Raw iNES flags 6.
    pub fn flags6(&self) -> Flags6 {
        self.base().flags6
    }

    /// Raw iNES flags 7.
    pub fn flags7(&self) -> Flags7 {
        self.base().flags7
    }

    /// Console type as advertised by flags 7 bits 0..=1.
    pub fn console_type(&self) -> ConsoleType {
        ConsoleType::from_bits(self.base().flags7.bits() & 0b11)
    }

    /// NES 2.0: console-type dependent byte 13 information.
    pub fn nes2_console_type_data(&self) -> Option<Nes2ConsoleTypeData> {
        match self {
            Header::Nes20(header) => Some(header.ext.console_type_data(self.console_type())),
            _ => None,
        }
    }

    /// NES 2.0: number of miscellaneous ROM regions (0..=3).
    pub fn nes2_misc_rom_count(&self) -> Option<Nes2MiscRomCount> {
        match self {
            Header::Nes20(header) => Some(header.ext.misc_rom_count()),
            _ => None,
        }
    }

    /// NES 2.0: default expansion device id (0..=63).
    pub fn nes2_default_expansion_device(&self) -> Option<Nes2ExpansionDevice> {
        match self {
            Header::Nes20(header) => Some(header.ext.default_expansion_device()),
            _ => None,
        }
    }

    /// NES 2.0: interpreted default expansion device.
    pub fn nes2_default_expansion_device_kind(&self) -> Option<Nes2DefaultExpansionDevice> {
        self.nes2_default_expansion_device()
            .map(Nes2ExpansionDevice::kind)
    }

    /// NES 2.0 CPU/PPU timing mode (byte 12 bits 0..=1).
    pub fn nes2_cpu_ppu_timing(&self) -> Option<Nes2CpuPpuTiming> {
        match self {
            Header::Nes20(header) => Some(Nes2CpuPpuTiming::from_bits(header.ext.timing)),
            _ => None,
        }
    }

    /// iNES 1.0: exposes the raw extension bytes 8..=15 (for diagnostics).
    pub fn ines_extension(&self) -> Option<INes10Extension> {
        match self {
            Header::INes(header) => Some(header.ext),
            _ => None,
        }
    }

    /// Mapper ID (0 == NROM, 1 == MMC1, ...).
    pub fn mapper(&self) -> u16 {
        match self {
            Header::INes(header) => combine_mapper(header.base.flags6, header.base.flags7, 0),
            Header::Nes20(header) => combine_mapper(
                header.base.flags6,
                header.base.flags7,
                header.ext.mapper_msb(),
            ),
        }
    }

    /// NES 2.0 submapper value. Always 0 for legacy iNES files.
    pub fn submapper(&self) -> u8 {
        match self {
            Header::INes(_) => 0,
            Header::Nes20(header) => header.ext.submapper(),
        }
    }

    /// How the PPU nametables are mirrored.
    pub fn mirroring(&self) -> Mirroring {
        self.base().mirroring()
    }

    /// Battery bit indicates the cartridge keeps RAM contents when powered off.
    pub fn battery_backed_ram(&self) -> bool {
        match self {
            Header::INes(header) => header.base.flags6.contains(Flags6::BATTERY),
            Header::Nes20(header) => {
                self.prg_nvram_size() != 0
                    || self.chr_nvram_size() != 0
                    || header.base.flags6.contains(Flags6::BATTERY)
            }
        }
    }

    /// Whether the optional 512 byte trainer block is present between the header and PRG data.
    pub fn trainer_present(&self) -> bool {
        self.base().trainer_present()
    }

    /// Amount of PRG ROM in bytes.
    pub fn prg_rom_size(&self) -> usize {
        match self {
            Header::INes(header) => (header.base.prg_rom_lsb as usize) * 16 * 1024,
            Header::Nes20(header) => {
                decode_nes2_rom_size(header.base.prg_rom_lsb, header.ext.prg_rom_msb(), 16 * 1024)
            }
        }
    }

    /// Amount of CHR ROM in bytes.
    pub fn chr_rom_size(&self) -> usize {
        match self {
            Header::INes(header) => (header.base.chr_rom_lsb as usize) * 8 * 1024,
            Header::Nes20(header) => {
                decode_nes2_rom_size(header.base.chr_rom_lsb, header.ext.chr_rom_msb(), 8 * 1024)
            }
        }
    }

    /// Volatile PRG RAM size (CPU accessible). Defaults to 8 KiB for legacy dumps that store 0.
    pub fn prg_ram_size(&self) -> usize {
        match self {
            Header::INes(header) => (header.ext.prg_ram_units.max(1) as usize) * 8 * 1024,
            Header::Nes20(header) => decode_nes2_ram_size(header.ext.prg_ram_shift()),
        }
    }

    /// Battery-backed PRG RAM size.
    pub fn prg_nvram_size(&self) -> usize {
        match self {
            Header::INes(header) => {
                if header.base.flags6.contains(Flags6::BATTERY) {
                    (header.ext.prg_ram_units.max(1) as usize) * 8 * 1024
                } else {
                    0
                }
            }
            Header::Nes20(header) => decode_nes2_ram_size(header.ext.prg_nvram_shift()),
        }
    }

    /// Volatile CHR RAM size located on the PPU side.
    pub fn chr_ram_size(&self) -> usize {
        match self {
            Header::INes(header) => {
                if header.base.chr_rom_lsb == 0 {
                    8 * 1024
                } else {
                    0
                }
            }
            Header::Nes20(header) => decode_nes2_ram_size(header.ext.chr_ram_shift()),
        }
    }

    /// Battery-backed CHR RAM size.
    pub fn chr_nvram_size(&self) -> usize {
        match self {
            Header::INes(_) => 0,
            Header::Nes20(header) => decode_nes2_ram_size(header.ext.chr_nvram_shift()),
        }
    }

    /// Set when the game targets the Vs. UniSystem arcade hardware.
    pub fn vs_unisystem(&self) -> bool {
        match self {
            Header::INes(header) => header.base.flags7.contains(Flags7::VS_UNISYSTEM),
            Header::Nes20(header) => {
                let console_type = header.base.flags7.bits() & 0b11;
                console_type == 1 || header.base.flags7.contains(Flags7::VS_UNISYSTEM)
            }
        }
    }

    /// Set when the cartridge contains PlayChoice-10 data.
    pub fn playchoice_10(&self) -> bool {
        match self {
            Header::INes(header) => header.base.flags7.contains(Flags7::PLAYCHOICE_10),
            Header::Nes20(header) => {
                let console_type = header.base.flags7.bits() & 0b11;
                console_type == 2 || header.base.flags7.contains(Flags7::PLAYCHOICE_10)
            }
        }
    }

    /// Region / timing hints described in the header.
    pub fn tv_system(&self) -> TvSystem {
        match self {
            Header::INes(header) => {
                let tv_bits = header.ext.flags10 & 0b11;
                match tv_bits {
                    0b00 => {
                        if header.ext.flags9 & 0b1 == 0 {
                            TvSystem::Ntsc
                        } else {
                            TvSystem::Pal
                        }
                    }
                    0b10 => TvSystem::Pal,
                    0b01 | 0b11 => TvSystem::Dual,
                    _ => TvSystem::Unknown,
                }
            }
            Header::Nes20(header) => match header.ext.timing & 0b11 {
                0b00 => TvSystem::Ntsc,
                0b01 => TvSystem::Pal,
                0b10 => TvSystem::Dual,
                0b11 => TvSystem::Dendy,
                _ => TvSystem::Unknown,
            },
        }
    }

    /// iNES 1.0 flags 10: hint whether the board has bus conflicts.
    pub fn ines_bus_conflicts(&self) -> Option<bool> {
        match self {
            Header::INes(header) => Some((header.ext.flags10 & 0x80) != 0),
            _ => None,
        }
    }

    /// iNES 1.0 flags 10: hint whether PRG RAM is present ($6000-$7FFF).
    ///
    /// Note: this is not part of the official iNES specification and is rarely used.
    pub fn ines_prg_ram_present_hint(&self) -> Option<bool> {
        match self {
            Header::INes(header) => Some((header.ext.flags10 & 0x10) == 0),
            _ => None,
        }
    }
}

fn resolve_mirroring(flags6: Flags6) -> Mirroring {
    if flags6.contains(Flags6::FOUR_SCREEN) {
        Mirroring::FourScreen
    } else if flags6.contains(Flags6::MIRRORING) {
        Mirroring::Vertical
    } else {
        Mirroring::Horizontal
    }
}

fn combine_mapper(flags6: Flags6, flags7: Flags7, upper: u8) -> u16 {
    let lower = (flags6.bits() >> 4) as u16;
    let middle = (flags7.bits() & 0xF0) as u16;
    let upper = (upper as u16) << 8;
    lower | middle | upper
}

fn decode_nes2_rom_size(lower: u8, upper_nibble: u8, unit: usize) -> usize {
    if upper_nibble != 0x0F {
        (((upper_nibble as usize) << 8) | lower as usize).saturating_mul(unit)
    } else {
        let exponent = ((lower & 0x3F) as u32).saturating_add(8);
        let base = 1usize.checked_shl(exponent).unwrap_or(usize::MAX);
        let multiplier = ((lower >> 6) as usize) + 1;
        base.saturating_mul(multiplier)
    }
}

fn decode_nes2_ram_size(nibble: u8) -> usize {
    if nibble == 0 {
        0
    } else {
        64usize << nibble.min(0x0F)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_header() {
        let header_bytes = [
            b'N',
            b'E',
            b'S',
            0x1A,        // magic
            2,           // 2 * 16 KiB PRG ROM
            1,           // 1 * 8 KiB CHR ROM
            0b0000_0001, // vertical mirroring
            0b0000_0000, // mapper 0
            0,           // prg ram
            0,           // tv system NTSC
            0,
            0,
            0,
            0,
            0,
            0, // padding
        ];

        let header = Header::parse(&header_bytes).expect("header parses");

        assert!(matches!(header.format(), RomFormat::INes));
        assert_eq!(header.prg_rom_size(), 2 * 16 * 1024);
        assert_eq!(header.chr_rom_size(), 8 * 1024);
        assert_eq!(header.mirroring(), Mirroring::Vertical);
        assert!(!header.trainer_present());
        assert_eq!(header.mapper(), 0);
        assert!(matches!(header.tv_system(), TvSystem::Ntsc));
        assert_eq!(header.prg_ram_size(), 8 * 1024);
        assert_eq!(header.prg_nvram_size(), 0);
        assert_eq!(header.submapper(), 0);
    }

    #[test]
    fn rejects_invalid_magic() {
        let mut header_bytes = [0u8; NES_HEADER_LEN];
        header_bytes[..4].copy_from_slice(b"NOPE");

        let err = Header::parse(&header_bytes).unwrap_err();
        assert!(matches!(err, Error::InvalidMagic));
    }

    #[test]
    fn parses_nes2_header() {
        let header_bytes = [
            b'N',
            b'E',
            b'S',
            0x1A,        // magic
            2,           // PRG LSB (2 * 16 KiB = 32 KiB)
            1,           // CHR LSB (1 * 8 KiB = 8 KiB)
            0b0000_0010, // horizontal mirroring
            0b0000_1000, // NES 2.0 format bits
            0b0011_0000, // mapper upper nibble = 0, submapper = 3
            0b0001_0000, // PRG MSB = 0x0, CHR MSB = 0x1 (adds 256 × 8 KiB)
            0b0010_0010, // PRG RAM = 256 B, PRG NVRAM = 256 B
            0b0100_0011, // CHR RAM = 512 B, CHR NVRAM = 1 KiB
            0b0000_0010, // timing: dual region
            0,
            0,
            0, // remaining padding
        ];

        let header = Header::parse(&header_bytes).expect("header parses");

        assert!(matches!(header.format(), RomFormat::Nes20));
        assert_eq!(header.mapper(), 0);
        assert_eq!(header.submapper(), 3);
        assert_eq!(header.prg_rom_size(), 2 * 16 * 1024);
        assert_eq!(header.chr_rom_size(), (1 + (1 << 8)) * 8 * 1024);
        assert_eq!(header.prg_ram_size(), 256);
        assert_eq!(header.prg_nvram_size(), 256);
        assert_eq!(header.chr_ram_size(), 512);
        assert_eq!(header.chr_nvram_size(), 1024);
        assert_eq!(header.mirroring(), Mirroring::Horizontal);
        assert!(matches!(header.tv_system(), TvSystem::Dual));
        assert_eq!(
            header.nes2_cpu_ppu_timing(),
            Some(Nes2CpuPpuTiming::MultipleRegion)
        );
    }

    #[test]
    fn parses_nes2_console_type_and_misc_fields() {
        let header_bytes = [
            b'N',
            b'E',
            b'S',
            0x1A,        // magic
            1,           // PRG LSB
            0,           // CHR LSB (CHR RAM)
            0,           // flags6
            0b0000_1001, // NES 2.0 + console type = Vs System
            0b0001_0000, // submapper 1
            0,           // PRG/CHR msb
            0,
            0,
            0,           // timing
            0xA3,        // hw type 0xA (unknown), ppu type 0x3 (RP2C04-0002)
            0b0000_0010, // misc ROM count = 2
            0x2A,        // expansion device id = 0x2A (Multicart)
        ];

        let header = Header::parse(&header_bytes).expect("header parses");

        assert_eq!(header.console_type(), ConsoleType::VsSystem);
        assert!(matches!(
            header.nes2_console_type_data(),
            Some(Nes2ConsoleTypeData::VsSystem {
                hardware_type: VsHardwareType::Unknown(0xA),
                ppu_type: VsPpuType::Rp2c04_0002
            })
        ));

        assert_eq!(header.nes2_misc_rom_count(), Some(Nes2MiscRomCount(2)));
        assert_eq!(
            header.nes2_default_expansion_device(),
            Some(Nes2ExpansionDevice(0x2A))
        );
        assert_eq!(
            header.nes2_default_expansion_device_kind(),
            Some(Nes2DefaultExpansionDevice::Multicart)
        );
        assert_eq!(header.nes2_cpu_ppu_timing(), Some(Nes2CpuPpuTiming::Rp2c02));
    }

    #[test]
    fn preserves_ines_padding_bytes() {
        let header_bytes = [
            b'N', b'E', b'S', 0x1A, // magic
            1, 0, 0, 0, // sizes + flags6/7
            0, 0, 0, // bytes 8..=10
            1, 2, 3, 4, 5, // bytes 11..=15 padding
        ];

        let header = Header::parse(&header_bytes).expect("header parses");
        let ext = header.ines_extension().expect("ines header");
        assert_eq!(ext.padding, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn parses_nes2_exponent_encoded_rom_size() {
        let header_bytes = [
            b'N',
            b'E',
            b'S',
            0x1A,
            0b0000_0001, // exponent = 1, multiplier = 1 (see formula)
            0,
            0,
            0b0000_1000, // NES 2.0
            0,
            0b0000_1111, // PRG MSB = 0xF triggers exponent encoding
            0,
            0,
            0,
            0,
            0,
            0,
        ];

        let header = Header::parse(&header_bytes).expect("header parses");

        assert!(matches!(header.format(), RomFormat::Nes20));
        assert_eq!(header.prg_rom_size(), 512);
        assert_eq!(header.chr_rom_size(), 0);
    }
}

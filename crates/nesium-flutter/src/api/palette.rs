use flutter_rust_bridge::frb;

use nesium_core::ppu::palette::PaletteKind as CorePaletteKind;

// NOTE: FRB cannot directly generate Dart enums for types defined in other crates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteKind {
    NesdevNtsc,
    FbxCompositeDirect,
    SonyCxa2025AsUs,
    Pal2c07,
    Mesen2C02,
    RawLinear,
}

impl From<PaletteKind> for CorePaletteKind {
    fn from(value: PaletteKind) -> Self {
        match value {
            PaletteKind::NesdevNtsc => CorePaletteKind::NesdevNtsc,
            PaletteKind::FbxCompositeDirect => CorePaletteKind::FbxCompositeDirect,
            PaletteKind::SonyCxa2025AsUs => CorePaletteKind::SonyCxa2025AsUs,
            PaletteKind::Pal2c07 => CorePaletteKind::Pal2c07,
            PaletteKind::Mesen2C02 => CorePaletteKind::Mesen2C02,
            PaletteKind::RawLinear => CorePaletteKind::RawLinear,
        }
    }
}

impl From<CorePaletteKind> for PaletteKind {
    fn from(value: CorePaletteKind) -> Self {
        match value {
            CorePaletteKind::NesdevNtsc => PaletteKind::NesdevNtsc,
            CorePaletteKind::FbxCompositeDirect => PaletteKind::FbxCompositeDirect,
            CorePaletteKind::SonyCxa2025AsUs => PaletteKind::SonyCxa2025AsUs,
            CorePaletteKind::Pal2c07 => PaletteKind::Pal2c07,
            CorePaletteKind::Mesen2C02 => PaletteKind::Mesen2C02,
            CorePaletteKind::RawLinear => PaletteKind::RawLinear,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PalettePresetInfo {
    pub kind: PaletteKind,
    pub description: String,
}

#[frb]
pub fn palette_presets() -> Vec<PalettePresetInfo> {
    CorePaletteKind::all()
        .iter()
        .copied()
        .map(|k| PalettePresetInfo {
            kind: PaletteKind::from(k),
            description: k.description().to_string(),
        })
        .collect()
}

#[frb]
pub fn set_palette_preset(kind: PaletteKind) -> Result<(), String> {
    crate::runtime_handle()
        .set_palette_kind(CorePaletteKind::from(kind))
        .map_err(|e| e.to_string())
}

#[frb]
pub fn set_palette_pal_data(data: Vec<u8>) -> Result<(), String> {
    crate::runtime_handle()
        .set_palette_from_pal_data(&data)
        .map_err(|e| e.to_string())
}

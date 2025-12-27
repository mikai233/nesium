use flutter_rust_bridge::frb;
use nesium_core::ppu::palette::PaletteKind;

#[derive(Debug, Clone)]
pub struct PalettePresetInfo {
    pub id: String,
    pub description: String,
}

fn parse_palette_kind(id: &str) -> Option<PaletteKind> {
    match id {
        "nesdev-ntsc" => Some(PaletteKind::NesdevNtsc),
        "fbx-composite-direct" => Some(PaletteKind::FbxCompositeDirect),
        "sony-cxa2025as-us" => Some(PaletteKind::SonyCxa2025AsUs),
        "pal-2c07" => Some(PaletteKind::Pal2c07),
        "raw-linear" => Some(PaletteKind::RawLinear),
        _ => None,
    }
}

#[frb]
pub fn palette_presets() -> Vec<PalettePresetInfo> {
    PaletteKind::all()
        .iter()
        .copied()
        .map(|k| PalettePresetInfo {
            id: k.as_str().to_string(),
            description: k.description().to_string(),
        })
        .collect()
}

#[frb]
pub fn set_palette_preset(id: String) -> Result<(), String> {
    let kind = parse_palette_kind(&id).ok_or_else(|| format!("unknown palette preset: {id}"))?;
    crate::runtime_handle()
        .set_palette_kind(kind)
        .map_err(|e| e.to_string())
}

#[frb]
pub fn set_palette_pal_data(data: Vec<u8>) -> Result<(), String> {
    crate::runtime_handle()
        .set_palette_from_pal_data(&data)
        .map_err(|e| e.to_string())
}

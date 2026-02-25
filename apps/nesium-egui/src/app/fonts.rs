use eframe::egui::{Context, FontData, FontDefinitions, FontFamily};

#[cfg(use_subset_fonts)]
const NOTO_REGULAR: &[u8] = include_bytes!("../../assets/fonts/subset/NotoSansSC-Regular.ttf");
#[cfg(use_subset_fonts)]
const NOTO_MEDIUM: &[u8] = include_bytes!("../../assets/fonts/subset/NotoSansSC-Medium.ttf");
#[cfg(use_subset_fonts)]
const NOTO_BOLD: &[u8] = include_bytes!("../../assets/fonts/subset/NotoSansSC-Bold.ttf");

#[cfg(not(use_subset_fonts))]
const NOTO_REGULAR: &[u8] =
    include_bytes!("../../../../apps/nesium_flutter/assets/fonts/NotoSansSC-Regular.ttf");
#[cfg(not(use_subset_fonts))]
const NOTO_MEDIUM: &[u8] =
    include_bytes!("../../../../apps/nesium_flutter/assets/fonts/NotoSansSC-Medium.ttf");
#[cfg(not(use_subset_fonts))]
const NOTO_BOLD: &[u8] =
    include_bytes!("../../../../apps/nesium_flutter/assets/fonts/NotoSansSC-Bold.ttf");

/// Install Noto Sans SC as the primary font.
pub fn setup_fonts(ctx: &Context) -> bool {
    let mut fonts = FontDefinitions::default();

    // Load embedded Noto Sans SC
    fonts.font_data.insert(
        "noto_regular".to_owned(),
        FontData::from_static(NOTO_REGULAR).into(),
    );
    fonts.font_data.insert(
        "noto_medium".to_owned(),
        FontData::from_static(NOTO_MEDIUM).into(),
    );
    fonts.font_data.insert(
        "noto_bold".to_owned(),
        FontData::from_static(NOTO_BOLD).into(),
    );

    // Set as primary for both Proportional and Monospace to ensure consistent UI
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "noto_regular".to_owned());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "noto_regular".to_owned());

    ctx.set_fonts(fonts);
    tracing::info!("installed embedded Noto Sans SC fonts as primary");
    true
}

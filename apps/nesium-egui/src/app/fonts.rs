use std::borrow::Cow;

use eframe::egui::{Context, FontData, FontDefinitions, FontFamily};
use fontdb::{Database, Family, Query};
use ttf_parser::{Face, Tag};

/// Install a CJK-capable system font as a fallback.
///
/// We avoid shipping huge embedded font files and keep egui's built-in Latin UI fonts
/// as the primary choice. When CJK glyphs are needed, egui will fall back to the system font.
pub fn install_cjk_font(ctx: &Context) -> bool {
    let mut db = Database::new();
    db.load_system_fonts();

    let mut candidates: Vec<Cow<'static, str>> = Vec::new();
    if let Ok(name) = std::env::var("NESIUM_EGUI_FONT")
        && !name.trim().is_empty()
    {
        candidates.push(Cow::Owned(name));
    }
    candidates.extend(default_candidates().into_iter().map(Cow::Borrowed));

    let probe = ['你', '汉', '测', '试'];

    for family in candidates {
        let id = db.query(&Query {
            families: &[Family::Name(&family)],
            ..Default::default()
        });
        let Some(id) = id else { continue };
        let Some(face) = db.face(id) else { continue };

        let Some((bytes, index)) = db.with_face_data(face.id, |data, idx| (data.to_vec(), idx))
        else {
            continue;
        };
        if !is_compatible_truetype(&bytes, index, &probe) {
            continue;
        }

        let mut fonts = FontDefinitions::default();
        let mut fd = FontData::from_owned(bytes);
        fd.index = index;
        fonts.font_data.insert("sys_cjk".to_owned(), fd.into());

        // Add as a fallback (NOT primary) to avoid a full UI text disappearance on edge cases.
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .push("sys_cjk".to_owned());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .push("sys_cjk".to_owned());

        ctx.set_fonts(fonts);
        tracing::info!(family = %family, index, "installed system CJK font fallback");
        return true;
    }

    tracing::warn!("no compatible system CJK font found; using egui defaults");
    false
}

fn default_candidates() -> Vec<&'static str> {
    #[cfg(target_os = "windows")]
    {
        return vec![
            "Microsoft YaHei UI",
            "Microsoft YaHei",
            "DengXian",
            "SimHei",
            "SimSun",
        ];
    }

    #[cfg(target_os = "macos")]
    {
        vec!["PingFang SC", "Hiragino Sans GB", "Heiti SC", "Songti SC"]
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        return vec![
            "Noto Sans CJK SC",
            "Noto Sans SC",
            "Source Han Sans SC",
            "WenQuanYi Micro Hei",
            "WenQuanYi Zen Hei",
        ];
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
    {
        vec![]
    }
}

fn is_compatible_truetype(data: &[u8], index: u32, probe: &[char]) -> bool {
    let Ok(face) = Face::parse(data, index) else {
        return false;
    };
    let raw = face.raw_face();

    // Filter out CFF-only OTF fonts (common source of "text renders nowhere").
    if raw.table(Tag::from_bytes(b"glyf")).is_none() {
        return false;
    }

    probe.iter().all(|&ch| face.glyph_index(ch).is_some())
}

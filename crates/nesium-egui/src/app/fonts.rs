use eframe::egui::{Context as EguiContext, FontData, FontDefinitions, FontFamily};

pub fn install_cjk_font(ctx: &EguiContext) {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();

    let target_chars = ['你', '汉', '测', '试'];
    let mut picked: Option<Vec<u8>> = None;

    for face in db.faces() {
        let has_all = db.with_face_data(face.id, |data, idx| {
            let face = match ttf_parser::Face::parse(data, idx) {
                Ok(f) => f,
                Err(_) => return false,
            };
            target_chars
                .iter()
                .all(|ch| face.glyph_index(*ch).is_some())
        });
        if has_all == Some(true)
            && let Some(bytes) = db.with_face_data(face.id, |data, _| data.to_vec())
        {
            picked = Some(bytes);
            break;
        }
    }

    if let Some(data) = picked {
        let mut fonts = FontDefinitions::default();
        fonts
            .font_data
            .insert("ui_cjk".to_string(), FontData::from_owned(data).into());
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "ui_cjk".to_string());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .push("ui_cjk".to_string());
        ctx.set_fonts(fonts);
    }
}

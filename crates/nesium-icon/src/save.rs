use skia_safe::{EncodedImageFormat, Surface};
use std::fs::File;
use std::io::Write;

/// Helper to save the rendered surface as a PNG.
pub fn save_surface(surface: &mut Surface, path: &str) -> Result<(), String> {
    let image = surface.image_snapshot();
    let data = image
        .encode(None, EncodedImageFormat::PNG, 100)
        .ok_or("Failed to encode image")?;

    let mut file = File::create(path).map_err(|e| e.to_string())?;
    file.write_all(data.as_bytes()).map_err(|e| e.to_string())?;

    println!("Successfully generated: {}", path);
    Ok(())
}

mod background;
mod controller;
mod dimensions;
mod ring;
mod save;

use crate::background::draw_background;
use crate::controller::draw_controller;
use crate::dimensions::{HEIGHT, WIDTH};
use crate::ring::draw_dashed_ring;
use crate::save::save_surface;
use skia_safe::image::CachingHint;
use skia_safe::surfaces::raster_n32_premul;
use skia_safe::{AlphaType, ColorType, ImageInfo, Surface};
#[cfg(feature = "svg")]
use skia_safe::{Rect, svg};

/// Default render dimension (square).
pub const DEFAULT_ICON_SIZE: u32 = WIDTH as u32;

/// A single RGBA layer (unpremultiplied).
#[derive(Clone)]
pub struct IconLayer {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Separate background and foreground layers.
#[derive(Clone)]
pub struct IconLayers {
    pub background: IconLayer,
    pub foreground: IconLayer,
}

/// Render layered icon assets (background + foreground) as unpremultiplied RGBA.
/// Both layers are square and returned at `size` (default: 1024).
pub fn render_layers(size: u32) -> IconLayers {
    let (mut bg_surface, mut fg_surface) =
        render_base_layers().expect("failed to render icon layers");

    let bg = premul_to_unpremul(read_premul_rgba(&mut bg_surface));
    let fg = premul_to_unpremul(read_premul_rgba(&mut fg_surface));

    if size == DEFAULT_ICON_SIZE {
        return IconLayers {
            background: IconLayer {
                width: DEFAULT_ICON_SIZE,
                height: DEFAULT_ICON_SIZE,
                rgba: bg,
            },
            foreground: IconLayer {
                width: DEFAULT_ICON_SIZE,
                height: DEFAULT_ICON_SIZE,
                rgba: fg,
            },
        };
    }

    IconLayers {
        background: IconLayer {
            width: size,
            height: size,
            rgba: resample_nearest(&bg, DEFAULT_ICON_SIZE, DEFAULT_ICON_SIZE, size, size),
        },
        foreground: IconLayer {
            width: size,
            height: size,
            rgba: resample_nearest(&fg, DEFAULT_ICON_SIZE, DEFAULT_ICON_SIZE, size, size),
        },
    }
}

/// Render the icon (background + foreground composited) and return RGBA8 bytes with **unpremultiplied** alpha.
/// The returned buffer length is always `size * size * 4`.
pub fn render_rgba_unpremul(size: u32) -> Vec<u8> {
    let layers = render_layers(size);
    composite_layers(&layers.background, &layers.foreground, size)
}

/// Convenience helper for the binary: renders the base icon and saves a PNG.
pub fn render_png(path: &str) -> Result<(), String> {
    let mut surface = render_base_surface()?;
    save_surface(&mut surface, path)
}

/// Convenience helper for the binary: renders the base icon and saves an SVG.
#[cfg(feature = "svg")]
pub fn render_svg(path: &str) -> Result<(), String> {
    let bounds = Rect::from_wh(WIDTH as f32, HEIGHT as f32);
    let mut canvas = svg::Canvas::new(bounds, None);

    draw_background(&mut canvas);
    draw_dashed_ring(&mut canvas);
    draw_controller(&mut canvas);

    let data = canvas.end();
    let mut file = File::create(path).map_err(|e| e.to_string())?;
    file.write_all(data.as_bytes()).map_err(|e| e.to_string())?;

    println!("Successfully generated: {}", path);
    Ok(())
}

/// Save background + foreground layers as separate PNGs.
///
/// This is useful for platforms that support layered/adaptive icons.
/// The saved PNGs contain the background-only and foreground-only layers.
pub fn render_layer_pngs(bg_path: &str, fg_path: &str) -> Result<(), String> {
    render_layer_pngs_sized(bg_path, fg_path, DEFAULT_ICON_SIZE)
}

/// Same as `render_layer_pngs`, but allows choosing the output size.
pub fn render_layer_pngs_sized(bg_path: &str, fg_path: &str, size: u32) -> Result<(), String> {
    let layers = render_layers(size);
    save_unpremul_rgba_png(&layers.background.rgba, size, size, bg_path)?;
    save_unpremul_rgba_png(&layers.foreground.rgba, size, size, fg_path)?;
    Ok(())
}

fn save_unpremul_rgba_png(
    rgba_unpremul: &[u8],
    width: u32,
    height: u32,
    path: &str,
) -> Result<(), String> {
    let premul = unpremul_to_premul(rgba_unpremul);

    let mut surface = raster_n32_premul((width as i32, height as i32))
        .ok_or_else(|| "Failed to create surface".to_string())?;

    // Write RGBA8888 premultiplied pixels into the surface.
    let info = ImageInfo::new(
        (width as i32, height as i32),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );
    let row_bytes = (width * 4) as usize;

    let ok = surface
        .canvas()
        .write_pixels(&info, &premul, row_bytes, (0, 0));
    if !ok {
        return Err("Failed to write pixels".to_string());
    }

    save_surface(&mut surface, path)
}

fn unpremul_to_premul(rgba_unpremul: &[u8]) -> Vec<u8> {
    let mut out = rgba_unpremul.to_vec();
    for chunk in out.chunks_exact_mut(4) {
        let a = chunk[3] as u16;
        if a == 0 {
            chunk[0] = 0;
            chunk[1] = 0;
            chunk[2] = 0;
            continue;
        }

        // Premultiply: c_premul = c_unpremul * alpha / 255
        chunk[0] = ((chunk[0] as u16 * a + 127) / 255) as u8;
        chunk[1] = ((chunk[1] as u16 * a + 127) / 255) as u8;
        chunk[2] = ((chunk[2] as u16 * a + 127) / 255) as u8;
    }
    out
}

fn render_base_surface() -> Result<Surface, String> {
    let mut surface = raster_n32_premul((WIDTH, HEIGHT)).ok_or("Failed to create surface")?;
    let canvas = surface.canvas();

    draw_background(canvas);
    draw_dashed_ring(canvas);
    draw_controller(canvas);

    Ok(surface)
}

fn render_base_layers() -> Result<(Surface, Surface), String> {
    let mut bg = raster_n32_premul((WIDTH, HEIGHT)).ok_or("Failed to create surface")?;
    let mut fg = raster_n32_premul((WIDTH, HEIGHT)).ok_or("Failed to create surface")?;

    draw_background(bg.canvas());
    draw_dashed_ring(fg.canvas());
    draw_controller(fg.canvas());

    Ok((bg, fg))
}

fn read_premul_rgba(surface: &mut Surface) -> Vec<u8> {
    let info = surface
        .image_info()
        .with_color_type(ColorType::RGBA8888)
        .with_alpha_type(AlphaType::Premul);
    let w = info.width();
    let h = info.height();
    let mut pixels = vec![0u8; (w * h * 4) as usize];
    let row_bytes = info.min_row_bytes();

    let ok = surface.image_snapshot().read_pixels(
        &info,
        &mut pixels,
        row_bytes,
        (0, 0),
        CachingHint::Allow,
    );
    assert!(ok, "failed to read pixels");

    pixels
}

fn premul_to_unpremul(pixels: Vec<u8>) -> Vec<u8> {
    let mut out = pixels;
    for chunk in out.chunks_exact_mut(4) {
        let a = chunk[3] as f32;
        if a <= 0.0 {
            chunk[0] = 0;
            chunk[1] = 0;
            chunk[2] = 0;
            continue;
        }

        // Undo premultiplication: c_unpremul = c_premul / alpha * 255
        let scale = 255.0 / a;
        chunk[0] = ((chunk[0] as f32 * scale).round() as i32).clamp(0, 255) as u8;
        chunk[1] = ((chunk[1] as f32 * scale).round() as i32).clamp(0, 255) as u8;
        chunk[2] = ((chunk[2] as f32 * scale).round() as i32).clamp(0, 255) as u8;
    }
    out
}

fn resample_nearest(src: &[u8], src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> Vec<u8> {
    let mut dst = vec![0u8; (dst_w * dst_h * 4) as usize];
    let x_ratio = src_w as f32 / dst_w as f32;
    let y_ratio = src_h as f32 / dst_h as f32;

    for y in 0..dst_h {
        let src_y = (y as f32 * y_ratio).floor() as u32;
        for x in 0..dst_w {
            let src_x = (x as f32 * x_ratio).floor() as u32;
            let src_idx = ((src_y * src_w + src_x) * 4) as usize;
            let dst_idx = ((y * dst_w + x) * 4) as usize;
            dst[dst_idx..dst_idx + 4].copy_from_slice(&src[src_idx..src_idx + 4]);
        }
    }

    dst
}

fn composite_layers(bg: &IconLayer, fg: &IconLayer, size: u32) -> Vec<u8> {
    let mut out = if bg.width == size && bg.height == size {
        bg.rgba.clone()
    } else {
        resample_nearest(&bg.rgba, bg.width, bg.height, size, size)
    };

    let fg_resampled_owned = if fg.width == size && fg.height == size {
        None
    } else {
        Some(resample_nearest(&fg.rgba, fg.width, fg.height, size, size))
    };
    let fg_resampled: &[u8] = fg_resampled_owned.as_deref().unwrap_or(&fg.rgba);

    for (idx, chunk) in out.chunks_exact_mut(4).enumerate() {
        let s = &fg_resampled[idx * 4..idx * 4 + 4];
        let sa = s[3] as f32 / 255.0;
        if sa <= 0.0 {
            continue;
        }
        let da = chunk[3] as f32 / 255.0;
        let out_a = sa + da * (1.0 - sa);
        if out_a <= 0.0 {
            chunk[0] = 0;
            chunk[1] = 0;
            chunk[2] = 0;
            chunk[3] = 0;
            continue;
        }

        for i in 0..3 {
            let sc = s[i] as f32 / 255.0;
            let dc = chunk[i] as f32 / 255.0;
            let oc = (sc * sa + dc * da * (1.0 - sa)) / out_a;
            chunk[i] = (oc * 255.0).round().clamp(0.0, 255.0) as u8;
        }
        chunk[3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
    }

    out
}

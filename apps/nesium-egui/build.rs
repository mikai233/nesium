use nesium_icon::{DEFAULT_ICON_SIZE, IconLayers};
use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

const ICON_SIZE: u32 = 256;
const BASE_RENDER_SIZE: u32 = DEFAULT_ICON_SIZE;
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
const APP_ID: &str = "com.mikai233.nesium";

#[derive(Clone, Copy)]
struct LayoutParams {
    pad_ratio: f32,    // background inset ratio per side (e.g. 0.11 => 11% per side)
    radius_ratio: f32, // corner radius ratio to final size (e.g. 0.20 => 20% of size)
    fg_scale: f32,     // foreground scale relative to background box
}

fn layout() -> LayoutParams {
    LayoutParams {
        pad_ratio: 0.11,
        radius_ratio: 0.20,
        fg_scale: 1.,
    }
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set by Cargo"));

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../nesium-icon/src");
    println!("cargo:rerun-if-changed=../nesium-icon/Cargo.toml");
    println!("cargo:rustc-env=NESIUM_APP_ID={}", APP_ID);

    let layers = nesium_icon::render_layers(BASE_RENDER_SIZE);
    let icon_rgba = compose_icon(&layers, ICON_SIZE, layout());

    write_icon_bin(&out_dir, &icon_rgba).expect("failed to write icon_rgba.bin");
    write_icon_rs(&out_dir).expect("failed to write egui_icon.rs");
    generate_bundle_icons(&layers);

    #[cfg(target_os = "windows")]
    generate_windows_resources(&out_dir, &layers);

    #[cfg(target_os = "linux")]
    generate_linux_assets(&layers);

    #[cfg(target_os = "macos")]
    generate_macos_assets(&layers);
}

fn write_icon_bin(out_dir: &Path, rgba: &[u8]) -> std::io::Result<()> {
    fs::write(out_dir.join("icon_rgba.bin"), rgba)
}

fn write_icon_rs(out_dir: &Path) -> std::io::Result<()> {
    let content = format!(
        r#"pub const ICON_WIDTH: u32 = {size};
pub const ICON_HEIGHT: u32 = {size};
pub static ICON_RGBA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/icon_rgba.bin"));
"#,
        size = ICON_SIZE,
    );

    fs::write(out_dir.join("egui_icon.rs"), content)
}

fn generate_bundle_icons(layers: &IconLayers) {
    use ico::{IconDir, IconDirEntry, IconImage, ResourceType};

    let generated_dir = target_root().join("generated");
    fs::create_dir_all(&generated_dir).expect("create target/generated");

    // PNG (used by Linux packaging and as a general-purpose fallback)
    let rgba_256 = compose_icon(layers, ICON_SIZE, layout());
    let png_path = generated_dir.join("nesium.png");
    write_png_file(&png_path, ICON_SIZE, ICON_SIZE, &rgba_256)
        .expect("write target/generated/nesium.png");

    // ICO (used by Windows packaging)
    let ico_path = generated_dir.join("nesium.ico");
    let sizes = [16u32, 32, 48, 64, 128, ICON_SIZE];
    let mut icon_dir = IconDir::new(ResourceType::Icon);
    for size in sizes {
        let rgba = compose_icon(layers, size, layout());
        let image = IconImage::from_rgba_data(size, size, rgba);
        let entry = IconDirEntry::encode(&image).expect("encode ICO entry");
        icon_dir.add_entry(entry);
    }
    let mut file = File::create(&ico_path).expect("create target/generated/nesium.ico");
    icon_dir
        .write(&mut file)
        .expect("write target/generated/nesium.ico");

    // ICNS (used by macOS packaging)
    let png_bytes =
        encode_png_to_vec(ICON_SIZE, ICON_SIZE, &rgba_256).expect("encode PNG for icns");
    let icns_path = generated_dir.join("nesium.icns");
    write_icns_from_png(&icns_path, &png_bytes).expect("write target/generated/nesium.icns");
}

#[cfg(target_os = "windows")]
fn generate_windows_resources(out_dir: &Path, layers: &IconLayers) {
    use ico::{IconDir, IconDirEntry, IconImage, ResourceType};
    use winres::WindowsResource;

    let sizes = [16u32, 32, 48, 64, 128, ICON_SIZE];
    let mut icon_dir = IconDir::new(ResourceType::Icon);

    for size in sizes {
        let rgba = compose_icon(layers, size, layout());
        let image = IconImage::from_rgba_data(size, size, rgba);
        let entry = IconDirEntry::encode(&image).expect("encode ICO entry");
        icon_dir.add_entry(entry);
    }

    let ico_path = out_dir.join("app.ico");
    let mut file = File::create(&ico_path).expect("create app.ico");
    icon_dir
        .write(&mut file)
        .expect("write ICO to OUT_DIR/app.ico");

    let mut res = WindowsResource::new();
    res.set_icon(
        ico_path
            .to_str()
            .expect("Windows resource path should be valid UTF-8"),
    );
    res.compile().expect("embed Windows icon resource");
}

#[cfg(target_os = "linux")]
fn generate_linux_assets(layers: &IconLayers) {
    let generated_dir = target_root().join("generated");
    let icon_dir = generated_dir.join("icons");
    fs::create_dir_all(&icon_dir).expect("create target/generated/icons");

    let rgba = compose_icon(layers, ICON_SIZE, layout());
    let icon_path = icon_dir.join(format!("{APP_ID}.png"));
    write_png_file(&icon_path, ICON_SIZE, ICON_SIZE, &rgba)
        .expect("write target/generated/icons/com.yourorg.nesium.png");

    let desktop_path = generated_dir.join(format!("{APP_ID}.desktop"));
    fs::create_dir_all(&generated_dir).expect("create target/generated");
    fs::write(&desktop_path, desktop_entry()).expect("write .desktop sample");
}

#[cfg(target_os = "macos")]
fn generate_macos_assets(layers: &IconLayers) {
    let generated_dir = target_root().join("generated");
    fs::create_dir_all(&generated_dir).expect("create target/generated");

    let rgba = compose_icon(layers, ICON_SIZE, layout());
    let png_bytes =
        encode_png_to_vec(ICON_SIZE, ICON_SIZE, &rgba).expect("encode PNG for macOS icns");
    let icns_path = generated_dir.join("nesium.icns");

    if let Err(err) = write_icns_from_png(&icns_path, &png_bytes) {
        println!("cargo:warning=Failed to write macOS icns (optional step): {err}");
    }
}

fn compose_icon(layers: &IconLayers, target: u32, layout: LayoutParams) -> Vec<u8> {
    let mut canvas = vec![0u8; (target * target * 4) as usize];

    let bg_len = (target as f32 * (1.0 - 2.0 * layout.pad_ratio))
        .round()
        .clamp(1.0, target as f32) as u32;
    let fg_len = (bg_len as f32 * layout.fg_scale)
        .round()
        .clamp(1.0, bg_len as f32) as u32;

    let bg = resample_nearest(
        &layers.background.rgba,
        layers.background.width,
        layers.background.height,
        bg_len,
        bg_len,
    );
    let fg = resample_nearest(
        &layers.foreground.rgba,
        layers.foreground.width,
        layers.foreground.height,
        fg_len,
        fg_len,
    );

    let bg_off = ((target - bg_len) / 2) as i32;
    let fg_off = ((target - fg_len) / 2) as i32;

    blit_over(
        &mut canvas,
        target,
        target,
        &bg,
        bg_len,
        bg_len,
        bg_off,
        bg_off,
    );
    blit_over(
        &mut canvas,
        target,
        target,
        &fg,
        fg_len,
        fg_len,
        fg_off,
        fg_off,
    );

    apply_round_mask(
        &mut canvas,
        target,
        (target as f32 * layout.radius_ratio).clamp(0.0, target as f32 / 2.0),
    );

    canvas
}

fn blit_over(
    dst: &mut [u8],
    dst_w: u32,
    dst_h: u32,
    src: &[u8],
    src_w: u32,
    src_h: u32,
    off_x: i32,
    off_y: i32,
) {
    for sy in 0..src_h {
        for sx in 0..src_w {
            let dx = off_x + sx as i32;
            let dy = off_y + sy as i32;
            if dx < 0 || dy < 0 || dx >= dst_w as i32 || dy >= dst_h as i32 {
                continue;
            }
            let s_idx = ((sy * src_w + sx) * 4) as usize;
            let d_idx = ((dy as u32 * dst_w + dx as u32) * 4) as usize;

            let sa = src[s_idx + 3] as f32 / 255.0;
            if sa <= 0.0 {
                continue;
            }
            let da = dst[d_idx + 3] as f32 / 255.0;
            let out_a = sa + da * (1.0 - sa);
            if out_a <= 0.0 {
                dst[d_idx..d_idx + 4].fill(0);
                continue;
            }

            for c in 0..3 {
                let sc = src[s_idx + c] as f32 / 255.0;
                let dc = dst[d_idx + c] as f32 / 255.0;
                let oc = (sc * sa + dc * da * (1.0 - sa)) / out_a;
                dst[d_idx + c] = (oc * 255.0).round().clamp(0.0, 255.0) as u8;
            }
            dst[d_idx + 3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }
}

fn apply_round_mask(buf: &mut [u8], size: u32, radius: f32) {
    if radius <= 0.0 {
        return;
    }
    let size_f = size as f32;
    let r = radius.min(size_f / 2.0);
    for y in 0..size {
        let y_f = y as f32 + 0.5;
        let cy_opt = if y_f < r {
            Some(r)
        } else if y_f > size_f - r {
            Some(size_f - r)
        } else {
            None
        };

        for x in 0..size {
            let x_f = x as f32 + 0.5;
            let cx_opt = if x_f < r {
                Some(r)
            } else if x_f > size_f - r {
                Some(size_f - r)
            } else {
                None
            };

            let (cx, cy) = match (cx_opt, cy_opt) {
                (Some(cx), Some(cy)) => (cx, cy),
                _ => continue,
            };

            let dx = x_f - cx;
            let dy = y_f - cy;
            if dx * dx + dy * dy > r * r {
                let idx = ((y * size + x) * 4) as usize;
                buf[idx] = 0;
                buf[idx + 1] = 0;
                buf[idx + 2] = 0;
                buf[idx + 3] = 0;
            }
        }
    }
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

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn desktop_entry() -> String {
    format!(
        r#"[Desktop Entry]
Type=Application
Version=1.0
Name=Nesium
Comment=NES emulator
Exec=nesium-egui %f
TryExec=nesium-egui
Icon={app_id}
Terminal=false
Categories=Game;
StartupNotify=true
StartupWMClass={app_id}
"#,
        app_id = APP_ID
    )
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn write_png_file(
    path: &Path,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(rgba)?;
    Ok(())
}

fn encode_png_to_vec(
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buf = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut buf, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(rgba)?;
    }
    Ok(buf)
}

fn write_icns_from_png(path: &Path, png_bytes: &[u8]) -> std::io::Result<()> {
    // Minimal ICNS writer containing a single 256px PNG chunk (ic08).
    let mut out = Vec::with_capacity(8 + 8 + png_bytes.len());
    let total_len = 8 + 8 + png_bytes.len();

    out.extend_from_slice(b"icns");
    out.extend_from_slice(&(total_len as u32).to_be_bytes());
    out.extend_from_slice(b"ic08");
    out.extend_from_slice(&((png_bytes.len() + 8) as u32).to_be_bytes());
    out.extend_from_slice(png_bytes);

    fs::write(path, out)
}

fn target_root() -> PathBuf {
    if let Ok(dir) = env::var("CARGO_TARGET_DIR") {
        return PathBuf::from(dir);
    }
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|root| root.join("target"))
        .unwrap_or_else(|| manifest_dir.join("target"))
}

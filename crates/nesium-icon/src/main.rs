use skia_safe::surfaces::raster_n32_premul;
use skia_safe::{
    Canvas, Color, EncodedImageFormat, Paint, PaintStyle, Path, PathBuilder, PathDirection, PathOp,
    Point, RRect, Rect, Surface, TileMode, gradient_shader,
};
use std::fs::File;
use std::io::Write;

// Canvas size
const WIDTH: i32 = 1024;
const HEIGHT: i32 = 1024;

fn main() -> Result<(), String> {
    // 1) Create a Surface (canvas)
    let mut surface = raster_n32_premul((WIDTH, HEIGHT)).ok_or("Failed to create surface")?;
    let canvas = surface.canvas();

    // 2) Background gradient
    draw_background(canvas);

    // 3) Background ring
    draw_dashed_ring(canvas);

    // 4) Controller body + controls
    draw_controller(canvas);

    // 5) Save PNG
    save_surface(&mut surface, "icon_1024.png")
}

// =====================
// Background
// =====================

/// Background gradient: warm top to cool bottom (closer to the reference).
fn draw_background(canvas: &Canvas) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);

    // Reference: rich warm yellow (top) -> light yellow -> mint -> teal (bottom), top-to-bottom.
    let colors = [
        Color::from_rgb(255, 214, 140), // rich warm yellow (top)
        Color::from_rgb(255, 236, 198), // light warm yellow
        Color::from_rgb(175, 248, 220), // mint green
        Color::from_rgb(110, 230, 225), // cyan/teal (bottom)
    ];
    // Keep the warm yellow more concentrated near the top.
    let pos = [0.0, 0.18, 0.55, 1.0];

    let p1 = Point::new(0.0, 0.0);
    let p2 = Point::new(0.0, HEIGHT as f32);

    paint.set_shader(gradient_shader::linear(
        (p1, p2),
        &colors[..],
        Some(&pos[..]),
        TileMode::Clamp,
        None,
        None,
    ));

    let rect = Rect::from_wh(WIDTH as f32, HEIGHT as f32);
    // Rounded icon background
    canvas.draw_round_rect(rect, 250.0, 250., &paint);
}

// =====================
// Ring
// =====================

/// Segmented ring (arc segments + ticks).
fn draw_dashed_ring(canvas: &Canvas) {
    let center = Point::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);

    let radius = 360.0;
    let oval = Rect::from_xywh(
        center.x - radius,
        center.y - radius,
        radius * 2.0,
        radius * 2.0,
    );

    draw_ring_arcs(canvas, oval);
    draw_ring_ticks(canvas, center, radius);
}

fn draw_ring_arcs(canvas: &Canvas, oval: Rect) {
    // Vertical gradient for arcs (top -> bottom).
    let mut arc_paint = Paint::default();
    arc_paint.set_anti_alias(true);
    arc_paint.set_style(PaintStyle::Stroke);
    arc_paint.set_stroke_width(35.0);

    let p1 = Point::new(0.0, oval.top());
    let p2 = Point::new(0.0, oval.bottom());
    let colors = [
        // top
        Color::from_argb(215, 254, 253, 231),
        // bottom
        Color::from_argb(170, 195, 252, 201),
    ];
    arc_paint.set_shader(gradient_shader::linear(
        (p1, p2),
        &colors[..],
        None,
        TileMode::Clamp,
        None,
        None,
    ));

    let start_angle = 45.0;
    let sweep_angle = 35.0;
    let offset = (start_angle - sweep_angle) / 2.0;

    // 8 arc segments: one every 45° with a gap.
    for i in 0..8 {
        let start = i as f32 * start_angle + offset;
        canvas.draw_arc(oval, start, sweep_angle, false, &arc_paint);
    }
}

fn draw_ring_ticks(canvas: &Canvas, center: Point, radius: f32) {
    let mut tick_paint = Paint::default();
    tick_paint.set_anti_alias(true);
    tick_paint.set_style(PaintStyle::Fill);

    // Interpolate color by device-space Y so ticks actually vary top-to-bottom.
    let top = (215u8, 254u8, 253u8, 231u8); // (a,r,g,b)
    let bot = (170u8, 195u8, 252u8, 201u8);
    let y0 = center.y - radius;
    let y1 = center.y + radius;

    let lerp_u8 = |a: u8, b: u8, t: f32| -> u8 {
        let af = a as f32;
        let bf = b as f32;
        ((af + (bf - af) * t).round() as i32).clamp(0, 255) as u8
    };

    // Global tick controls
    let small_len = 44.0; // small tick length (every 45°)
    let big_len = 70.0; // big tick length (every 90°)
    let small_thickness = 15.;
    let big_thickness = 26.;
    let corner = 6.0; // tick corner radius
    let outset = 5.0; // push ticks outward from the ring radius

    // 0..8 => 0°,45°,...,315°
    for i in 0..8 {
        let angle_deg = i as f32 * 45.0;
        let is_big = i % 2 == 0; // 0,2,4,6 => 0/90/180/270
        let len = if is_big { big_len } else { small_len };
        let thickness = if is_big {
            big_thickness
        } else {
            small_thickness
        };

        // Angle convention: 0° at 3 o'clock, increasing clockwise.
        let rad = angle_deg.to_radians();
        let dist = radius + outset;
        let x = center.x + rad.cos() * dist;
        let y = center.y + rad.sin() * dist;

        // Lerp tick color by its center Y (0=top, 1=bottom).
        let t = if (y1 - y0).abs() < f32::EPSILON {
            0.0
        } else {
            ((y - y0) / (y1 - y0)).clamp(0.0, 1.0)
        };
        let a = lerp_u8(top.0, bot.0, t);
        let r = lerp_u8(top.1, bot.1, t);
        let g = lerp_u8(top.2, bot.2, t);
        let b = lerp_u8(top.3, bot.3, t);
        tick_paint.set_color(Color::from_argb(a, r, g, b));

        // Local transform at the tick center, rotate so the long axis points outward.
        canvas.save();
        canvas.translate((x, y));
        canvas.rotate(angle_deg, None);

        let rect = Rect::from_xywh(-len / 2.0, -thickness / 2.0, len, thickness);
        canvas.draw_round_rect(rect, corner, corner, &tick_paint);

        canvas.restore();
    }
}

// =====================
// Controller
// =====================

fn draw_controller(canvas: &Canvas) {
    let geom = ControllerGeom::new();

    draw_controller_shell(canvas, &geom);
    draw_controller_controls(canvas, &geom);
}

#[derive(Clone, Copy, Debug)]
struct ControllerGeom {
    center: Point,
    rect: Rect,
    rrect: RRect,
    inner_rrect: RRect,
}

impl ControllerGeom {
    fn new() -> Self {
        let center = Point::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);

        // Controller body size
        let w = 780.0;
        let h = 340.0;
        let r = 40.0;

        let rect = Rect::from_xywh(center.x - w / 2.0, center.y - h / 2.0, w, h);
        let rrect = RRect::new_rect_xy(&rect, r, r);

        // Inner border geometry
        let inset = 18.0;
        let inner_rect = Rect::from_xywh(
            rect.left() + inset,
            rect.top() + inset,
            w - inset * 2.0,
            h - inset * 2.0,
        );
        let inner_r = (r - inset).max(0.0);
        let inner_rrect = RRect::new_rect_xy(&inner_rect, inner_r, inner_r);

        Self {
            center,
            rect,
            rrect,
            inner_rrect,
        }
    }
}

fn draw_controller_shell(canvas: &Canvas, geom: &ControllerGeom) {
    // A) Fill: subtle teal gradient
    let mut fill_paint = Paint::default();
    fill_paint.set_anti_alias(true);

    let fill_colors = [
        Color::from_rgb(200, 255, 245),
        Color::from_rgb(120, 235, 230),
    ];
    let p1 = Point::new(geom.rect.left(), geom.rect.top());
    let p2 = Point::new(geom.rect.right(), geom.rect.bottom());
    fill_paint.set_shader(gradient_shader::linear(
        (p1, p2),
        &fill_colors[..],
        None,
        TileMode::Clamp,
        None,
        None,
    ));
    canvas.draw_rrect(&geom.rrect, &fill_paint);

    // B) Inner white border
    let mut inner_stroke = Paint::default();
    inner_stroke.set_anti_alias(true);
    inner_stroke.set_color(Color::from_argb(180, 255, 255, 255));
    inner_stroke.set_style(PaintStyle::Stroke);
    inner_stroke.set_stroke_width(10.0);
    canvas.draw_rrect(&geom.inner_rrect, &inner_stroke);

    // C) Top highlight (upper half only)
    let mut top_hi = Paint::default();
    top_hi.set_anti_alias(true);
    top_hi.set_color(Color::from_argb(110, 255, 255, 255));
    top_hi.set_style(PaintStyle::Stroke);
    top_hi.set_stroke_width(22.0);

    canvas.save();
    let clip = Rect::from_xywh(
        geom.rect.left(),
        geom.rect.top(),
        geom.rect.width(),
        geom.rect.height() * 0.55,
    );
    canvas.clip_rect(clip, None, true);
    canvas.draw_rrect(&geom.inner_rrect, &top_hi);
    canvas.restore();

    // D) Dark outer stroke
    let mut stroke_paint = Paint::default();
    stroke_paint.set_anti_alias(true);
    stroke_paint.set_color(Color::from_rgb(30, 50, 80));
    stroke_paint.set_style(PaintStyle::Stroke);
    stroke_paint.set_stroke_width(28.0);
    canvas.draw_rrect(&geom.rrect, &stroke_paint);
}

fn draw_controller_controls(canvas: &Canvas, geom: &ControllerGeom) {
    // Left D-pad
    draw_dpad(canvas, geom.center);

    // Center minus button
    draw_minus_button(canvas, geom.center);

    // Right face pad + buttons
    draw_face_pad(canvas, geom.center);
}

// ---------------------
// D-Pad
// ---------------------

fn draw_dpad(canvas: &Canvas, center: Point) {
    let geom = DPadGeom::new(center);

    // Subtle outer outline
    draw_dpad_outline(canvas, &geom);
    // Main fill
    draw_dpad_body(canvas, &geom);
    // Top highlight
    draw_dpad_highlight(canvas, &geom);
}

#[derive(Clone, Copy, Debug)]
struct DPadLayer {
    v_rect: Rect,
    h_rect: Rect,
}

#[derive(Clone, Copy, Debug)]
struct DPadGeom {
    main: DPadLayer,
    radius: f32,
}

impl DPadGeom {
    fn new(center: Point) -> Self {
        // D-pad center position
        let dpad_x = center.x - 240.0;
        let dpad_y = center.y;

        // D-pad dimensions
        let arm_len = 60.0;
        let arm_thick = 30.0;

        let main = Self::make_layer(dpad_x, dpad_y, arm_len, arm_thick);

        Self { main, radius: 2. }
    }

    fn make_layer(cx: f32, cy: f32, arm_len: f32, arm_thick: f32) -> DPadLayer {
        // Two rounded rects form a plus.
        let v_rect = Rect::from_xywh(
            cx - arm_thick,
            cy - arm_len - arm_thick,
            arm_thick * 2.0,
            arm_len * 2.0 + arm_thick * 2.0,
        );
        let h_rect = Rect::from_xywh(
            cx - arm_len - arm_thick,
            cy - arm_thick,
            arm_len * 2.0 + arm_thick * 2.0,
            arm_thick * 2.0,
        );
        DPadLayer { v_rect, h_rect }
    }
}

fn build_dpad_path(geom: &DPadGeom) -> Path {
    // Union the two rounded-rect arms into a single plus outline.
    let vr = RRect::new_rect_xy(&geom.main.v_rect, geom.radius, geom.radius);
    let hr = RRect::new_rect_xy(&geom.main.h_rect, geom.radius, geom.radius);

    let v_path = Path::rrect(vr, Some(PathDirection::CW));
    let h_path = Path::rrect(hr, Some(PathDirection::CW));

    // Boolean union (fallback to simple add on failure).
    v_path.op(&h_path, PathOp::Union).unwrap_or_else(|| {
        let mut pb = PathBuilder::new();
        pb.add_rrect(vr, Some(PathDirection::CW), None);
        pb.add_rrect(hr, Some(PathDirection::CW), None);
        pb.detach()
    })
}

fn draw_dpad_body(canvas: &Canvas, geom: &DPadGeom) {
    let path = build_dpad_path(geom);

    let mut fill = Paint::default();
    fill.set_anti_alias(true);
    fill.set_style(PaintStyle::Fill);
    fill.set_color(Color::from_rgb(30, 50, 80));

    canvas.draw_path(&path, &fill);
}

fn draw_dpad_outline(canvas: &Canvas, geom: &DPadGeom) {
    let path = build_dpad_path(geom);

    // Draw a uniform outline by stroking the unified path.
    let outline_w = 5.0;

    let mut stroke = Paint::default();
    stroke.set_anti_alias(true);
    stroke.set_style(PaintStyle::Stroke);
    stroke.set_stroke_width(outline_w * 2.0);
    stroke.set_color(Color::from_rgb(18, 32, 58));

    canvas.draw_path(&path, &stroke);
}

// ---------------------
// D-Pad highlight
// ---------------------

fn draw_dpad_highlight(canvas: &Canvas, geom: &DPadGeom) {
    let path = build_dpad_path(geom);

    // Highlight: a soft top inner edge.
    let hi_w = 15.;
    let fade_h = 8.0;

    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(PaintStyle::Stroke);
    paint.set_stroke_width(hi_w);

    // Bright at the top, transparent below.
    let colors = [
        Color::from_argb(180, 255, 255, 255),
        Color::from_argb(0, 255, 255, 255),
    ];
    let pos = [0.0, 1.0];

    // Clip to the inside of the D-pad so the stroke becomes an inner highlight.
    canvas.save();
    canvas.clip_path(&path, None, true);

    // Draw only within the provided top band.
    let mut draw_band = |band: Rect| {
        // Gradient starts at band.top.
        let p1 = Point::new(0.0, band.top());
        let p2 = Point::new(0.0, band.top() + fade_h);
        paint.set_shader(gradient_shader::linear(
            (p1, p2),
            &colors[..],
            Some(&pos[..]),
            TileMode::Clamp,
            None,
            None,
        ));

        canvas.save();
        canvas.clip_rect(band, None, true);
        canvas.draw_path(&path, &paint);
        canvas.restore();
    };

    // 1) Horizontal arm top band (split left/right to avoid the vertical junction).
    {
        let r = geom.main.h_rect;
        let band = Rect::from_xywh(
            r.left(),
            r.top(),
            r.width() / 2. - geom.main.v_rect.width() / 2.,
            fade_h + hi_w,
        );
        draw_band(band);

        let band = Rect::from_xywh(
            r.left() + r.width() / 2. + geom.main.v_rect.width() / 2.,
            r.top(),
            r.width() / 2. - geom.main.v_rect.width() / 2.,
            fade_h + hi_w,
        );
        draw_band(band);
    }

    // 2) Vertical arm top cap band.
    {
        let r = geom.main.v_rect;
        let band = Rect::from_xywh(r.left(), r.top(), r.width(), fade_h + hi_w);
        draw_band(band);
    }

    canvas.restore();
}

fn draw_minus_button(canvas: &Canvas, center: Point) {
    let rect = Rect::from_xywh(center.x - 30.0, center.y - 12.0, 60.0, 24.0);
    let radius = 8.0;
    let off = 6.0;

    let mut paint = Paint::default();
    paint.set_anti_alias(true);

    // Shadow
    paint.set_color(Color::from_argb(160, 15, 25, 40));
    canvas.draw_round_rect(
        Rect::from_xywh(
            rect.left() + off,
            rect.top() + off,
            rect.width(),
            rect.height(),
        ),
        radius,
        radius,
        &paint,
    );

    // Body
    paint.set_color(Color::from_rgb(30, 50, 80));
    canvas.draw_round_rect(rect, radius, radius, &paint);
}

// ---------------------
// Face pad + buttons
// ---------------------

fn draw_face_pad(canvas: &Canvas, center: Point) {
    let with = 250.;
    let height = 200.;
    let pad_rect = Rect::from_xywh(center.x + 80.0, center.y - height / 2., with, height);
    let pad_rrect = RRect::new_rect_xy(&pad_rect, 90.0, 90.0);

    draw_face_pad_base(canvas, &pad_rrect);
    draw_face_buttons(canvas, pad_rect, center.y);
}

fn draw_face_pad_base(canvas: &Canvas, pad: &RRect) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(Color::from_rgb(30, 50, 80));
    canvas.draw_rrect(pad, &paint);
}

fn draw_face_buttons(canvas: &Canvas, pad_rect: Rect, btn_center_y: f32) {
    let pad_cx = pad_rect.center_x();

    let btn_r = 36.0;
    let btn_dx = 50.0;

    let mut paint = Paint::default();
    paint.set_anti_alias(true);

    // Yellow button
    paint.set_color(Color::from_rgb(255, 210, 60));
    canvas.draw_circle((pad_cx - btn_dx, btn_center_y), btn_r, &paint);

    // Orange/red button
    paint.set_color(Color::from_rgb(255, 120, 90));
    canvas.draw_circle((pad_cx + btn_dx, btn_center_y), btn_r, &paint);

    // Specular dots
    paint.set_color(Color::from_argb(220, 255, 255, 255));
    canvas.draw_circle((pad_cx - btn_dx, btn_center_y - 14.0), 7.5, &paint);
    canvas.draw_circle((pad_cx + btn_dx, btn_center_y - 14.0), 7.5, &paint);
}

// =====================
// Save
// =====================

/// Helper to save the rendered surface as a PNG.
fn save_surface(surface: &mut Surface, path: &str) -> Result<(), String> {
    let image = surface.image_snapshot();
    let data = image
        .encode(None, EncodedImageFormat::PNG, 100)
        .ok_or("Failed to encode image")?;

    let mut file = File::create(path).map_err(|e| e.to_string())?;
    file.write_all(data.as_bytes()).map_err(|e| e.to_string())?;

    println!("Successfully generated: {}", path);
    Ok(())
}

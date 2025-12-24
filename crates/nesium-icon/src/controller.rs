use crate::dimensions::{HEIGHT, WIDTH};
use skia_safe::{
    Canvas, Color, Paint, PaintStyle, Path, PathBuilder, PathDirection, PathFillType, PathOp,
    Point, RRect, Rect, TileMode, gradient_shader,
};
use std::f32::consts::TAU;

pub fn draw_controller(canvas: &Canvas) {
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
    radius: f32,
}

impl ControllerGeom {
    fn new() -> Self {
        let center = Point::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);

        // Controller body size
        let w = 819.;
        let h = 357.;
        let r = 63.;

        let rect = Rect::from_xywh(center.x - w / 2.0, center.y - h / 2.0, w, h);
        let rrect = RRect::new_rect_xy(rect, r, r);

        // Inner border geometry
        let inset = 18.0;
        let inner_rect = Rect::from_xywh(
            rect.left() + inset,
            rect.top() + inset,
            w - inset * 2.0,
            h - inset * 2.0,
        );
        let inner_r = (r - inset).max(0.0);
        let inner_rrect = RRect::new_rect_xy(inner_rect, inner_r, inner_r);

        Self {
            center,
            rect,
            rrect,
            inner_rrect,
            radius: r,
        }
    }
}

fn draw_controller_shell(canvas: &Canvas, geom: &ControllerGeom) {
    // A) Fill: subtle teal gradient
    let mut fill_paint = Paint::default();
    fill_paint.set_anti_alias(true);

    let fill_colors = [
        Color::from_rgb(204, 252, 213),
        Color::from_rgb(121, 222, 206),
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
    canvas.draw_rrect(geom.rrect, &fill_paint);

    // B) Inner white border
    let mut inner_stroke = Paint::default();
    inner_stroke.set_anti_alias(true);
    inner_stroke.set_color(Color::from_argb(180, 255, 255, 255));
    inner_stroke.set_style(PaintStyle::Stroke);
    inner_stroke.set_stroke_width(10.0);
    canvas.draw_rrect(geom.inner_rrect, &inner_stroke);

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
    canvas.draw_rrect(geom.inner_rrect, &top_hi);
    canvas.restore();

    // D) Dark outer stroke with variable thickness (thicker bottom, thinner top)
    draw_variable_border_rrect(
        canvas,
        geom.rect,
        geom.radius,
        16.0,
        28.0,
        Color::from_rgb(30, 50, 80),
    );
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn smootherstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Variable border only on the TOP side:
/// - top edge thickness = w_top
/// - right/left/bottom edges thickness = w_other
/// - transition happens only on top-left & top-right corner arcs.
fn draw_variable_border_rrect(
    canvas: &Canvas,
    rect: Rect,
    radius: f32,
    w_top: f32,
    w_other: f32,
    color: Color,
) {
    let left = rect.left();
    let right = rect.right();
    let top = rect.top();
    let bottom = rect.bottom();

    let r = radius.min(rect.width() * 0.5).min(rect.height() * 0.5);

    // Increase for smoother corners if you see faceting.
    let samples_edge = 48;
    let samples_arc = 30;

    // Store: (pos, outward_normal, thickness_at_pos)
    let mut outer: Vec<(Point, Point, f32)> = Vec::new();

    // ---- Top edge (thin) ----
    for i in 0..=samples_edge {
        let t = i as f32 / samples_edge as f32;
        let x = (left + r) + (right - r - (left + r)) * t;
        outer.push((Point::new(x, top), Point::new(0.0, -1.0), w_top));
    }

    // ---- Top-right arc: thin -> thick ----
    // angles: -90° -> 0° ; u=0 at top, u=1 at right
    {
        let cx = right - r;
        let cy = top + r;
        for i in 0..=samples_arc {
            let u = i as f32 / samples_arc as f32;
            let a = (-90.0 + 90.0 * u).to_radians();
            let nx = a.cos();
            let ny = a.sin();
            let p = Point::new(cx + r * nx, cy + r * ny);
            let n = Point::new(nx, ny);
            let s = smootherstep(u).powf(1.8);
            let w = lerp(w_top, w_other, s);
            outer.push((p, n, w));
        }
    }

    // ---- Right edge (thick) ----
    for i in 0..=samples_edge {
        let t = i as f32 / samples_edge as f32;
        let y = (top + r) + (bottom - r - (top + r)) * t;
        outer.push((Point::new(right, y), Point::new(1.0, 0.0), w_other));
    }

    // ---- Bottom-right arc (thick) ----
    // angles: 0° -> 90°
    {
        let cx = right - r;
        let cy = bottom - r;
        for i in 0..=samples_arc {
            let u = i as f32 / samples_arc as f32;
            let a = (0.0 + 90.0 * u).to_radians();
            let nx = a.cos();
            let ny = a.sin();
            let p = Point::new(cx + r * nx, cy + r * ny);
            let n = Point::new(nx, ny);
            outer.push((p, n, w_other));
        }
    }

    // ---- Bottom edge (thick) ----
    for i in 0..=samples_edge {
        let t = i as f32 / samples_edge as f32;
        let x = (right - r) + (left + r - (right - r)) * t;
        outer.push((Point::new(x, bottom), Point::new(0.0, 1.0), w_other));
    }

    // ---- Bottom-left arc (thick) ----
    // angles: 90° -> 180°
    {
        let cx = left + r;
        let cy = bottom - r;
        for i in 0..=samples_arc {
            let u = i as f32 / samples_arc as f32;
            let a = (90.0 + 90.0 * u).to_radians();
            let nx = a.cos();
            let ny = a.sin();
            let p = Point::new(cx + r * nx, cy + r * ny);
            let n = Point::new(nx, ny);
            outer.push((p, n, w_other));
        }
    }

    // ---- Left edge (thick) ----
    for i in 0..=samples_edge {
        let t = i as f32 / samples_edge as f32;
        let y = (bottom - r) + (top + r - (bottom - r)) * t;
        outer.push((Point::new(left, y), Point::new(-1.0, 0.0), w_other));
    }

    // ---- Top-left arc: thin -> thick ----
    // angles: 180° -> 270°
    // We want u=0 at top side, u=1 at left side, so use u = 1 - u0.
    {
        let cx = left + r;
        let cy = top + r;
        for i in 0..=samples_arc {
            let u0 = i as f32 / samples_arc as f32;
            let a = (180.0 + 90.0 * u0).to_radians();
            let nx = a.cos();
            let ny = a.sin();
            let p = Point::new(cx + r * nx, cy + r * ny);
            let n = Point::new(nx, ny);

            let u = 1.0 - u0; // u=0 at top, u=1 at left
            let s = smootherstep(u).powf(1.8);
            let w = lerp(w_top, w_other, s);
            outer.push((p, n, w));
        }
    }

    // Build an EvenOdd ring: outer contour + inner contour (reversed)
    let mut pb = PathBuilder::new();
    pb.set_fill_type(PathFillType::EvenOdd);

    // Outer
    pb.move_to(outer[0].0);
    for (p, _, _) in outer.iter().skip(1) {
        pb.line_to(*p);
    }
    pb.close();

    // Inner (reverse)
    let mut first = true;
    for (p, n, w) in outer.iter().rev() {
        let inner = Point::new(p.x - n.x * *w, p.y - n.y * *w);
        if first {
            pb.move_to(inner);
            first = false;
        } else {
            pb.line_to(inner);
        }
    }
    pb.close();

    let path = pb.detach();

    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(PaintStyle::Fill);
    paint.set_color(color);

    canvas.draw_path(&path, &paint);
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
        let dpad_x = center.x - 220.0;
        let dpad_y = center.y;

        // D-pad dimensions
        let arm_len = 75.0;
        let arm_thick = 35.0;

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
    let vr = RRect::new_rect_xy(geom.main.v_rect, geom.radius, geom.radius);
    let hr = RRect::new_rect_xy(geom.main.h_rect, geom.radius, geom.radius);

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
    let rect = Rect::from_xywh(center.x - 80., center.y + 60., 100.0, 40.0);
    let radius = 6.0;

    let mut paint = Paint::default();
    paint.set_anti_alias(true);

    // Shadow
    paint.set_color(Color::from_argb(160, 15, 25, 40));
    canvas.draw_round_rect(
        Rect::from_xywh(rect.left(), rect.top(), rect.width(), rect.height()),
        radius,
        radius,
        &paint,
    );

    // Body
    paint.set_color(Color::from_rgb(30, 50, 80));
    canvas.draw_round_rect(rect, radius, radius, &paint);

    // Thin outline (match D-pad outline tone)
    paint.set_style(PaintStyle::Stroke);
    paint.set_stroke_width(4.0);
    paint.set_color(Color::from_rgb(18, 32, 58));
    canvas.draw_round_rect(rect, radius, radius, &paint);
}

// ---------------------
// Face pad + buttons
// ---------------------
fn draw_face_pad(canvas: &Canvas, center: Point) {
    let with = 300.;
    let height = 220.;
    let pad_rect = Rect::from_xywh(center.x + 40.0, center.y - height / 2., with, height);
    let pad_rrect = RRect::new_rect_xy(pad_rect, 120.0, 120.0);

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

    let btn_r = 50.0;
    let btn_dx = 60.0;
    let btn_dy = -10.;

    let left_center = Point::new(pad_cx - btn_dx, btn_center_y - btn_dy);
    let right_center = Point::new(pad_cx + btn_dx, btn_center_y - btn_dy);

    draw_button_with_highlight(canvas, left_center, btn_r, Color::from_rgb(255, 210, 60));
    draw_button_with_highlight(canvas, right_center, btn_r, Color::from_rgb(255, 120, 90));
}

fn draw_button_with_highlight(canvas: &Canvas, center: Point, radius: f32, fill: Color) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);

    // Base fill
    paint.set_style(PaintStyle::Fill);
    paint.set_color(fill);
    canvas.draw_circle(center, radius, &paint);

    draw_button_highlight(canvas, center, radius);
}

fn draw_button_highlight(canvas: &Canvas, center: Point, radius: f32) {
    // Build a variable-width ring (thickest at upper-left, thinnest at lower-right).
    let path = build_variable_ring(center, radius, 12.0, 6.0);
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(PaintStyle::Fill);
    paint.set_color(Color::from_rgb(250, 250, 250));
    canvas.draw_path(&path, &paint);
}

fn build_variable_ring(center: Point, radius: f32, w_max: f32, w_min: f32) -> Path {
    let n = 256usize;
    let phi0 = (-135.0_f32).to_radians(); // thickest at upper-left

    let mut outer: Vec<Point> = Vec::with_capacity(n);
    let mut inner: Vec<Point> = Vec::with_capacity(n);

    for i in 0..n {
        let t = i as f32 / n as f32;
        let theta = t * TAU;

        // Cosine weight: max thickness at phi0, min on the opposite side.
        let k = 0.5 * (1.0 + (theta - phi0).cos());
        let w = w_min + (w_max - w_min) * k;

        let ro = radius + w * 0.5;
        let ri = radius - w * 0.5;

        outer.push(Point::new(
            center.x + ro * theta.cos(),
            center.y + ro * theta.sin(),
        ));
        inner.push(Point::new(
            center.x + ri * theta.cos(),
            center.y + ri * theta.sin(),
        ));
    }

    let mut pb = PathBuilder::new();
    pb.set_fill_type(PathFillType::EvenOdd);

    // Outer contour (clockwise)
    pb.move_to(outer[0]);
    for p in &outer[1..] {
        pb.line_to(*p);
    }
    pb.close();

    // Inner contour (reverse)
    pb.move_to(*inner.last().unwrap());
    for p in inner.iter().rev().skip(1) {
        pb.line_to(*p);
    }
    pb.close();

    pb.detach()
}

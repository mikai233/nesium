use crate::dimensions::{HEIGHT, WIDTH};
use skia_safe::{
    Canvas, Color, Paint, PaintStyle, Path, PathBuilder, PathDirection, PathOp, Point, RRect, Rect,
    TileMode, gradient_shader,
};

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

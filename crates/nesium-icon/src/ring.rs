use crate::dimensions::{HEIGHT, WIDTH};
use skia_safe::{Canvas, Color, Paint, PaintStyle, Point, Rect, TileMode, gradient_shader};

/// Segmented ring (arc segments + ticks).
pub fn draw_dashed_ring(canvas: &Canvas) {
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

    let color_for_y = |y: f32| {
        let t = if (y1 - y0).abs() < f32::EPSILON {
            0.0
        } else {
            ((y - y0) / (y1 - y0)).clamp(0.0, 1.0)
        };
        let a = lerp_u8(top.0, bot.0, t);
        let r = lerp_u8(top.1, bot.1, t);
        let g = lerp_u8(top.2, bot.2, t);
        let b = lerp_u8(top.3, bot.3, t);
        Color::from_argb(a, r, g, b)
    };

    // Global tick controls
    let small_len = 44.0; // small tick length (every 45°)
    let big_len = 70.0; // big tick length (every 90°)
    let small_thickness = 15.;
    let big_thickness = 26.;
    let corner = 3.0; // tick corner radius
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

        tick_paint.set_color(color_for_y(y));

        // Local transform at the tick center, rotate so the long axis points outward.
        canvas.save();
        canvas.translate((x, y));
        canvas.rotate(angle_deg, None);

        let rect = Rect::from_xywh(-len / 2.0, -thickness / 2.0, len, thickness);
        canvas.draw_round_rect(rect, corner, corner, &tick_paint);

        canvas.restore();
    }

    // Inner ticks: 6 total, aligned at top (big) and +/-30° plus mirrored bottom.
    let inner_angles: [f32; 6] = [270.0, 240.0, 300.0, 90.0, 60.0, 120.0];
    let inner_big = 48.0;
    let inner_small = 32.0;
    let inner_thick_big = 18.0;
    let inner_thick_small = 16.0;
    let inner_corner = 6.0;
    let inset = 80.0; // pull ticks inside the ring arc

    for angle_deg in inner_angles {
        let is_big = (angle_deg - 270.0_f32).abs() < f32::EPSILON
            || (angle_deg - 90.0_f32).abs() < f32::EPSILON;
        let len = if is_big { inner_big } else { inner_small };
        let thickness = if is_big {
            inner_thick_big
        } else {
            inner_thick_small
        };

        let rad = angle_deg.to_radians();
        let dist = radius - inset;
        let x = center.x + rad.cos() * dist;
        let y = center.y + rad.sin() * dist;

        tick_paint.set_color(color_for_y(y));

        canvas.save();
        canvas.translate((x, y));
        canvas.rotate(angle_deg, None);

        let rect = Rect::from_xywh(-len / 2.0, -thickness / 2.0, len, thickness);
        canvas.draw_round_rect(rect, inner_corner, inner_corner, &tick_paint);

        canvas.restore();
    }
}

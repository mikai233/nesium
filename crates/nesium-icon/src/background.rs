use crate::dimensions::{HEIGHT, WIDTH};
use skia_safe::{Canvas, Color, Paint, Point, Rect, TileMode, gradient_shader};

/// Background gradient: warm top to cool bottom (closer to the reference).
pub fn draw_background(canvas: &Canvas) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);

    // Option 1: clean blue-green vertical gradient.
    let colors = [
        Color::from_rgb(183, 227, 255), // top: soft sky blue
        Color::from_rgb(92, 192, 243),  // bottom: brighter cyan-blue
    ];
    let pos = [0.0, 1.0];

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

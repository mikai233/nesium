use crate::dimensions::{HEIGHT, WIDTH};
use skia_safe::{Canvas, Color, Paint, Point, Rect, TileMode, gradient_shader};

/// Background gradient: warm top to cool bottom (closer to the reference).
pub fn draw_background(canvas: &Canvas) {
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

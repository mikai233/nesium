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
use skia_safe::surfaces::raster_n32_premul;

fn main() -> Result<(), String> {
    let mut surface = raster_n32_premul((WIDTH, HEIGHT)).ok_or("Failed to create surface")?;
    let canvas = surface.canvas();

    // 1) Background gradient
    draw_background(canvas);

    // 2) Background ring
    draw_dashed_ring(canvas);

    // 3) Controller body + controls
    draw_controller(canvas);

    // 4) Save PNG
    save_surface(&mut surface, "icon_1024.png")
}

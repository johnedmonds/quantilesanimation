use std::fs::File;

use ggez::graphics::Color;
use lazy_static::lazy_static;
use png::{BitDepth, ColorType};
use rand_distr::Normal;
pub type Element = u32;
pub const ELEMENT_WIDTH: u32 = 10;
pub const SPACE_BETWEEN_ELEMENTS: u32 = 2;
pub const MAX_ELEMENT_HEIGHT: u32 = 20;
pub const CAPACITY_RECT_COLOR: Color = Color {
    r: 0.1,
    g: 0.1,
    b: 0.1,
    a: 1.0,
};
pub const USED_CAPACITY_RECT_COLOR: Color = Color {
    r: 0.25,
    g: 0.25,
    b: 0.25,
    a: 1.0,
};
lazy_static! {
    pub static ref DISTRIBUTION: Normal<f32> = Normal::new(10.0, 3.0).unwrap();
    pub static ref DISTRIBUTION_MIN: f32 = DISTRIBUTION.mean() - DISTRIBUTION.std_dev() * 4.0;
    pub static ref DISTRIBUTION_PRACTICAL_RANGE: f32 =
        (DISTRIBUTION.mean() + DISTRIBUTION.std_dev() * 3.0) - *DISTRIBUTION_MIN;
}
pub fn save_frame(frame: Vec<u8>, frame_id: u32, width: u32, height: u32) {
    let mut encoder = png::Encoder::new(
        File::create(format!("frame{:0w$}.png", frame_id, w = 4)).unwrap(),
        width,
        height,
    );
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&frame)
        .unwrap();
}
use ggez::graphics::Color;
use lazy_static::lazy_static;
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

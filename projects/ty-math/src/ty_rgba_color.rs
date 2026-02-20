/// An RGBA color with `f32` components.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TyRgbaColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl TyRgbaColor {
    /// Creates a new color from `r`, `g`, `b`, and `a` components.
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

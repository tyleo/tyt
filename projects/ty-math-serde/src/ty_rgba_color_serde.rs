use serde::{Deserialize, Serialize};
use ty_math::TyRgbaColor;

/// Serde-compatible parity type for [`TyRgbaColor`].
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct TyRgbaColorSerde {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<TyRgbaColor> for TyRgbaColorSerde {
    fn from(c: TyRgbaColor) -> Self {
        Self { r: c.r, g: c.g, b: c.b, a: c.a }
    }
}

impl From<TyRgbaColorSerde> for TyRgbaColor {
    fn from(c: TyRgbaColorSerde) -> Self {
        Self::new(c.r, c.g, c.b, c.a)
    }
}

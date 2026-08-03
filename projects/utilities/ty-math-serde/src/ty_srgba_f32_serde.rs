use serde::{Deserialize, Serialize};
use ty_math::TySrgbaF32;

/// Serde-compatible parity type for [`TySrgbaF32`].
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct TySrgbaF32Serde {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<TySrgbaF32> for TySrgbaF32Serde {
    fn from(c: TySrgbaF32) -> Self {
        Self {
            r: c.red,
            g: c.green,
            b: c.blue,
            a: c.alpha,
        }
    }
}

impl From<TySrgbaF32Serde> for TySrgbaF32 {
    fn from(c: TySrgbaF32Serde) -> Self {
        Self::new(c.r, c.g, c.b, c.a)
    }
}

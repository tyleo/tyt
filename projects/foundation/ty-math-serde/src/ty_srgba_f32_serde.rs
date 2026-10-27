use serde::{Deserialize, Serialize};
use ty_math::TySrgbaF32;

/// Serde-compatible parity type for [`TySrgbaF32`].
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct TySrgbaF32Serde {
    /// The red channel.
    pub r: f32,

    /// The green channel.
    pub g: f32,

    /// The blue channel.
    pub b: f32,

    /// The alpha channel.
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

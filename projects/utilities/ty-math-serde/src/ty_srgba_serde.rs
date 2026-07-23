use serde::{Deserialize, Serialize};
use ty_math::TySrgba;

/// Serde-compatible parity type for [`TySrgba`].
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct TySrgbaSerde {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<TySrgba> for TySrgbaSerde {
    fn from(c: TySrgba) -> Self {
        Self {
            r: c.red,
            g: c.green,
            b: c.blue,
            a: c.alpha,
        }
    }
}

impl From<TySrgbaSerde> for TySrgba {
    fn from(c: TySrgbaSerde) -> Self {
        Self::new(c.r, c.g, c.b, c.a)
    }
}

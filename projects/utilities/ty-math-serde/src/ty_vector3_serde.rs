use serde::{Deserialize, Serialize};
use ty_math::TyVector3;

/// Serde-compatible parity type for [`TyVector3`].
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct TyVector3Serde {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl From<TyVector3> for TyVector3Serde {
    fn from(v: TyVector3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<TyVector3Serde> for TyVector3 {
    fn from(v: TyVector3Serde) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

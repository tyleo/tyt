use serde::{Deserialize, Serialize};
use ty_math::TyVector3F64;

/// Serde-compatible parity type for [`TyVector3F64`].
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct TyVector3F64Serde {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl From<TyVector3F64> for TyVector3F64Serde {
    fn from(v: TyVector3F64) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<TyVector3F64Serde> for TyVector3F64 {
    fn from(v: TyVector3F64Serde) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

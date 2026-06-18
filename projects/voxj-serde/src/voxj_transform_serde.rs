use serde::{Deserialize, Serialize};
use voxj::VoxjTransform;

/// Serde-compatible parity type for [`VoxjTransform`].
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct VoxjTransformSerde {
    pub position: [f64; 3],
    pub rotation: [f64; 4],
    pub scale: [f64; 3],
}

impl From<VoxjTransform> for VoxjTransformSerde {
    fn from(v: VoxjTransform) -> Self {
        Self {
            position: v.position,
            rotation: v.rotation,
            scale: v.scale,
        }
    }
}

impl From<VoxjTransformSerde> for VoxjTransform {
    fn from(v: VoxjTransformSerde) -> Self {
        Self {
            position: v.position,
            rotation: v.rotation,
            scale: v.scale,
        }
    }
}

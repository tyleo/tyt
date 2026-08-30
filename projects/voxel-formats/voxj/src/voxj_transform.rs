#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A node transform, composing as `Translation * Rotation * Scale`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VoxjTransform {
    /// `[x, y, z]`, may be fractional.
    pub position: [f64; 3],

    /// Unit quaternion `[x, y, z, w]`.
    pub rotation: [f64; 4],

    /// Per-axis `[x, y, z]`.
    pub scale: [f64; 3],
}
